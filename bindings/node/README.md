# @sonejs/sone

A declarative canvas layout engine with rich international text, rendered by
Skia. The same fluent API as the [TypeScript package](https://github.com/seanghay/sone),
backed by the Rust engine — natively on Node, Bun and Deno, and as WebAssembly
in the browser.

```bash
npm install @sonejs/sone
```

```ts
import { Column, Font, Row, Text, sone } from "@sonejs/sone";

await Font.load("Inter", "fonts/Inter-Regular.ttf");

const root = Column(
  Text("Hello").size(28).weight("bold"),
  Row(
    Column().bg("lightgreen").size(50).rounded(14),
    Column().bg("salmon").height(50).rounded(14).flex(1),
    Column().bg("orange").size(50).rounded(14),
  ).gap(10),
)
  .gap(20)
  .padding(20)
  .size(420, 300)
  .bg("khaki")
  .cornerRadius(28)
  .borderColor("chocolate")
  .borderWidth(4)
  .rotate(20);

await sone(root).save("card.png", { density: 2 });
```

`ts/core.ts` and `ts/ir.ts` are the TypeScript engine's own files, vendored
unchanged apart from their imports — so the builder is not a reimplementation,
and a test asserts both copies still serialize to identical documents.

## Output formats

```ts
const page = sone(root, { width: 794, pageHeight: 1123, margin: 64 });

await page.png({ density: 2 }); // Uint8Array (a Buffer on Node)
await page.jpg(0.9);            //   quality is the first argument, as in sone v2
await page.webp();
await page.svg();               //   vector, with live <text>
await page.pdf();               //   one page per break, text selectable
await page.raw();               //   unpremultiplied RGBA
await page.pages();             // Uint8Array[], one raster per page

await page.save("out.pdf");     // format inferred from the suffix
await page.savePages("p.png");  // p-1.png, p-2.png, …

await page.layout();            // the computed layout tree
await page.metadata("line");    // dataset boxes: "node" | "line" | "word"
page.document();                // the IR document
```

Headers and footers may use `{pageNumber}` and `{totalPages}`, substituted per
page. The function form is called once with sentinel numbers, so it may *place*
them anywhere but cannot branch on them:

```ts
await sone(root, {
  pageHeight: 1123,
  header: Row(Text("Report")).padding(12),
  footer: ({ pageNumber, totalPages }) =>
    Row(Text(`${pageNumber} of ${totalPages}`)).padding(12),
}).save("report.pdf");
```

## Fonts and assets

Skia has no system fonts, so register at least one family before drawing text.
`Font` uses a process-wide engine; for isolation, or to render on several
threads at once, create an `Engine` per thread:

```ts
import { Engine, sone } from "@sonejs/sone";

const engine = new Engine("assets");          // relative image paths resolve here
await engine.registerFont("Moul", "fonts/Moul-Regular.ttf");
await engine.registerImage("logo", logoBytes); // referenced as Photo("asset:logo")

await sone(root, { engine }).png();
```

`Font.load` accepts a path, a `URL`, an `ArrayBuffer` or a `Uint8Array`.
`Photo` accepts a path, an `asset:<name>` handle, an `http(s)` URL, or raw bytes.

The engine itself never does network I/O during a render. Remote images are
fetched by this package *before* the document reaches the engine and registered
as assets — so `Photo("https://…")` works, but the fetch is visible in your
network log rather than hidden inside a render.

## Runtimes

| | how it loads | notes |
|---|---|---|
| Node ≥ 18 | prebuilt `.node` addon | rendering runs on the libuv threadpool, off the main thread |
| Bun | same addon | |
| Deno 2 | same addon | needs `--allow-ffi` and `--node-modules-dir=auto`; Deno does not run install scripts, and this package needs none |
| Browsers, workers, workerd | `@sonejs/sone-wasm` | resolved through the `browser` export condition; see that package for size and threading |

Everything above the loader is shared, so the API is identical on all of them.
The one behavioural difference is that the WebAssembly engine has no filesystem:
`Engine.registerFontFile` throws there, and image paths are fetched as URLs
relative to the document.

## Errors

`SoneError` is the base class; `IrError`, `AssetError` and `RenderError` are
thrown for document, asset and rendering failures respectively — the same split
`SoneError::exit_code()` makes in the engine.

## Differences from `sone` v2

The authoring API is the same call for call. What is not carried over is the
part of v2's surface that hands out CanvasKit objects, which have no equivalent
when the engine is Rust:

| v2 export | here |
|---|---|
| `SoneCanvas`, `SoneImage`, `SonePath` | not exported — the engine owns its own Skia objects |
| `sone(…).canvas()`, `.canvasWithMetadata()` | not exported; use `.layout()` / `.metadata()` |
| `sone(…).pages()` | returns `Uint8Array[]` rather than `SoneCanvas[]` |
| `configureSkia` | not needed; the wasm loader takes `wasmUrl` instead |
| `render`, `renderPages`, `renderWithMetadata` | not exported — they returned canvases. Go through `sone()` |
| `fontBuilder`, `qrcode`, `toYoloDataset`, `/shiki` | not yet ported; they are pure TypeScript over the same data |
| `.bg(parsedGradient)` | typed for source compatibility, but the IR only carries gradients as CSS strings, so pass the string |
| `SoneRenderConfig.cache` | dropped — `Engine` owns the decoded-image cache |

Added, matching the other bindings: `Engine`, `layout()`, `metadata()`,
`document()`, `json()`, `save()`, `savePages()`, `Font.families()`.

## Building from source

```bash
npm install                                       # from the repository root
npm run build --workspace @sonejs/sone-wasm       # needs emscripten
npm run build --workspace @sonejs/sone            # needs a Rust toolchain

npm test  --workspace @sonejs/sone                # vitest
npm run test:bun  --workspace @sonejs/sone
npm run test:deno --workspace @sonejs/sone
```

Without emscripten, `npm run build:ts --workspace @sonejs/sone-wasm` is enough to
work in this package: it emits the type declarations `ts/binding.browser.ts`
imports, which `npm run lint` needs and nothing else here does. The cross-engine
test skips itself while the real `dist/sone.wasm` is missing.

The suites cover the builder, every output format, the error classes, byte
equality with `sone-cli`, IR equality with the TypeScript engine (when a
checkout is next door, or `SONE_TS_REPO` points at one), and agreement between
the native and WebAssembly engines.

## Licence

Apache-2.0.
