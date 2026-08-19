/**
 * Backend resolution for the browser and other runtimes with no Node-API.
 *
 * `@sonejs/sone-wasm` is an emscripten build of the same Rust engine, and its
 * module has to be instantiated before anything can be called — so unlike the
 * native path, the backend is genuinely asynchronous here. `await Font.load()`
 * or `await sone(...).png()` drives the instantiation; the synchronous
 * accessors (`Font.has`, `Font.families`) only work once something has been
 * awaited, which is the same rule sone v2 has for CanvasKit.
 */
import type { Backend } from "./backend.ts";

let cached: Backend | null = null;
let pending: Promise<Backend> | null = null;

export function backendSync(): Backend {
  if (cached == null) {
    throw new Error(
      "The sone WebAssembly engine is not ready yet. Await something first — " +
        "`await Font.load(...)` or `await sone(node).png()` — then this call works.",
    );
  }
  return cached;
}

export function loadBackend(): Promise<Backend> {
  if (cached != null) return Promise.resolve(cached);
  pending ??= import("@sonejs/sone-wasm").then(async (mod) => {
    const wasm = await mod.load();
    cached = { Engine: wasm.Engine, version: wasm.version, hasFilesystem: false };
    return cached;
  });
  return pending;
}

export const isSynchronous = false;
