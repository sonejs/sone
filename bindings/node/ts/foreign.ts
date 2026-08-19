/**
 * Types `core.ts` refers to that belong to other layers.
 *
 * The vendored builder is the sone v2 file unchanged except for its imports,
 * which pointed at the CanvasKit renderer. Those three types are declared here
 * structurally instead, so the builder keeps the same signatures without
 * dragging in an engine this package does not use.
 */

/**
 * A pre-parsed CSS gradient, structurally the shape `gradient-parser` returns.
 *
 * Accepted by `.bg()` for source compatibility with sone v2, but the IR only
 * carries gradients as CSS strings (`ir::Background` is `Css(String) |
 * Photo(Node)`), so a parsed node reaches the engine as an unparseable
 * background and raises an `IrError`. Pass the gradient as a string instead.
 */
export interface GradientNode {
  type: string;
  orientation?: unknown;
  colorStops: unknown[];
}

/**
 * A decoded CanvasKit image handle. The Rust engine decodes images itself, so
 * this exists only to keep `PhotoProps` assignable; use a path, a URL, an
 * `asset:` handle or raw bytes.
 */
export interface SoneImage {
  readonly width: number;
  readonly height: number;
}

/** A laid-out paragraph. Engine-internal; `ir.ts` strips it before serializing. */
export interface SoneParagraphBlock {
  readonly [key: string]: unknown;
}
