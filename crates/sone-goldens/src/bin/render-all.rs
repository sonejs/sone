//! Renders every committed IR document with the Rust engine.
//!
//! Output mirrors what the TypeScript visual scripts write: a JPEG at density 2
//! per canvas fixture, and a PDF for the paginated ones. Files land in a
//! separate directory so the committed goldens stay untouched.
//!
//! ```text
//! cargo run --release -p sone-goldens --bin render-all [out-dir] [name-filter]
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use rayon::prelude::*;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_skia::render::{Engine, RenderOptions};

/// The TypeScript visual scripts export at density 2.
const DENSITY: f32 = 2.0;

/// Output file name and its size in bytes.
type Rendered = (String, u64);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn main() -> ExitCode {
    let root = repo_root();
    let ir_dir = root.join("fixtures/visual/ir");

    let mut args = std::env::args().skip(1);
    let out_dir = args
        .next()
        .map(|a| root.join(a))
        .unwrap_or_else(|| root.join("target/renders"));
    let filter = args.next();

    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("cannot create {}: {e}", out_dir.display());
        return ExitCode::from(3);
    }

    let mut names: Vec<String> = match std::fs::read_dir(&ir_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
            .filter(|n| {
                filter
                    .as_ref()
                    .map(|f| n.contains(f.as_str()))
                    .unwrap_or(true)
            })
            .collect(),
        Err(e) => {
            eprintln!(
                "no IR corpus at {} ({e}); run `tools/sync-fixtures.sh <path-to-sone-checkout>`",
                ir_dir.display()
            );
            return ExitCode::from(3);
        }
    };
    names.sort();

    let started = Instant::now();
    let results: Vec<(String, Result<Rendered, String>)> = names
        .par_iter()
        .map(|name| (name.clone(), render_one(name, &ir_dir, &out_dir)))
        .collect();

    let mut failed = 0;
    for (name, result) in &results {
        match result {
            Ok((file, bytes)) => {
                println!("ok    {name:<40} {file}  {:.0} KB", *bytes as f64 / 1024.0)
            }
            Err(message) => {
                failed += 1;
                println!("FAIL  {name:<40} {message}");
            }
        }
    }

    let index = out_dir.join("index.html");
    let rendered: Vec<(&str, &str)> = results
        .iter()
        .filter_map(|(name, r)| {
            r.as_ref()
                .ok()
                .map(|(file, _)| (name.as_str(), file.as_str()))
        })
        .collect();
    if let Err(e) = std::fs::write(&index, index_html(&rendered, &root)) {
        eprintln!("could not write {}: {e}", index.display());
    }

    println!(
        "\n{} of {} rendered in {:.1}s → {}\n     side-by-side: {}",
        results.len() - failed,
        results.len(),
        started.elapsed().as_secs_f64(),
        out_dir.display(),
        index.display()
    );

    if failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// TypeScript golden on the left, Rust render on the right.
fn index_html(rendered: &[(&str, &str)], root: &Path) -> String {
    let goldens = root.join("fixtures/visual");
    let mut rows = String::new();
    for (name, file) in rendered {
        let cell = if file.ends_with(".pdf") {
            format!(
                r#"<a href="{golden}/{name}.pdf">{name}.pdf (TypeScript)</a> · <a href="{name}.pdf">{name}.pdf (Rust)</a>"#,
                golden = goldens.display(),
            )
        } else {
            format!(
                r#"<img src="{golden}/{name}.jpg" loading="lazy"><img src="{file}" loading="lazy">"#,
                golden = goldens.display(),
            )
        };
        rows.push_str(&format!(
            "<section><h2>{name}</h2><div>{cell}</div></section>\n"
        ));
    }
    format!(
        r#"<!doctype html><meta charset="utf-8"><title>sone — Rust renders</title>
<style>
body{{font:14px/1.5 system-ui;margin:24px;background:#0b0e14;color:#e6e6e6}}
h1{{font-size:20px}} h2{{font-size:14px;font-weight:600;color:#9aa4b2;margin:24px 0 6px}}
section div{{display:flex;gap:8px;align-items:flex-start}}
img{{max-width:calc(50% - 4px);border:1px solid #222;background:#fff}}
a{{color:#7aa2f7}}
p.legend{{color:#9aa4b2}}
</style>
<h1>sone — Rust renders</h1>
<p class="legend">Left: TypeScript golden. Right: Rust engine, same IR document, density 2.</p>
{rows}"#
    )
}

fn render_one(name: &str, ir_dir: &Path, out_dir: &Path) -> Result<Rendered, String> {
    let json =
        std::fs::read_to_string(ir_dir.join(format!("{name}.json"))).map_err(|e| e.to_string())?;
    let doc = Document::from_json(&json).map_err(|e| e.to_string())?;

    let engine = Engine::new();
    engine
        .load_document_fonts(&doc, ir_dir)
        .map_err(|e| e.to_string())?;

    // Paginated documents are PDFs in the TypeScript suite; everything else is
    // a single JPEG.
    let paginated = doc.config.page_height.is_some();
    let options = RenderOptions {
        format: if paginated {
            OutputFormat::Pdf
        } else {
            OutputFormat::Jpeg
        },
        density: doc.config.density.unwrap_or(DENSITY),
        quality: 1.0,
        ..Default::default()
    };

    let bytes = if paginated {
        engine
            .render_pdf(&doc, ir_dir, &options)
            .map_err(|e| e.to_string())?
    } else {
        let prepared = engine.prepare(&doc, ir_dir).map_err(|e| e.to_string())?;
        engine
            .encode(&prepared, &options)
            .map_err(|e| e.to_string())?
    };

    let file = format!("{name}.{}", if paginated { "pdf" } else { "jpg" });
    std::fs::write(out_dir.join(&file), &bytes).map_err(|e| e.to_string())?;
    Ok((file, bytes.len() as u64))
}
