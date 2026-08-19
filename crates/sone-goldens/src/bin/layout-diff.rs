//! Numeric layout-tree diff against the TypeScript engine's dump.

use std::path::PathBuf;

use serde_json::Value;
use sone_core::ir::Document;
use sone_skia::render::Engine;

const TOLERANCE: f64 = 0.5;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn main() -> std::process::ExitCode {
    let root = repo_root();
    let ir_dir = root.join("fixtures/visual/ir");
    let ts_dir = root.join("fixtures/visual/layout");
    let filter = std::env::args().nth(1);

    let mut names: Vec<String> = std::fs::read_dir(&ts_dir)
        .expect("run `tools/sync-fixtures.sh <path-to-sone-checkout>` first")
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
        .collect();
    names.sort();

    let mut failing = 0;
    for name in &names {
        let ts: Value = match std::fs::read_to_string(ts_dir.join(format!("{name}.json"))) {
            Ok(s) => serde_json::from_str(&s).unwrap(),
            Err(e) => {
                println!("ERR  {name}: {e}");
                failing += 1;
                continue;
            }
        };

        let json = std::fs::read_to_string(ir_dir.join(format!("{name}.json"))).unwrap();
        let doc = Document::from_json(&json).unwrap();
        let engine = Engine::new();
        if let Err(e) = engine.load_document_fonts(&doc, &ir_dir) {
            println!("ERR  {name}: {e}");
            failing += 1;
            continue;
        }
        let prepared = match engine.prepare(&doc, &ir_dir) {
            Ok(p) => p,
            Err(e) => {
                println!("ERR  {name}: {e}");
                failing += 1;
                continue;
            }
        };
        let ours = sone_core::dump::layout_json(&prepared.root, &prepared.layout);

        let mut diffs = Vec::new();
        compare(&ts, &ours, "root", &mut diffs);
        if diffs.is_empty() {
            println!("ok   {name}");
        } else {
            failing += 1;
            println!("DIFF {name}  ({} nodes differ)", diffs.len());
            for d in diffs.iter().take(6) {
                println!("       {d}");
            }
        }
    }

    println!(
        "\n{} fixtures, {failing} differing (tolerance {TOLERANCE}px)",
        names.len()
    );
    if failing > 0 {
        std::process::ExitCode::from(1)
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)
}

fn compare(ts: &Value, ours: &Value, path: &str, diffs: &mut Vec<String>) {
    let ty = ts.get("type").and_then(|v| v.as_str()).unwrap_or("?");
    for key in ["x", "y", "width", "height"] {
        let a = num(ts, key);
        let b = num(ours, key);
        if (a - b).abs() > TOLERANCE {
            diffs.push(format!("{path} ({ty}) {key}: ts={a} rust={b}"));
        }
    }

    // The TS dump walks the Yoga tree, which has no children under a grid
    // (they live in a side cache), so grid subtrees are compared by box only.
    if ty == "grid" {
        return;
    }

    let empty = Vec::new();
    let ts_children = ts
        .get("children")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    let our_children = ours
        .get("children")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);
    if ts_children.len() != our_children.len() {
        diffs.push(format!(
            "{path} ({ty}) child count: ts={} rust={}",
            ts_children.len(),
            our_children.len()
        ));
        return;
    }
    for (i, (a, b)) in ts_children.iter().zip(our_children.iter()).enumerate() {
        compare(a, b, &format!("{path}/{i}"), diffs);
    }
}
