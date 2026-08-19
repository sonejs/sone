/**
 * sone — a declarative canvas layout engine with rich international text,
 * rendered by the Rust engine.
 *
 * The authoring API is the sone v2 one, method for method: `core.ts` and
 * `ir.ts` are that package's own files, vendored unchanged. What differs is
 * underneath — instead of driving CanvasKit from JavaScript, the node tree is
 * serialized to an IR document and handed to Rust, natively on Node, Bun and
 * Deno and as WebAssembly in the browser.
 *
 * @example
 * import { sone, Font, Column, Row, Text } from "@sonejs/sone";
 *
 * await Font.load("Inter", "fonts/Inter-Regular.ttf");
 *
 * const root = Column(
 *   Text("Hello").size(28).weight("bold"),
 *   Row(Column().bg("salmon").size(50).rounded(14)).gap(10),
 * ).gap(20).padding(20).bg("khaki").cornerRadius(28);
 *
 * await sone(root).save("card.png", { density: 2 });
 */
import { resolveAssets } from "./assets.ts";
import { wrap } from "./bytes.ts";
import type { FontSource, SoneRenderConfig } from "./config.ts";
import type { SoneNode } from "./core.ts";
import { defaultEngine, Engine } from "./engine.ts";
import { type IrDocument, toIR } from "./ir.ts";

export * from "./core.ts";
export type {
  FontSource,
  SoneHeaderFooter,
  SonePageInfo,
  SoneRenderConfig,
} from "./config.ts";
export { Engine, defaultEngine } from "./engine.ts";
export {
  AssetError,
  IrError,
  RenderError,
  SoneError,
} from "./errors.ts";
export { IR_VERSION, toIR, toIrNode } from "./ir.ts";
export type { IrDocument, IrFont, IrNode } from "./ir.ts";

/** Raster output formats. `pdf` and `svg` are vector and ignore `density`. */
export type OutputFormat = "png" | "jpg" | "jpeg" | "webp" | "raw" | "pdf" | "svg";

export interface ExportOptions {
  /** 0..1, honoured by jpg and webp. */
  quality?: number;
  /** Pixel density multiplier for raster output. Ignored by pdf/svg. */
  density?: number;
}

const FORMAT_BY_SUFFIX: Record<string, OutputFormat> = {
  ".png": "png",
  ".jpg": "jpg",
  ".jpeg": "jpg",
  ".webp": "webp",
  ".pdf": "pdf",
  ".svg": "svg",
  ".raw": "raw",
  ".rgba": "raw",
};

async function writeFile(path: string, data: Uint8Array): Promise<void> {
  const { writeFile: write } = await import(/* @vite-ignore */ "node:fs/promises");
  await write(path, data);
}

/** A node plus render config, with one method per output format. */
class Rendered {
  readonly #node: SoneNode;
  readonly #config: SoneRenderConfig | undefined;
  readonly #engine: Engine | undefined;

  constructor(node: SoneNode, config?: SoneRenderConfig, engine?: Engine) {
    this.#node = node;
    this.#config = config;
    this.#engine = engine;
  }

  /** The engine this render will run on. */
  get engine(): Engine {
    return this.#engine ?? defaultEngine();
  }

  // ── the document ─────────────────────────────────────────────────────────

  /** The IR document, ready to hand to any sone engine. */
  document(): IrDocument {
    return toIR(this.#node, this.#config);
  }

  /** The IR document as JSON. */
  json(indent?: number): string {
    return JSON.stringify(this.document(), null, indent);
  }

  /**
   * Serialize, then fetch and register whatever the engine will not load
   * itself — remote images always, and every image under WebAssembly.
   */
  async #prepare(): Promise<string> {
    const engine = this.engine;
    const native = await engine.ready();
    const document = this.document();
    await resolveAssets(document, native, engine.hasFilesystem);
    return JSON.stringify(document);
  }

  async #encode(
    format: OutputFormat,
    options?: ExportOptions,
  ): Promise<Uint8Array> {
    const document = await this.#prepare();
    return wrap(
      await this.engine.render(
        document,
        format,
        options?.density,
        options?.quality,
      ),
    );
  }

  // ── outputs ──────────────────────────────────────────────────────────────

  /** JPEG image. `quality` is 0..1 (default 1). */
  jpg(quality = 1.0, options?: ExportOptions): Promise<Uint8Array> {
    return this.#encode("jpg", { quality, ...options });
  }

  /** PNG image. */
  png(options?: ExportOptions): Promise<Uint8Array> {
    return this.#encode("png", options);
  }

  /** SVG vector graphic with live `<text>` elements. */
  svg(options?: ExportOptions): Promise<Uint8Array> {
    return this.#encode("svg", options);
  }

  /**
   * PDF document. With `pageHeight` set in config, one page per break; text
   * stays selectable with subsetted fonts.
   */
  pdf(options?: ExportOptions): Promise<Uint8Array> {
    return this.#encode("pdf", options);
  }

  /** WebP image. */
  webp(options?: ExportOptions): Promise<Uint8Array> {
    return this.#encode("webp", options);
  }

  /** Raw RGBA pixel buffer, row-major and unpremultiplied. */
  raw(options?: ExportOptions): Promise<Uint8Array> {
    return this.#encode("raw", options);
  }

  /** One raster image per page. Requires `pageHeight` in config. */
  async pages(
    format: OutputFormat = "png",
    options?: ExportOptions,
  ): Promise<Uint8Array[]> {
    const document = await this.#prepare();
    const pages = await this.engine.renderPages(
      document,
      format,
      options?.density,
      options?.quality,
    );
    return pages.map(wrap);
  }

  /** Render and write to `path`, inferring the format from its suffix. */
  async save(path: string, options?: ExportOptions): Promise<string> {
    const suffix = path.slice(path.lastIndexOf(".")).toLowerCase();
    const format = FORMAT_BY_SUFFIX[suffix];
    if (format == null) {
      throw new TypeError(`cannot infer an output format from ${path}`);
    }
    await writeFile(path, await this.#encode(format, options));
    return path;
  }

  /** Write `name-1.png`, `name-2.png`, … next to `path`. */
  async savePages(path: string, options?: ExportOptions): Promise<string[]> {
    const dot = path.lastIndexOf(".");
    const stem = dot === -1 ? path : path.slice(0, dot);
    const suffix = dot === -1 ? ".png" : path.slice(dot);
    const format = FORMAT_BY_SUFFIX[suffix.toLowerCase()] ?? "png";
    const pages = await this.pages(format, options);
    return Promise.all(
      pages.map(async (data, index) => {
        const name = `${stem}-${index + 1}${suffix}`;
        await writeFile(name, data);
        return name;
      }),
    );
  }

  // ── introspection ────────────────────────────────────────────────────────

  /** The computed layout tree. */
  async layout(): Promise<unknown> {
    return JSON.parse(await this.engine.dumpLayout(await this.#prepare()));
  }

  /** Dataset-style boxes: `"node"`, `"line"` or `"word"`. */
  async metadata(granularity: "node" | "line" | "word" = "node"): Promise<unknown> {
    return JSON.parse(
      await this.engine.dumpMetadata(await this.#prepare(), granularity),
    );
  }
}

export type { Rendered };

/**
 * Render a node, exposing one method per output format.
 *
 * On Node the byte-returning methods yield a `Buffer` at runtime (typed as
 * `Uint8Array`); other runtimes return a plain `Uint8Array`.
 */
export function sone(
  node: SoneNode,
  config?: SoneRenderConfig & { engine?: Engine },
): Rendered {
  const { engine, ...rest } = config ?? {};
  return new Rendered(node, rest, engine);
}

/**
 * Font management on the process-wide engine.
 *
 * Skia has no system fonts, so at least one family must be registered before
 * rendering any text. For isolation, or to render on several threads at once,
 * make an `Engine` per thread and pass it as `sone(node, { engine })`.
 */
export const Font = {
  /** Load and register a font from a path, URL, or raw bytes. */
  load: (name: string, source: FontSource): Promise<void> =>
    defaultEngine().registerFont(name, source),
  /** Unregister a previously loaded font. */
  unload: (name: string): void => defaultEngine().unregisterFont(name),
  /** Remove every registered font. */
  reset: (): void => defaultEngine().resetFonts(),
  /** Whether a font with this name is registered. */
  has: (name: string): boolean => defaultEngine().hasFont(name),
  /** Every registered family name. */
  families: (): string[] => defaultEngine().fontFamilies(),
};

/** The engine version — the Rust crates', not the npm package's. */
export const version = (): string => defaultEngine().version();
