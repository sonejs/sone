/**
 * Render configuration.
 *
 * Lifted from sone v2's `renderer.ts` so the surface is identical. The one
 * omission is `cache`, which handed the CanvasKit renderer a map of decoded
 * images — the Rust engine owns its own asset cache inside `Engine`, so there
 * is nothing for a caller to pass.
 */
import type { ColorValue, SoneNode } from "./core.ts";

/** Anything a font can be loaded from. */
export type FontSource = string | string[] | Uint8Array | ArrayBuffer | URL;

export interface SonePageInfo {
  /** 1-based page number */
  pageNumber: number;
  /** Total number of pages in the document */
  totalPages: number;
}

/**
 * A header or footer value: either a static node (same on every page) or a
 * function called once per page.
 *
 * The function form is called a single time, with sentinel page numbers, and
 * whatever it produces is turned into the literal `{pageNumber}` /
 * `{totalPages}` tokens that `pagination.rs` substitutes per page. So it may
 * *place* the numbers anywhere, but it cannot branch on them.
 */
export type SoneHeaderFooter = SoneNode | ((info: SonePageInfo) => SoneNode);

/**
 * Configuration for rendering
 */
export interface SoneRenderConfig {
  /** canvas width (auto-sized if not specified) */
  width?: number;
  /** canvas height (auto-sized if not specified) */
  height?: number;
  /** canvas background color */
  background?: ColorValue;
  /** When set, enables pagination — each page is this many pixels tall */
  pageHeight?: number;
  /**
   * Node (or function returning a node) rendered at the top of every page.
   * When a function, it receives `{ pageNumber, totalPages }` per page.
   */
  header?: SoneHeaderFooter;
  /**
   * Node (or function returning a node) rendered at the bottom of every page.
   * When a function, it receives `{ pageNumber, totalPages }` per page.
   */
  footer?: SoneHeaderFooter;
  /**
   * Page margins (pixels). A single number applies to all sides; an object
   * sets each side individually. Left/right expand the canvas; top/bottom
   * add space between the header/footer bands and the content area.
   */
  margin?:
    | number
    | { top?: number; right?: number; bottom?: number; left?: number };
  /**
   * Controls the height of the last page.
   * - `"uniform"` (default) — every page canvas is the same height; the last
   *   page has whitespace below the content.
   * - `"content"` — the last page canvas is only as tall as its content.
   */
  lastPageHeight?: "uniform" | "content";
}
