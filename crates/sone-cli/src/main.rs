use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_core::Result;
use sone_skia::render::{base_dir_for, Engine, RenderOptions};

const USAGE: &str = "\
sone — render a sone IR document

USAGE:
    sone render <doc.json> -o <out.png|jpg|webp|pdf|svg|raw> [options]
    sone dump-layout <doc.json>
    sone dump-metadata <doc.json> [--granularity node|line|word]

OPTIONS:
    -o, --output <path>     output file ('-' writes to stdout)
    --density <n>           raster scale factor (default 1)
    --quality <q>           JPEG/WebP quality 0..1 (default 1)
    --format <f>            override the format inferred from the output path
    --backend <cpu|gpu>     rasterization backend (default cpu)
    --strict                reject unknown IR fields
    --debug-layout          draw layout bounding boxes
    --debug-text            draw text segment boxes
    --pages                 with pageHeight set, write one raster file per page
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let result = match args[0].as_str() {
        "render" => cmd_render(&args[1..]),
        "dump-layout" => cmd_dump_layout(&args[1..]),
        "dump-metadata" => cmd_dump_metadata(&args[1..]),
        other => {
            eprintln!("sone: unknown command {other:?}\n\n{USAGE}");
            return ExitCode::from(64);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sone: {e}");
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

struct Args {
    input: PathBuf,
    output: Option<String>,
    density: f32,
    quality: f32,
    format: Option<String>,
    gpu: bool,
    strict: bool,
    debug_layout: bool,
    debug_text: bool,
    pages: bool,
    granularity: String,
}

fn parse_args(argv: &[String]) -> std::result::Result<Args, String> {
    let mut out = Args {
        input: PathBuf::new(),
        output: None,
        density: 1.0,
        quality: 1.0,
        format: None,
        gpu: false,
        strict: false,
        debug_layout: false,
        debug_text: false,
        pages: false,
        granularity: "node".into(),
    };
    let mut positional = None;
    let mut i = 0;
    while i < argv.len() {
        let a = argv[i].as_str();
        let mut next = |name: &str| -> std::result::Result<String, String> {
            i += 1;
            argv.get(i)
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match a {
            "-o" | "--output" => out.output = Some(next("--output")?),
            "--density" => {
                out.density = next("--density")?
                    .parse()
                    .map_err(|_| "--density must be a number")?
            }
            "--quality" => {
                out.quality = next("--quality")?
                    .parse()
                    .map_err(|_| "--quality must be a number")?
            }
            "--format" => out.format = Some(next("--format")?),
            "--granularity" => out.granularity = next("--granularity")?,
            "--backend" => out.gpu = next("--backend")? == "gpu",
            "--strict" => out.strict = true,
            "--debug-layout" => out.debug_layout = true,
            "--debug-text" => out.debug_text = true,
            "--pages" => out.pages = true,
            _ if a.starts_with('-') => return Err(format!("unknown option {a:?}")),
            _ => positional = Some(PathBuf::from(a)),
        }
        i += 1;
    }
    out.input = positional.ok_or("no input document given")?;
    Ok(out)
}

fn load(args: &Args) -> Result<(Document, PathBuf)> {
    let json = std::fs::read_to_string(&args.input)?;
    let doc = if args.strict {
        Document::from_json_strict(&json)?
    } else {
        Document::from_json(&json)?
    };
    Ok((doc, base_dir_for(&args.input)))
}

fn cmd_render(argv: &[String]) -> Result<()> {
    let args = parse_args(argv).map_err(sone_core::SoneError::Render)?;
    let (doc, base) = load(&args)?;

    let output = args.output.clone().unwrap_or_else(|| "-".into());
    let format = resolve_format(args.format.as_deref(), &output)?;

    if args.gpu {
        eprintln!(
            "sone: the GPU backend is not built into this binary; falling back to CPU raster"
        );
    }

    let engine = Engine::new();
    engine.load_document_fonts(&doc, &base)?;

    let density = if args.density != 1.0 {
        args.density
    } else {
        doc.config.density.unwrap_or(1.0)
    };
    let options = RenderOptions {
        format,
        density,
        quality: args.quality,
        strict: args.strict,
        debug_layout: args.debug_layout,
        debug_text: args.debug_text,
    };

    // Paginated PDFs go through the multi-page path; everything else renders
    // the document as one surface.
    if format == OutputFormat::Pdf {
        let bytes = engine.render_pdf(&doc, &base, &options)?;
        return write_output(&output, &bytes);
    }

    if doc.config.page_height.is_some() && args.pages {
        let pages = engine.render_pages(&doc, &base, &options)?;
        return write_pages(&output, &pages);
    }

    let prepared = engine.prepare(&doc, &base)?;
    for warning in &prepared.warnings {
        eprintln!("sone: {warning}");
    }
    let bytes = engine.encode(&prepared, &options)?;
    write_output(&output, &bytes)
}

fn resolve_format(explicit: Option<&str>, output: &str) -> Result<OutputFormat> {
    if let Some(f) = explicit {
        return OutputFormat::from_extension(f)
            .ok_or_else(|| sone_core::SoneError::Render(format!("unknown format {f:?}")));
    }
    let ext = Path::new(output)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    OutputFormat::from_extension(ext).ok_or_else(|| {
        sone_core::SoneError::Render(format!("cannot infer a format from {output:?}"))
    })
}

fn write_output(path: &str, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    if path == "-" {
        std::io::stdout().write_all(bytes)?;
        return Ok(());
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

/// `out-1.png`, `out-2.png`, … next to the requested output path.
fn write_pages(path: &str, pages: &[Vec<u8>]) -> Result<()> {
    let p = Path::new(path);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("png");
    let dir = p.parent().unwrap_or(Path::new("."));
    for (i, bytes) in pages.iter().enumerate() {
        std::fs::write(dir.join(format!("{stem}-{}.{ext}", i + 1)), bytes)?;
    }
    println!("{} pages written", pages.len());
    Ok(())
}

fn cmd_dump_layout(argv: &[String]) -> Result<()> {
    let args = parse_args(argv).map_err(sone_core::SoneError::Render)?;
    let (doc, base) = load(&args)?;
    let engine = Engine::new();
    engine.load_document_fonts(&doc, &base)?;
    let prepared = engine.prepare(&doc, &base)?;
    let json = sone_core::dump::layout_json(&prepared.root, &prepared.layout);
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}

fn cmd_dump_metadata(argv: &[String]) -> Result<()> {
    let args = parse_args(argv).map_err(sone_core::SoneError::Render)?;
    let (doc, base) = load(&args)?;
    let engine = Engine::new();
    engine.load_document_fonts(&doc, &base)?;
    let prepared = engine.prepare(&doc, &base)?;
    let json = sone_core::metadata::build(
        &prepared.root,
        &prepared.layout,
        &prepared.state,
        &args.granularity,
    );
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}
