use std::path::PathBuf;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/visual/ir")
}

#[test]
fn every_committed_ir_document_parses_strictly() {
    let dir = corpus_dir();
    if !dir.exists() {
        eprintln!(
            "no IR corpus at {}; run `tools/sync-fixtures.sh <path-to-sone-checkout>`",
            dir.display()
        );
        return;
    }
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let json = std::fs::read_to_string(&path).unwrap();
        let doc = sone_core::ir::Document::from_json_strict(&json)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(doc.sone, sone_core::ir::IR_VERSION);
        count += 1;
    }
    assert!(count > 40, "expected the full fixture corpus, got {count}");
}
