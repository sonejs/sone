//! Compiles `tests/c/smoke.c` against the built static library and checks that
//! the C ABI produces byte-identical output to the CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_root() -> PathBuf {
    manifest_dir().join("../..").canonicalize().unwrap()
}

/// `target/debug` (or `release`) for the current test binary.
fn target_dir() -> PathBuf {
    let mut dir = std::env::current_exe().unwrap();
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    dir
}

fn cc() -> String {
    std::env::var("CC").unwrap_or_else(|_| "cc".into())
}

fn have_cc() -> bool {
    Command::new(cc())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn c_abi_matches_the_cli_byte_for_byte() {
    if !have_cc() {
        eprintln!("no C compiler on PATH; skipping");
        return;
    }

    let root = repo_root();
    let fixture = root.join("fixtures/visual/ir/corners-1.json");
    if !fixture.exists() {
        eprintln!("no IR corpus; run `tools/sync-fixtures.sh <path-to-sone-checkout>`");
        return;
    }

    // Build the staticlib the C program links against.
    let build = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["build", "-p", "sone-ffi"])
        .current_dir(&root)
        .status()
        .expect("cargo build");
    assert!(build.success());

    let lib_dir = target_dir();
    let staticlib = lib_dir.join("libsone.a");
    assert!(staticlib.exists(), "expected {}", staticlib.display());

    let out_dir = lib_dir.join("c-abi");
    std::fs::create_dir_all(&out_dir).unwrap();
    let binary = out_dir.join("smoke");

    let mut compile = Command::new(cc());
    compile
        .arg("-std=c99")
        .arg("-Wall")
        .arg("-Wextra")
        .arg("-I")
        .arg(root.join("include"))
        .arg(manifest_dir().join("tests/c/smoke.c"))
        .arg(&staticlib)
        .arg("-o")
        .arg(&binary);
    for flag in platform_link_flags() {
        compile.arg(flag);
    }
    let status = compile.status().expect("cc");
    assert!(status.success(), "compiling the C smoke test failed");

    let c_output = out_dir.join("from-c.png");
    let run = Command::new(&binary)
        .arg(&fixture)
        .arg(root.join("fixtures/visual/ir"))
        .arg(&c_output)
        .output()
        .expect("run smoke");
    assert!(
        run.status.success(),
        "C smoke test failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let cli_output = out_dir.join("from-cli.png");
    let cli = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["run", "-q", "-p", "sone-cli", "--", "render"])
        .arg(&fixture)
        .args(["--density", "2", "-o"])
        .arg(&cli_output)
        .current_dir(&root)
        .status()
        .expect("cargo run sone-cli");
    assert!(cli.success());

    let a = std::fs::read(&c_output).unwrap();
    let b = std::fs::read(&cli_output).unwrap();
    assert_eq!(a.len(), b.len(), "C ABI and CLI output differ in length");
    assert!(a == b, "C ABI and CLI output differ");
}

fn platform_link_flags() -> Vec<String> {
    let mut flags: Vec<String> = Vec::new();
    if cfg!(target_os = "macos") {
        for framework in [
            "CoreFoundation",
            "CoreGraphics",
            "CoreText",
            "CoreServices",
            "ApplicationServices",
        ] {
            flags.push("-framework".into());
            flags.push(framework.into());
        }
        flags.push("-lc++".into());
    } else if cfg!(target_os = "linux") {
        flags.push("-lstdc++".into());
        flags.push("-lm".into());
        flags.push("-lpthread".into());
        flags.push("-ldl".into());
        flags.push("-lfontconfig".into());
    }
    flags
}

/// The header must stay in sync with the ABI.
#[test]
fn header_declares_the_public_surface() {
    let header = std::fs::read_to_string(repo_root().join("include/sone.h")).unwrap();
    for symbol in [
        "sone_engine_new",
        "sone_engine_free",
        "sone_engine_last_error",
        "sone_register_font",
        "sone_register_image",
        "sone_render_json",
        "sone_buffer_free",
        "sone_version",
        "SoneRenderOptions",
        "SoneBuffer",
    ] {
        assert!(header.contains(symbol), "sone.h is missing {symbol}");
    }
    assert!(Path::new(&repo_root().join("include/sone.h")).exists());
}
