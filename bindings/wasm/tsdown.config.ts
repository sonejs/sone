import { defineConfig } from "tsdown";

/**
 * Only the glue is bundled. `sone.js` and `sone.wasm` are copied into `dist/`
 * by `build.sh` and stay external — emscripten's loader resolves the binary
 * relative to itself, and bundling it would break that.
 */
export default defineConfig({
  entry: { index: "glue/index.ts" },
  format: ["esm"],
  platform: "neutral",
  dts: true,
  clean: false,
  outDir: "dist",
  external: ["./sone.js"],
});
