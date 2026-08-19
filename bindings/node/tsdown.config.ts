import { fileURLToPath } from "node:url";

import { defineConfig } from "tsdown";

const browserBinding = fileURLToPath(
  new URL("ts/binding.browser.ts", import.meta.url),
);

/**
 * Two builds of one entry.
 *
 * `dist/index.*` loads the Node-API addon and is what Node, Bun and Deno get.
 * `dist/browser.js` swaps `ts/binding.ts` for `ts/binding.browser.ts`, which
 * instantiates `@sonejs/sone-wasm` instead — everything above that one file is
 * identical, which is the point of the `Backend` contract in `ts/backend.ts`.
 */
export default defineConfig([
  {
    entry: { index: "ts/index.ts" },
    format: ["esm", "cjs"],
    platform: "node",
    dts: true,
    clean: true,
    outDir: "dist",
    // The addon loader is required at runtime from the package root, not
    // bundled — it has to find the `.node` file next to itself.
    external: ["../binding.cjs", "node:module", "node:fs/promises"],
  },
  {
    entry: { browser: "ts/index.ts" },
    format: ["esm"],
    platform: "browser",
    dts: true,
    clean: false,
    outDir: "dist",
    alias: { "./binding.ts": browserBinding },
    external: ["@sonejs/sone-wasm", "node:fs/promises"],
  },
]);
