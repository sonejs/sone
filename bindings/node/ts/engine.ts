/**
 * The engine handle: a font registry plus an asset cache.
 *
 * Required of every sone binding — see `docs/bindings.md` — along with a
 * process-wide default so a script does not have to make one. Skia's font
 * collection is shared inside an engine, so one engine drives one thread; make
 * a second `Engine` for a second worker.
 *
 * Two things are asynchronous here that are synchronous in the Python binding:
 * loading the backend (the WebAssembly build has to instantiate) and reading
 * font bytes (which may be a `fetch`). Both are awaited on the paths that
 * already return promises, so nothing new leaks into the API.
 */
import type { Backend, NativeEngine } from "./backend.ts";
import { backendSync, isSynchronous, loadBackend } from "./binding.ts";
import { loadFontBytes } from "./bytes.ts";
import type { FontSource } from "./config.ts";
import { rethrow, rethrowSync } from "./errors.ts";

export class Engine {
  readonly baseDir: string | undefined;
  #backend: Backend | null = null;
  #native: NativeEngine | null = null;
  #pending: Promise<NativeEngine> | null = null;

  /** `baseDir` is the directory relative image paths resolve against. */
  constructor(baseDir?: string) {
    this.baseDir = baseDir;
    // Native backends load synchronously, which keeps `hasFont()` and
    // `fontFamilies()` synchronous from the very first call.
    if (isSynchronous) this.#attach(backendSync());
  }

  #attach(backend: Backend): NativeEngine {
    this.#backend = backend;
    this.#native ??= new backend.Engine(this.baseDir);
    return this.#native;
  }

  /** Resolve the backend and the engine behind it. */
  ready(): Promise<NativeEngine> {
    if (this.#native != null) return Promise.resolve(this.#native);
    this.#pending ??= loadBackend().then((backend) => this.#attach(backend));
    return this.#pending;
  }

  /**
   * The engine, resolving the backend if that can be done synchronously.
   * Under WebAssembly it cannot, and `backendSync` throws saying so.
   */
  get native(): NativeEngine {
    return this.#native ?? this.#attach(backendSync());
  }

  /** True when the engine can read image paths itself — false under WebAssembly. */
  get hasFilesystem(): boolean {
    return this.#backend?.hasFilesystem ?? isSynchronous;
  }

  /** Register a font family from a path, URL, or raw bytes. */
  async registerFont(name: string, source: FontSource): Promise<void> {
    const native = await this.ready();
    const parts = await loadFontBytes(source);
    rethrowSync(() => {
      for (const bytes of parts) native.registerFont(name, bytes);
    });
  }

  /** Register a font family from a file, reading it inside the engine. */
  async registerFontFile(name: string, path: string): Promise<void> {
    const native = await this.ready();
    rethrowSync(() => native.registerFontFile(name, path));
  }

  /** Drop one family and the shaping caches that depend on it. */
  unregisterFont(name: string): void {
    this.native.unregisterFont(name);
  }

  /** Make bytes available to documents as `asset:<name>`. */
  async registerImage(name: string, data: Uint8Array): Promise<void> {
    (await this.ready()).registerImage(name, data);
  }

  hasFont(name: string): boolean {
    return this.native.hasFont(name);
  }

  fontFamilies(): string[] {
    return this.native.fontFamilies();
  }

  resetFonts(): void {
    this.native.resetFonts();
  }

  /** Render a serialized IR document to bytes. */
  render(
    document: string,
    format: string,
    density?: number,
    quality?: number,
    strict?: boolean,
  ): Promise<Uint8Array> {
    return rethrow(async () =>
      (await this.ready()).render(document, format, density, quality, strict),
    );
  }

  /** One raster image per page. Requires `pageHeight` in the document config. */
  renderPages(
    document: string,
    format: string,
    density?: number,
    quality?: number,
    strict?: boolean,
  ): Promise<Uint8Array[]> {
    return rethrow(async () =>
      (await this.ready()).renderPages(document, format, density, quality, strict),
    );
  }

  dumpLayout(document: string): Promise<string> {
    return rethrow(async () => (await this.ready()).dumpLayout(document));
  }

  dumpMetadata(document: string, granularity = "node"): Promise<string> {
    return rethrow(async () =>
      (await this.ready()).dumpMetadata(document, granularity),
    );
  }

  /** The engine version — the Rust crates', not the npm package's. */
  version(): string {
    return (this.#backend ?? backendSync()).version();
  }
}

let fallback: Engine | null = null;

/** The process-wide engine used when no explicit one is passed. */
export function defaultEngine(): Engine {
  fallback ??= new Engine();
  return fallback;
}
