import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["__test__/*.test.ts"],
    exclude: ["__test__/bun.test.ts", "__test__/deno.test.ts"],
    testTimeout: 60_000,
    hookTimeout: 60_000,
    // One engine per thread is the binding contract, and the default engine is
    // process-wide — so the suites that share it must not run in parallel.
    fileParallelism: false,
  },
});
