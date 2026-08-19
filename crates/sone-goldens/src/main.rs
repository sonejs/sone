use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rayon::prelude::*;
use serde::Deserialize;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_skia::render::{Engine, RenderOptions};

const DEFAULT_THRESHOLD: f64 = 0.02;

#[derive(Debug, Default, Deserialize)]
struct Waivers {
    #[serde(default)]
    fixture: HashMap<String, Waiver>,
}

#[derive(Debug, Deserialize)]
struct Waiver {
    /// Written justification — required, so waivers cannot be silent.
    reason: String,
    #[serde(default)]
    threshold: Option<f64>,
    #[serde(default)]
    skip: bool,
}

struct Outcome {
    name: String,
    dssim: Option<f64>,
    threshold: f64,
    status: Status,
    note: String,
}

#[derive(PartialEq)]
enum Status {
    Pass,
    Fail,
    Waived,
    Skipped,
    Error,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn main() -> ExitCode {
    let root = repo_root();
    let ir_dir = root.join("fixtures/visual/ir");
    let golden_dir = root.join("fixtures/visual");
    let out_dir = root.join("target/goldens");
    std::fs::create_dir_all(&out_dir).ok();

    let waivers: Waivers = std::fs::read_to_string(root.join("goldens-waivers.toml"))
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();

    let filter = std::env::args().nth(1);

    let mut names: Vec<String> = std::fs::read_dir(&ir_dir)
        .expect("run `tools/sync-fixtures.sh <path-to-sone-checkout>` first")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
        .filter(|n| golden_dir.join(format!("{n}.jpg")).exists())
        .filter(|n| {
            filter
                .as_ref()
                .map(|f| n.contains(f.as_str()))
                .unwrap_or(true)
        })
        .collect();
    names.sort();

    let outcomes: Vec<Outcome> = names
        .par_iter()
        .map(|name| run_one(name, &ir_dir, &golden_dir, &out_dir, &waivers))
        .collect();

    let mut failed = 0;
    for o in &outcomes {
        let label = match o.status {
            Status::Pass => "ok  ",
            Status::Waived => "waiv",
            Status::Skipped => "skip",
            Status::Fail => "FAIL",
            Status::Error => "ERR ",
        };
        let score = o
            .dssim
            .map(|d| format!("{d:.5}"))
            .unwrap_or_else(|| "-".into());
        println!("{label}  {:<40} dssim={score:<9} {}", o.name, o.note);
        if o.status == Status::Fail || o.status == Status::Error {
            failed += 1;
        }
    }

    let report = out_dir.join("report.html");
    std::fs::write(&report, html_report(&outcomes, &golden_dir, &out_dir)).ok();
    println!(
        "\n{} fixtures, {failed} failing — report: {}",
        outcomes.len(),
        report.display()
    );

    if failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_one(
    name: &str,
    ir_dir: &Path,
    golden_dir: &Path,
    out_dir: &Path,
    waivers: &Waivers,
) -> Outcome {
    let waiver = waivers.fixture.get(name);
    let threshold = waiver
        .and_then(|w| w.threshold)
        .unwrap_or(DEFAULT_THRESHOLD);
    // Only the first line goes in the console listing; the report shows it all.
    let note = waiver
        .map(|w| w.reason.trim().lines().next().unwrap_or("").to_string())
        .unwrap_or_default();

    if waiver.map(|w| w.skip).unwrap_or(false) {
        return Outcome {
            name: name.into(),
            dssim: None,
            threshold,
            status: Status::Skipped,
            note,
        };
    }

    let ir_path = ir_dir.join(format!("{name}.json"));
    let actual_path = out_dir.join(format!("{name}.png"));

    let result = (|| -> sone_core::Result<()> {
        let json = std::fs::read_to_string(&ir_path)?;
        let doc = Document::from_json(&json)?;
        let base = ir_dir.to_path_buf();
        let engine = Engine::new();
        engine.load_document_fonts(&doc, &base)?;
        let prepared = engine.prepare(&doc, &base)?;
        let options = RenderOptions {
            format: OutputFormat::Png,
            density: doc.config.density.unwrap_or(2.0),
            ..Default::default()
        };
        let bytes = engine.encode(&prepared, &options)?;
        std::fs::write(&actual_path, bytes)?;
        Ok(())
    })();

    if let Err(e) = result {
        return Outcome {
            name: name.into(),
            dssim: None,
            threshold,
            status: Status::Error,
            note: format!("{e}"),
        };
    }

    match compare(&golden_dir.join(format!("{name}.jpg")), &actual_path) {
        Ok(score) => {
            let status = if score <= threshold {
                Status::Pass
            } else if waiver.is_some() {
                Status::Waived
            } else {
                Status::Fail
            };
            Outcome {
                name: name.into(),
                dssim: Some(score),
                threshold,
                status,
                note,
            }
        }
        Err(message) => {
            // A documented divergence can change the canvas size, so a waiver
            // covers a size mismatch too.
            let status = if waiver.is_some() {
                Status::Waived
            } else {
                Status::Error
            };
            Outcome {
                name: name.into(),
                dssim: None,
                threshold,
                status,
                note: format!("{note} [{message}]"),
            }
        }
    }
}

fn compare(expected: &Path, actual: &Path) -> Result<f64, String> {
    let a = load_rgb(expected)?;
    let b = load_rgb(actual)?;
    if a.0 != b.0 || a.1 != b.1 {
        return Err(format!(
            "size differs: golden {}x{}, actual {}x{}",
            a.0, a.1, b.0, b.1
        ));
    }
    let attr = dssim_core::Dssim::new();
    let img_a = attr
        .create_image_rgb(&a.2, a.0, a.1)
        .ok_or("could not build the golden image")?;
    let img_b = attr
        .create_image_rgb(&b.2, b.0, b.1)
        .ok_or("could not build the render image")?;
    let (score, _) = attr.compare(&img_a, &img_b);
    Ok(score.into())
}

fn load_rgb(path: &Path) -> Result<(usize, usize, Vec<rgb::RGB8>), String> {
    let img = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgb8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let pixels = img
        .pixels()
        .map(|p| rgb::RGB8::new(p[0], p[1], p[2]))
        .collect();
    Ok((w, h, pixels))
}

fn html_report(outcomes: &[Outcome], golden_dir: &Path, out_dir: &Path) -> String {
    let mut rows = String::new();
    for o in outcomes {
        let score = o
            .dssim
            .map(|d| format!("{d:.5}"))
            .unwrap_or_else(|| "—".into());
        let status = match o.status {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Waived => "waived",
            Status::Skipped => "skipped",
            Status::Error => "error",
        };
        rows.push_str(&format!(
            r#"<tr class="{status}"><td>{name}</td><td>{status}</td><td>{score}</td><td>{threshold}</td><td>{note}</td></tr>
<tr class="imgs"><td colspan="5"><img src="{golden}/{name}.jpg"><img src="{out}/{name}.png"></td></tr>"#,
            name = o.name,
            threshold = o.threshold,
            note = html_escape(&o.note),
            golden = golden_dir.display(),
            out = out_dir.display(),
        ));
    }
    format!(
        r#"<!doctype html><meta charset="utf-8"><title>sone goldens</title>
<style>
body{{font:14px/1.5 system-ui;margin:24px;background:#0b0e14;color:#e6e6e6}}
table{{border-collapse:collapse;width:100%}}
td{{padding:6px 10px;border-bottom:1px solid #222}}
tr.fail td,tr.error td{{color:#ff8080}} tr.pass td{{color:#7ee787}} tr.waived td{{color:#e3b341}}
tr.imgs img{{max-width:48%;margin-right:1%;border:1px solid #333;vertical-align:top}}
</style>
<h1>sone goldens</h1><table>{rows}</table>"#
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
