/**
 * Bun loads the same Node-API addon Node does.
 *
 * Run with `bun test __test__/bun.test.ts`. This deliberately imports the built
 * `dist/` bundle rather than `ts/`, so it exercises what a consumer installs.
 */
import { fileURLToPath } from "node:url";

import { expect, test } from "bun:test";

// @ts-expect-error — resolved by Bun at runtime, not by this repo's tsconfig.
import { Column, Font, Row, sone, Text, version } from "../dist/index.mjs";

// `.pathname` on a file URL is not a path on Windows — it keeps the leading
// slash, so `D:\...` arrives as `/D:/...`. On POSIX the two coincide, which is
// why this only ever failed on the Windows runner.
const FONT = fileURLToPath(new URL("../../../fixtures/font/GeistMono-Regular.ttf", import.meta.url));

test("renders a PNG through the native addon", async () => {
  await Font.load("GeistMono", FONT);
  expect(Font.has("GeistMono")).toBe(true);
  expect(typeof version()).toBe("string");

  const png = await sone(
    Column(Text("Bun").size(18).font("GeistMono").color("black"), Row().height(8))
      .padding(12)
      .bg("khaki"),
  ).png({ density: 2 });

  expect(png.length).toBeGreaterThan(100);
  expect([...png.slice(0, 4)]).toEqual([0x89, 0x50, 0x4e, 0x47]);
});

test("paginates", async () => {
  const pages = await sone(
    Column(
      Column().height(100).bg("red"),
      Column().height(100).bg("blue").pageBreak("before"),
    ),
    { width: 100, pageHeight: 150 },
  ).pages();
  expect(pages.length).toBe(2);
});
