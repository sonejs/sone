//! Regenerates `include/sone.h` from the C ABI.
//! Run with `cargo run -p sone-ffi --features generate-header --bin sone-header`.

fn main() {
    #[cfg(feature = "generate-header")]
    {
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out = crate_dir.join("../../include/sone.h");
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        cbindgen::generate(&crate_dir)
            .expect("cbindgen")
            .write_to_file(&out);
        println!("wrote {}", out.display());
    }
    #[cfg(not(feature = "generate-header"))]
    eprintln!("rebuild with --features generate-header");
}
