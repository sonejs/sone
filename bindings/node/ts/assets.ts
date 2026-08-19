/**
 * Pre-resolving the image sources the engine will not fetch itself.
 *
 * `Assets::read` in `crates/sone-skia/src/assets.rs` handles `data:`, `asset:`
 * and filesystem paths, and deliberately refuses `http(s)` — a render never
 * does network I/O. Under WebAssembly there is no filesystem either, so paths
 * have to go too.
 *
 * Both gaps close in the same place: fetch the bytes here, hand them to
 * `registerImage`, and rewrite the source to the `asset:` handle. The handle is
 * the original source string, so the engine's decoded-image cache still hits
 * across renders.
 */
import type { NativeEngine } from "./backend.ts";
import { isDataUrl, isHttp, readBytes } from "./bytes.ts";
import type { IrDocument } from "./ir.ts";

interface WithSrc {
  src: string;
}

function hasSrc(value: unknown): value is WithSrc {
  return (
    value != null &&
    typeof value === "object" &&
    typeof (value as { src?: unknown }).src === "string"
  );
}

/** Every object in the document carrying a string `src`, in document order. */
function collect(value: unknown, out: WithSrc[]): void {
  if (Array.isArray(value)) {
    for (const item of value) collect(item, out);
    return;
  }
  if (value == null || typeof value !== "object") return;
  if (hasSrc(value)) out.push(value);
  for (const nested of Object.values(value)) collect(nested, out);
}

/**
 * Fetch and register whatever the engine cannot resolve, rewriting the
 * document in place. `hasFilesystem` is false for the WebAssembly engine, in
 * which case every source that is not already inline has to be fetched.
 */
export async function resolveAssets(
  document: IrDocument,
  engine: NativeEngine,
  hasFilesystem: boolean,
): Promise<void> {
  const targets: WithSrc[] = [];
  collect(document, targets);

  // Deduplicated, so the same image used twice is fetched once.
  const needed = new Set(
    targets
      .map((node) => node.src)
      .filter(
        (src) =>
          !isDataUrl(src) &&
          !src.startsWith("asset:") &&
          (isHttp(src) || !hasFilesystem),
      ),
  );
  if (needed.size === 0) return;

  await Promise.all(
    [...needed].map(async (src) => engine.registerImage(src, await readBytes(src))),
  );
  for (const node of targets) {
    if (needed.has(node.src)) node.src = `asset:${node.src}`;
  }
}
