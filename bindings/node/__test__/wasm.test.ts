/**
 * The WebAssembly engine agrees with the native one.
 *
 * This drives `dist/browser.js` — the exact bundle a browser gets, backed by
 * `@sonejs/sone-wasm` — and then renders the same documents through the native
 * addon and compares. Two engines that disagree would be far worse than one
 * that is simply slower, so this is the suite that keeps them honest.
 *
 * They agree exactly on everything but glyph coverage: layout trees are
 * identical field for field, and every non-text primitive is pixel-identical.
 * Text is bounded rather than exact, because macOS rasterizes glyphs through
 * CoreText and the emscripten build through Skia's bundled FreeType.
 *
 * It needs both builds: `npm run build --workspace @sonejs/sone-wasm` then
 * `npm run build --workspace @sonejs/sone`. Skips cleanly when they are absent.
 */
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

import { beforeAll, describe, expect, it } from "vitest";

import { Column, Engine as NativeEngine, Path, toIR } from "../ts/index.ts";
import { FONTS, PNG_MAGIC, startsWith } from "./helpers.ts";

const BROWSER_BUNDLE = fileURLToPath(new URL("../dist/browser.js", import.meta.url));
const WASM_BUILD = fileURLToPath(new URL("../../wasm/dist/sone.wasm", import.meta.url));
const available = existsSync(BROWSER_BUNDLE) && existsSync(WASM_BUILD);

type Browser = typeof import("../ts/index.ts");

let browser: Browser;
let fontBytes: Uint8Array;

/** The corpus both engines must agree on, built with whichever builder. */
function documents(b: Browser) {
  return {
    card: b
      .Column(
        b.Text("Hello ភាសាខ្មែរ").size(20).font("GeistMono", "NotoSansKhmer").color("black"),
        b.Row(
          b.Column().bg("lightgreen").size(40).rounded(8),
          b.Column().bg("salmon").height(40).rounded(8).flex(1),
        ).gap(8),
      )
      .gap(12)
      .padding(16)
      .size(260, 160)
      .bg("khaki")
      .cornerRadius(20)
      .cornerSmoothing(0.6),
    shapes: b
      .Column(
        b.Path("M0 0 L40 40 L0 40 Z").fill("teal").size(40),
        b.Column().size(30).bg("linear-gradient(45deg, red, blue)").rounded(6),
      )
      .gap(6)
      .padding(8)
      .bg("white"),
    paged: b.Column(
      b.Column().height(120).bg("red"),
      b.Column().height(120).bg("green").pageBreak("before"),
    ),
  };
}

describe.skipIf(!available)("the WebAssembly engine", () => {
  beforeAll(async () => {
    browser = (await import(/* @vite-ignore */ BROWSER_BUNDLE)) as Browser;
    fontBytes = new Uint8Array(await readFile(`${FONTS}/GeistMono-Regular.ttf`));
    const khmer = new Uint8Array(await readFile(`${FONTS}/NotoSansKhmer.ttf`));
    await browser.Font.load("GeistMono", fontBytes);
    await browser.Font.load("NotoSansKhmer", khmer);
  }, 120_000);

  it("registers fonts and reports them", () => {
    expect(browser.Font.has("GeistMono")).toBe(true);
    expect(browser.Font.families().sort()).toEqual(["GeistMono", "NotoSansKhmer"]);
    expect(typeof browser.version()).toBe("string");
  });

  it("renders a PNG", async () => {
    const png = await browser.sone(documents(browser).card).png({ density: 2 });
    expect(startsWith(png, PNG_MAGIC)).toBe(true);
  });

  it("renders vector output", async () => {
    const svg = new TextDecoder().decode(
      await browser.sone(documents(browser).card).svg(),
    );
    expect(svg).toContain("<svg");
    expect(svg).toContain("<text");
    expect(startsWith(await browser.sone(documents(browser).card).pdf(), "%PDF-")).toBe(true);
  });

  it("paginates", async () => {
    const pages = await browser
      .sone(documents(browser).paged, { width: 200, pageHeight: 200 })
      .pages();
    expect(pages).toHaveLength(2);
  });

  it("reports the computed layout", async () => {
    const layout = (await browser.sone(browser.Column().size(120, 60)).layout()) as {
      width: number;
      height: number;
    };
    expect([layout.width, layout.height]).toEqual([120, 60]);
  });

  it("reads a registered asset", async () => {
    const swatch = await browser.sone(browser.Column().size(8).bg("red")).png();
    const engine = new browser.Engine();
    await engine.registerImage("swatch", swatch);
    const out = await browser
      .sone(browser.Column(browser.Photo("asset:swatch").size(8)).size(8), { engine })
      .png();
    expect(startsWith(out, PNG_MAGIC)).toBe(true);
  });

  it("raises the same typed errors", async () => {
    await expect(
      new browser.Engine().render('{"sone":99,"root":{"type":"column"}}', "png"),
    ).rejects.toBeInstanceOf(browser.IrError);
  });

  it("says plainly that it has no filesystem", async () => {
    await expect(
      new browser.Engine().registerFontFile("X", "fonts/X.ttf"),
    ).rejects.toThrow(/no filesystem/);
  });
});

describe.skipIf(!available)("parity between the two engines", () => {
  /**
   * Both engines get their own `Engine` with the same fonts, so neither can
   * benefit from state the other does not have.
   */
  async function pair(): Promise<[NativeEngine, InstanceType<Browser["Engine"]>]> {
    const khmer = new Uint8Array(await readFile(`${FONTS}/NotoSansKhmer.ttf`));
    const native = new NativeEngine();
    const wasm = new browser.Engine();
    for (const engine of [native, wasm]) {
      await engine.registerFont("GeistMono", fontBytes);
      await engine.registerFont("NotoSansKhmer", khmer);
    }
    return [native, wasm];
  }

  /** Raw RGBA rather than PNG — a difference should mean pixels, not zlib. */
  async function pixels(document: string): Promise<[Uint8Array, Uint8Array]> {
    const [native, wasm] = await pair();
    return Promise.all([
      native.render(document, "raw", 2),
      wasm.render(document, "raw", 2),
    ]);
  }

  function differing(a: Uint8Array, b: Uint8Array): number {
    let count = 0;
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) count++;
    return count;
  }

  // Everything that is not a glyph goes down the same Skia paths in both
  // builds, so it is pixel-exact — worth asserting exactly, because a
  // regression here would mean a real difference in the drawing code.
  const DRAWING = {
    "solid box": Column().size(60, 40).bg("red"),
    "rounded border": Column()
      .size(60, 40)
      .bg("salmon")
      .cornerRadius(12)
      .borderColor("teal")
      .borderWidth(3),
    squircle: Column().size(60, 40).bg("orange").cornerRadius(16).cornerSmoothing(0.8),
    gradient: Column().size(60, 40).bg("linear-gradient(45deg, red, blue)"),
    shadow: Column().size(60, 40).bg("white").shadow("0 4px 12px rgba(0,0,0,.4)"),
    path: Path("M0 0 L40 40 L0 40 Z").fill("teal").size(40),
  };

  it.each(Object.keys(DRAWING))("%s is pixel-identical", async (name) => {
    const node = DRAWING[name as keyof typeof DRAWING];
    const document = JSON.stringify(toIR(Column(node).padding(6).bg("white")));
    const [a, b] = await pixels(document);
    expect(differing(a, b)).toBe(0);
  });

  it("lays text out identically, to the last field", async () => {
    const [native, wasm] = await pair();
    const document = JSON.stringify(toIR(documents(browser).card as never));
    expect(await wasm.dumpLayout(document)).toBe(await native.dumpLayout(document));
  });

  it("rasterizes text closely, but not identically", async () => {
    // The one place the builds genuinely diverge: glyphs go through CoreText on
    // macOS and through Skia's bundled FreeType under emscripten, so coverage
    // values differ by a hair along the edges. Geometry is identical (above),
    // and this bounds the difference rather than pretending it is not there.
    const document = JSON.stringify(toIR(documents(browser).card as never));
    const [a, b] = await pixels(document);
    const ratio = differing(a, b) / a.length;
    expect(ratio).toBeGreaterThan(0); // if this ever fails, delete the waiver
    expect(ratio).toBeLessThan(0.05);
  });
});
