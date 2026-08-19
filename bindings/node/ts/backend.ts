/**
 * The contract both engines implement.
 *
 * There are two: the Node-API addon built by `src/lib.rs`, and the emscripten
 * WebAssembly module in `@sonejs/sone-wasm`. They expose the same method names
 * and the same shapes — document in, bytes out — so nothing above this file
 * ever branches on which one is loaded.
 */

/** A live engine: owns a font registry and an asset cache. */
export interface NativeEngine {
  registerFont(name: string, data: Uint8Array): void;
  registerFontFile(name: string, path: string): void;
  unregisterFont(name: string): void;
  registerImage(name: string, data: Uint8Array): void;
  hasFont(name: string): boolean;
  fontFamilies(): string[];
  resetFonts(): void;
  render(
    document: string,
    format: string,
    density?: number | null,
    quality?: number | null,
    strict?: boolean | null,
  ): Promise<Uint8Array>;
  renderPages(
    document: string,
    format: string,
    density?: number | null,
    quality?: number | null,
    strict?: boolean | null,
  ): Promise<Uint8Array[]>;
  dumpLayout(document: string): Promise<string>;
  dumpMetadata(document: string, granularity?: string | null): Promise<string>;
}

export interface Backend {
  Engine: new (baseDir?: string) => NativeEngine;
  version(): string;
  /** Whether the engine can read files itself. False under WebAssembly. */
  hasFilesystem: boolean;
}
