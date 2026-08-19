/**
 * The WebAssembly engine, wrapped in the interface the native addon exposes.
 *
 * `@sonejs/sone` consumes this through the same `Backend` contract it uses for
 * the Node-API addon (`bindings/node/ts/backend.ts`), so nothing above the
 * loader knows which one it got. The only visible difference is that this
 * module has to be instantiated first, which is what `load()` is for.
 *
 * @example
 * import { load } from "@sonejs/sone-wasm";
 * const { Engine } = await load();
 * const engine = new Engine();
 */

/** The subset of the emscripten module this wrapper drives. */
interface WasmExports {
  HEAPU8: Uint8Array;
  _sone_wasm_alloc(len: number): number;
  _sone_wasm_dealloc(ptr: number, len: number): void;
  _sone_wasm_engine_new(): number;
  _sone_wasm_engine_free(engine: number): void;
  _sone_wasm_last_error(engine: number): number;
  _sone_wasm_buffer_ptr(buffer: number): number;
  _sone_wasm_buffer_len(buffer: number): number;
  _sone_wasm_buffer_free(buffer: number): void;
  _sone_wasm_pages_len(pages: number): number;
  _sone_wasm_pages_item(pages: number, index: number): number;
  _sone_wasm_pages_free(pages: number): void;
  _sone_wasm_register_font(e: number, n: number, nl: number, d: number, dl: number): number;
  _sone_wasm_unregister_font(e: number, n: number, nl: number): number;
  _sone_wasm_register_image(e: number, n: number, nl: number, d: number, dl: number): number;
  _sone_wasm_has_font(e: number, n: number, nl: number): number;
  _sone_wasm_font_families(e: number): number;
  _sone_wasm_reset_fonts(e: number): void;
  _sone_wasm_render(
    e: number, d: number, dl: number, f: number, fl: number,
    density: number, quality: number, strict: number,
  ): number;
  _sone_wasm_render_pages(
    e: number, d: number, dl: number, f: number, fl: number,
    density: number, quality: number, strict: number,
  ): number;
  _sone_wasm_dump(e: number, d: number, dl: number, g: number, gl: number): number;
  _sone_wasm_version(): number;
}

export interface LoadOptions {
  /**
   * Where to fetch `sone.wasm` from. Defaults to the file sitting next to this
   * module, which is what a bundler that emits assets will produce; set it when
   * serving the binary from a CDN or a path the bundler does not rewrite.
   */
  wasmUrl?: string | URL;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/** A pointer plus the length it was allocated with, so it can be released. */
interface Alloc {
  ptr: number;
  len: number;
}

/**
 * Errors carry the engine's own message, tagged with the failure class the
 * `@sonejs/sone` error types are built from.
 */
function fail(wasm: WasmExports, engine: number, fallback: string): never {
  const buffer = wasm._sone_wasm_last_error(engine);
  const message = buffer === 0 ? fallback : readString(wasm, buffer);
  if (buffer !== 0) wasm._sone_wasm_buffer_free(buffer);
  // The class prefix is the same contract the Node-API addon uses; the engine
  // does not report it separately, so infer it the way the message reads.
  const cls = /IR parse error|unsupported IR version|unknown .*format/i.test(message)
    ? "ir"
    : /asset|font|image/i.test(message)
      ? "asset"
      : "render";
  throw new Error(`sone:${cls}: ${message}`);
}

function readBytes(wasm: WasmExports, buffer: number): Uint8Array {
  const ptr = wasm._sone_wasm_buffer_ptr(buffer);
  const len = wasm._sone_wasm_buffer_len(buffer);
  // Copied, not a view: the heap may be detached by a later allocation.
  return wasm.HEAPU8.slice(ptr, ptr + len);
}

function readString(wasm: WasmExports, buffer: number): string {
  return decoder.decode(readBytes(wasm, buffer));
}

function take(wasm: WasmExports, buffer: number): Uint8Array {
  const bytes = readBytes(wasm, buffer);
  wasm._sone_wasm_buffer_free(buffer);
  return bytes;
}

/** Copy bytes into the module's memory. Zero-length needs no allocation. */
function put(wasm: WasmExports, data: Uint8Array): Alloc {
  if (data.length === 0) return { ptr: 0, len: 0 };
  const ptr = wasm._sone_wasm_alloc(data.length);
  if (ptr === 0) throw new Error("sone:render: out of WebAssembly memory");
  wasm.HEAPU8.set(data, ptr);
  return { ptr, len: data.length };
}

function putString(wasm: WasmExports, text: string): Alloc {
  return put(wasm, encoder.encode(text));
}

function release(wasm: WasmExports, ...allocs: Alloc[]): void {
  for (const { ptr, len } of allocs) wasm._sone_wasm_dealloc(ptr, len);
}

/**
 * Owns the font registry and the decoded-image cache.
 *
 * The methods are `async` only to match the native addon's signatures — the
 * WebAssembly engine is synchronous, so a render blocks whichever thread it is
 * on. Run it in a Web Worker if that thread is the one painting.
 */
class WasmSoneEngine {
  #wasm: WasmExports;
  #ptr: number;

  constructor(wasm: WasmExports) {
    this.#wasm = wasm;
    this.#ptr = wasm._sone_wasm_engine_new();
    if (this.#ptr === 0) throw new Error("sone:render: could not create an engine");
  }

  /** Release the engine. Nothing may be called on it afterwards. */
  destroy(): void {
    if (this.#ptr !== 0) {
      this.#wasm._sone_wasm_engine_free(this.#ptr);
      this.#ptr = 0;
    }
  }

  registerFont(name: string, data: Uint8Array): void {
    const wasm = this.#wasm;
    const n = putString(wasm, name);
    const d = put(wasm, data);
    try {
      const status = wasm._sone_wasm_register_font(this.#ptr, n.ptr, n.len, d.ptr, d.len);
      if (status !== 0) fail(wasm, this.#ptr, `could not register the font ${name}`);
    } finally {
      release(wasm, n, d);
    }
  }

  /** There is no filesystem here — read the file yourself and pass the bytes. */
  registerFontFile(name: string, path: string): void {
    throw new Error(
      `sone:asset: the WebAssembly engine has no filesystem, so it cannot read ${path}. ` +
        `Fetch the font and call Font.load(${JSON.stringify(name)}, bytes) instead.`,
    );
  }

  unregisterFont(name: string): void {
    const n = putString(this.#wasm, name);
    try {
      this.#wasm._sone_wasm_unregister_font(this.#ptr, n.ptr, n.len);
    } finally {
      release(this.#wasm, n);
    }
  }

  registerImage(name: string, data: Uint8Array): void {
    const wasm = this.#wasm;
    const n = putString(wasm, name);
    const d = put(wasm, data);
    try {
      const status = wasm._sone_wasm_register_image(this.#ptr, n.ptr, n.len, d.ptr, d.len);
      if (status !== 0) fail(wasm, this.#ptr, `could not register the asset ${name}`);
    } finally {
      release(wasm, n, d);
    }
  }

  hasFont(name: string): boolean {
    const n = putString(this.#wasm, name);
    try {
      return this.#wasm._sone_wasm_has_font(this.#ptr, n.ptr, n.len) !== 0;
    } finally {
      release(this.#wasm, n);
    }
  }

  fontFamilies(): string[] {
    const buffer = this.#wasm._sone_wasm_font_families(this.#ptr);
    if (buffer === 0) return [];
    return JSON.parse(decoder.decode(take(this.#wasm, buffer))) as string[];
  }

  resetFonts(): void {
    this.#wasm._sone_wasm_reset_fonts(this.#ptr);
  }

  async render(
    document: string,
    format: string,
    density?: number | null,
    quality?: number | null,
    strict?: boolean | null,
  ): Promise<Uint8Array> {
    const wasm = this.#wasm;
    const d = putString(wasm, document);
    const f = putString(wasm, format);
    try {
      const buffer = wasm._sone_wasm_render(
        this.#ptr, d.ptr, d.len, f.ptr, f.len,
        density ?? 0, quality ?? 1, strict ? 1 : 0,
      );
      if (buffer === 0) fail(wasm, this.#ptr, "render failed");
      return take(wasm, buffer);
    } finally {
      release(wasm, d, f);
    }
  }

  async renderPages(
    document: string,
    format: string,
    density?: number | null,
    quality?: number | null,
    strict?: boolean | null,
  ): Promise<Uint8Array[]> {
    const wasm = this.#wasm;
    const d = putString(wasm, document);
    const f = putString(wasm, format);
    try {
      const list = wasm._sone_wasm_render_pages(
        this.#ptr, d.ptr, d.len, f.ptr, f.len,
        density ?? 0, quality ?? 1, strict ? 1 : 0,
      );
      if (list === 0) fail(wasm, this.#ptr, "render failed");
      const pages: Uint8Array[] = [];
      for (let i = 0; i < wasm._sone_wasm_pages_len(list); i++) {
        // Borrowed from the list, so read it but do not free it individually.
        pages.push(readBytes(wasm, wasm._sone_wasm_pages_item(list, i)));
      }
      wasm._sone_wasm_pages_free(list);
      return pages;
    } finally {
      release(wasm, d, f);
    }
  }

  dumpLayout(document: string): Promise<string> {
    return this.#dump(document, "");
  }

  dumpMetadata(document: string, granularity?: string | null): Promise<string> {
    return this.#dump(document, granularity ?? "node");
  }

  /** An empty `granularity` means the layout tree rather than the metadata. */
  async #dump(document: string, granularity: string): Promise<string> {
    const wasm = this.#wasm;
    const d = putString(wasm, document);
    const g = putString(wasm, granularity);
    try {
      const buffer = wasm._sone_wasm_dump(this.#ptr, d.ptr, d.len, g.ptr, g.len);
      if (buffer === 0) fail(wasm, this.#ptr, "layout failed");
      return decoder.decode(take(wasm, buffer));
    } finally {
      release(wasm, d, g);
    }
  }
}

export type SoneWasmEngine = WasmSoneEngine;

export interface SoneWasm {
  Engine: new () => WasmSoneEngine;
  version(): string;
}

let pending: Promise<SoneWasm> | null = null;

/**
 * Instantiate the engine. Repeated calls share one module — it is ~20 MB, and
 * instantiating twice would double that for no benefit. Engines created from it
 * are still independent, each with its own fonts and asset cache.
 */
export function load(options: LoadOptions = {}): Promise<SoneWasm> {
  pending ??= instantiate(options);
  return pending;
}

async function instantiate(options: LoadOptions): Promise<SoneWasm> {
  // @ts-expect-error — emitted by emscripten at build time, so it has no types.
  const { default: createSoneEngine } = await import("./sone.js");
  const wasm = (await createSoneEngine(
    options.wasmUrl == null
      ? {}
      : { locateFile: () => String(options.wasmUrl) },
  )) as WasmExports;

  return {
    Engine: class extends WasmSoneEngine {
      constructor() {
        super(wasm);
      }
    },
    version: () => decoder.decode(take(wasm, wasm._sone_wasm_version())),
  };
}
