# @sonejs/sone-wasm

The sone layout engine — Rust and Skia — compiled to WebAssembly.

This is the browser backend for [`@sonejs/sone`](https://www.npmjs.com/package/@sonejs/sone),
which resolves it automatically through its `browser` export condition. Install
that package; you rarely need this one directly.

It is engine only: bytes in, bytes out. The fluent builder lives in
`@sonejs/sone` and there is exactly one copy of it.

## Direct use

```ts
import { load } from "@sonejs/sone-wasm";

const { Engine, version } = await load();
const engine = new Engine();

engine.registerFont("Inter", new Uint8Array(await (await fetch("/Inter.ttf")).arrayBuffer()));

const png = await engine.render(
  JSON.stringify({ sone: 1, root: { type: "column", props: { width: 64, height: 64, background: ["red"] } } }),
  "png",
  2, // density
);

engine.destroy();
```

`load()` instantiates once and shares the module — it is ~6 MB, so a second
instantiation would double that for nothing. Engines created from it are still
independent, each with its own fonts and asset cache.

Pass `load({ wasmUrl })` when the binary is not sitting next to the loader —
serving it from a CDN, or through a bundler that does not rewrite the asset URL.

## What to expect

- **~6 MB** of `.wasm`, about 2.5 MB over the wire with Brotli. Most of it is
  Skia's ICU table, which `textlayout` links unconditionally — see the
  binary-size notes in [status.md](../../docs/status.md).
- **No filesystem.** Fonts and images must arrive as bytes;
  `registerFontFile` throws with a message saying so.
- **Synchronous rendering.** The methods return promises to match the native
  addon's signatures, but the work happens on the calling thread. Render inside
  a Web Worker if that thread is also painting.
- **No cross-origin isolation needed.** The module is single-threaded, so there
  is no `SharedArrayBuffer` and no COOP/COEP requirement.
- **Same pixels as the native engine**, exactly, for everything except glyphs:
  macOS rasterizes those through CoreText and this build through Skia's bundled
  FreeType, so coverage differs by a hair along the edges. Layout is identical
  field for field. `bindings/node/__test__/wasm.test.ts` asserts both halves.

## Building

```bash
./build.sh
```

Needs emscripten — set `EMSDK` to an emsdk checkout, or install
`brew install emscripten` and the script bridges the layout difference itself.
Roughly two minutes on a laptop, cached in `target/` afterwards.

Skia is compiled from source rather than downloaded, for two independent
reasons, both explained in `build.sh`:

1. The published `wasm32-unknown-emscripten` prebuilt was linked with
   emscripten's legacy JavaScript exception handling. Current Rust emits
   `-fwasm-exceptions` for this target and the flag that used to let it opt out
   has been removed, so the two object formats cannot be linked together.
2. `skia-bindings` 0.99 cannot consume its own emscripten prebuilt anyway — it
   renames `lib*.wasm.a` to `lib*.a` unconditionally, and the archive already
   ships them as `.a`.

## Licence

Apache-2.0.
