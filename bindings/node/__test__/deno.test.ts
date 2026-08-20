/// <reference lib="deno.ns" />
/**
 * Deno loads the same Node-API addon Node does.
 *
 * Deno needs two things npm packages usually get for free: `--allow-ffi`,
 * because a native addon is machine code that the permission system cannot
 * sandbox, and a real `node_modules` directory (`--node-modules-dir=auto`) for
 * the platform package to be resolvable.
 *
 *   deno test --allow-read --allow-env --allow-ffi --node-modules-dir=auto \
 *     __test__/deno.test.ts
 */
import { fileURLToPath } from "node:url";

import { Column, Font, sone, Text, version } from "../dist/index.mjs";

// `.pathname` on a file URL is not a path on Windows — it keeps the leading
// slash, so `D:\...` arrives as `/D:/...`. On POSIX the two coincide, which is
// why this only ever failed on the Windows runner.
const FONT = fileURLToPath(new URL("../../../fixtures/font/GeistMono-Regular.ttf", import.meta.url));

Deno.test("renders a PNG through the native addon", async () => {
  await Font.load("GeistMono", FONT);
  if (!Font.has("GeistMono")) throw new Error("font was not registered");
  if (typeof version() !== "string") throw new Error("no version");

  const png = await sone(
    Column(Text("Deno").size(18).font("GeistMono").color("black"))
      .padding(12)
      .bg("khaki"),
  ).png({ density: 2 });

  const magic = [0x89, 0x50, 0x4e, 0x47];
  if (!magic.every((b, i) => png[i] === b)) throw new Error("not a PNG");
});

Deno.test("paginates", async () => {
  const pages = await sone(
    Column(
      Column().height(100).bg("red"),
      Column().height(100).bg("blue").pageBreak("before"),
    ),
    { width: 100, pageHeight: 150 },
  ).pages();
  if (pages.length !== 2) throw new Error(`expected 2 pages, got ${pages.length}`);
});
