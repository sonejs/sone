/**
 * IR dump — the serialization boundary shared with the Rust engine.
 *
 * Produces a plain JSON document (schema `sone: 1`) from a node tree plus
 * render config, so the same document can be rendered by either engine and
 * diffed. This is a pure serializer: it never renders and never loads assets.
 */
import type {
  ListNode,
  PhotoNode,
  SoneNode,
  SpanNode,
  TextNode,
} from "./core.ts";
import type {
  SoneHeaderFooter,
  SonePageInfo,
  SoneRenderConfig,
} from "./config.ts";

export const IR_VERSION = 1;

export interface IrNode {
  type: string;
  props?: Record<string, unknown>;
  children?: IrNode[];
  inline?: Array<string | IrNode>;
}

export interface IrFont {
  name: string;
  src: string;
}

export interface IrDocument {
  sone: number;
  config: Record<string, unknown>;
  fonts: IrFont[];
  root: IrNode;
}

export interface ToIrOptions {
  /** Directory the IR file will live in; asset paths are made relative to it. */
  baseDir?: string;
  /** Fonts to record in the document. */
  fonts?: IrFont[];
  /** Render density recorded in config. */
  density?: number;
  /** Rewrites an asset src; defaults to passing the value through. */
  resolveSrc?: (src: string) => string;
}

const PAGE_TOKEN = 987654321;
const TOTAL_TOKEN = 987654322;

/** Props that exist only inside the running engine. */
const INTERNAL_PROPS = new Set(["id", "image", "blocks", "bounds"]);

function toBase64(bytes: Uint8Array): string {
  if (typeof Buffer !== "undefined")
    return Buffer.from(bytes).toString("base64");
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary);
}

function encodeSrc(src: unknown, options: ToIrOptions): string | undefined {
  if (src == null) return undefined;
  if (typeof src === "string") {
    return options.resolveSrc ? options.resolveSrc(src) : src;
  }
  if (src instanceof Uint8Array) {
    return `data:application/octet-stream;base64,${toBase64(src)}`;
  }
  return undefined;
}

function isNode(value: unknown): value is Exclude<SoneNode, null | undefined> {
  return (
    value != null && typeof value === "object" && "type" in (value as object)
  );
}

function encodeProps(
  node: Exclude<SoneNode, null | undefined> | SpanNode,
  options: ToIrOptions,
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  const props = (node as { props: Record<string, unknown> }).props ?? {};

  for (const [key, value] of Object.entries(props)) {
    if (value === undefined) continue;
    if (INTERNAL_PROPS.has(key)) continue;

    if (key === "src") {
      const encoded = encodeSrc(value, options);
      if (encoded !== undefined) out.src = encoded;
      continue;
    }

    if (key === "background" && Array.isArray(value)) {
      out.background = value.map((bg) =>
        typeof bg === "string" ? bg : toIrNode(bg as SoneNode, options),
      );
      continue;
    }

    if (key === "clipImage" && isNode(value)) {
      out.clipImage = toIrNode(value as PhotoNode, options);
      continue;
    }

    if (key === "listStyle") {
      // Callback markers are materialized per item (see `encodeList`); a span
      // marker serializes as a node, a keyword stays a string.
      if (typeof value === "function") continue;
      out.listStyle = isNode(value)
        ? toIrNode(value as unknown as SoneNode, options)
        : value;
      continue;
    }

    if (typeof value === "function") continue;

    out[key] = value;
  }

  return out;
}

/** Resolve callback markers into a per-item `marker` node. */
function encodeListMarkers(node: ListNode, out: IrNode, options: ToIrOptions) {
  const listStyle = node.props.listStyle;
  if (typeof listStyle !== "function") return;

  const items = out.children ?? [];
  let index = 0;
  for (const item of items) {
    if (item.type !== "list-item") continue;
    const span = listStyle(index);
    item.props = item.props ?? {};
    item.props.marker = toIrNode(span as unknown as SoneNode, options);
    index++;
  }
}

export function toIrNode(
  node: SoneNode | SpanNode | null | undefined,
  options: ToIrOptions = {},
): IrNode {
  if (node == null) throw new Error("Cannot serialize a nullish node");

  const type = (node as { type: string }).type;
  const out: IrNode = { type };

  const props = encodeProps(
    node as Exclude<SoneNode, null | undefined>,
    options,
  );
  if (Object.keys(props).length > 0) out.props = props;

  if (type === "text" || type === "span") {
    const children = (node as TextNode | SpanNode).children;
    const list = typeof children === "string" ? [children] : (children ?? []);
    const inline = list
      .filter((c) => c != null)
      .map((c) => (typeof c === "string" ? c : toIrNode(c, options)));
    if (inline.length > 0) out.inline = inline;
    return out;
  }

  const children = (node as { children?: SoneNode[] }).children;
  if (Array.isArray(children)) {
    const encoded = children
      .filter((c) => c != null)
      .map((c) => toIrNode(c, options));
    if (encoded.length > 0) out.children = encoded;
  }

  if (type === "list") encodeListMarkers(node as ListNode, out, options);

  return out;
}

function resolveHeaderFooter(
  value: SoneHeaderFooter | undefined,
  options: ToIrOptions,
): IrNode | undefined {
  if (value == null) return undefined;
  if (typeof value !== "function") return toIrNode(value, options);
  const info: SonePageInfo = {
    pageNumber: PAGE_TOKEN,
    totalPages: TOTAL_TOKEN,
  };
  return substituteTokens(toIrNode(value(info), options));
}

/** Replace the probe page numbers with the template tokens Rust expands. */
function substituteTokens<T>(value: T): T {
  if (typeof value === "string") {
    return value
      .split(String(PAGE_TOKEN))
      .join("{pageNumber}")
      .split(String(TOTAL_TOKEN))
      .join("{totalPages}") as unknown as T;
  }
  if (Array.isArray(value)) {
    return value.map((v) => substituteTokens(v)) as unknown as T;
  }
  if (value != null && typeof value === "object") {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(value)) out[k] = substituteTokens(v);
    return out as unknown as T;
  }
  return value;
}

function encodeConfig(
  config: SoneRenderConfig | undefined,
  options: ToIrOptions,
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  if (options.density != null) out.density = options.density;
  if (config == null) return out;

  if (config.width != null) out.width = config.width;
  if (config.height != null) out.height = config.height;
  if (config.background != null) out.background = config.background;
  if (config.pageHeight != null) out.pageHeight = config.pageHeight;
  if (config.margin != null) out.margin = config.margin;
  if (config.lastPageHeight != null) out.lastPageHeight = config.lastPageHeight;

  const header = resolveHeaderFooter(config.header, options);
  if (header != null) out.header = header;
  const footer = resolveHeaderFooter(config.footer, options);
  if (footer != null) out.footer = footer;

  return out;
}

/** Serialize a node tree and render config into an IR document. */
export function toIR(
  node: SoneNode,
  config?: SoneRenderConfig,
  options: ToIrOptions = {},
): IrDocument {
  return {
    sone: IR_VERSION,
    config: encodeConfig(config, options),
    fonts: options.fonts ?? [],
    root: toIrNode(node, options),
  };
}
