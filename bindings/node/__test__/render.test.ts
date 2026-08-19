import { mkdtemp, readdir, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { beforeAll, describe, expect, it } from "vitest";

import {
  Column,
  Engine,
  Font,
  Photo,
  Row,
  sone,
  Text,
} from "../ts/index.ts";
import { FONTS, PNG_MAGIC, startsWith } from "./helpers.ts";

const card = () =>
  Column(
    Text("Hello").size(24).font("GeistMono").color("black"),
    Row(
      Column().bg("lightgreen").size(40).rounded(8),
      Column().bg("salmon").height(40).rounded(8).flex(1),
    ).gap(8),
  )
    .gap(12)
    .padding(16)
    .size(240, 160)
    .bg("khaki");

beforeAll(async () => {
  Font.reset();
  await Font.load("GeistMono", `${FONTS}/GeistMono-Regular.ttf`);
  await Font.load("NotoSansKhmer", `${FONTS}/NotoSansKhmer.ttf`);
});

describe("output formats", () => {
  it("renders a PNG", async () => {
    const png = await sone(card()).png();
    expect(startsWith(png, PNG_MAGIC)).toBe(true);
  });

  it("renders JPEG and WebP", async () => {
    expect(startsWith(await sone(card()).jpg(0.9), Uint8Array.from([0xff, 0xd8, 0xff]))).toBe(true);
    expect(startsWith(await sone(card()).webp(), "RIFF")).toBe(true);
  });

  it("renders an SVG with live text", async () => {
    const svg = Buffer.from(await sone(card()).svg()).toString("utf8");
    expect(svg).toContain("<svg");
    expect(svg).toContain("<text");
  });

  it("renders a PDF", async () => {
    expect(startsWith(await sone(card()).pdf(), "%PDF-")).toBe(true);
  });

  it("renders raw RGBA at four bytes per pixel", async () => {
    const raw = await sone(Column().size(10, 5).bg("red")).raw();
    expect(raw.length).toBe(10 * 5 * 4);
    expect([raw[0], raw[1], raw[2], raw[3]]).toEqual([255, 0, 0, 255]);
  });

  it("scales the raster by density", async () => {
    const one = await sone(Column().size(10).bg("red")).raw();
    const two = await sone(Column().size(10).bg("red")).raw({ density: 2 });
    expect(two.length).toBe(one.length * 4);
  });

  it("returns a Buffer on Node", async () => {
    expect(Buffer.isBuffer(await sone(card()).png())).toBe(true);
  });
});

describe("shaping", () => {
  it("renders Khmer, which needs a dictionary line breaker", async () => {
    const node = Text("ភាសាខ្មែរ").size(20).font("NotoSansKhmer").color("black");
    const png = await sone(Column(node).padding(8).bg("white")).png();
    expect(startsWith(png, PNG_MAGIC)).toBe(true);
    expect(png.length).toBeGreaterThan(100);
  });
});

describe("pagination", () => {
  const paged = () =>
    sone(
      Column(
        Column().height(300).bg("red"),
        Column().height(300).bg("green").pageBreak("before"),
        Column().height(300).bg("blue").pageBreak("before"),
      ),
      {
        width: 200,
        pageHeight: 400,
        footer: ({ pageNumber, totalPages }) =>
          Row(Text(`${pageNumber} / ${totalPages}`).size(10).font("GeistMono")),
      },
    );

  it("produces one raster per declared break", async () => {
    const pages = await paged().pages();
    expect(pages).toHaveLength(3);
    for (const page of pages) expect(startsWith(page, PNG_MAGIC)).toBe(true);
  });

  it("produces a multi-page PDF", async () => {
    const pdf = Buffer.from(await paged().pdf());
    expect(startsWith(pdf, "%PDF-")).toBe(true);
    expect(pdf.toString("latin1").match(/\/Type\s*\/Page[^s]/g)?.length).toBe(3);
  });

  it("writes one file per page", async () => {
    const dir = await mkdtemp(join(tmpdir(), "sone-pages-"));
    const written = await paged().savePages(join(dir, "p.png"));
    expect(written.map((p) => p.split("/").pop())).toEqual([
      "p-1.png",
      "p-2.png",
      "p-3.png",
    ]);
    expect((await readdir(dir)).sort()).toEqual(["p-1.png", "p-2.png", "p-3.png"]);
  });
});

describe("save", () => {
  it("infers the format from the suffix", async () => {
    const dir = await mkdtemp(join(tmpdir(), "sone-save-"));
    for (const [name, magic] of [
      ["out.png", PNG_MAGIC],
      ["out.pdf", "%PDF-"],
      ["out.jpeg", Uint8Array.from([0xff, 0xd8, 0xff])],
    ] as const) {
      const path = await sone(card()).save(join(dir, name));
      expect(startsWith(new Uint8Array(await readFile(path)), magic)).toBe(true);
    }
  });

  it("rejects a suffix it cannot map", async () => {
    await expect(sone(card()).save("/tmp/out.bmp")).rejects.toThrow(
      /cannot infer an output format/,
    );
  });
});

describe("introspection", () => {
  it("reports computed boxes", async () => {
    const layout = (await sone(Column().size(120, 60)).layout()) as {
      width: number;
      height: number;
    };
    expect(layout.width).toBe(120);
    expect(layout.height).toBe(60);
  });

  it("returns text boxes at line and word granularity", async () => {
    interface MetaNode {
      type: string;
      segments?: Array<{ text?: string; width: number }>;
      children?: MetaNode[];
    }
    const segments = async (granularity: "line" | "word") => {
      const meta = (await sone(
        Column(Text("one two three").size(14).font("GeistMono")).size(200, 60),
      ).metadata(granularity)) as MetaNode;
      const found: NonNullable<MetaNode["segments"]> = [];
      const walk = (node: MetaNode) => {
        found.push(...(node.segments ?? []));
        for (const child of node.children ?? []) walk(child);
      };
      walk(meta);
      return found;
    };

    expect((await segments("line")).length).toBe(1);
    // "one two three" is three words on one line.
    expect((await segments("word")).length).toBe(3);
  });

  it("exposes the document without rendering", () => {
    const rendered = sone(Column().size(10), { width: 100 });
    expect(rendered.document().config).toEqual({ width: 100 });
    expect(JSON.parse(rendered.json()).sone).toBe(1);
  });
});

describe("engines and assets", () => {
  it("keeps fonts isolated per engine", async () => {
    const engine = new Engine();
    expect(engine.hasFont("GeistMono")).toBe(false);
    await engine.registerFont("Solo", `${FONTS}/GeistMono-Regular.ttf`);
    expect(engine.fontFamilies()).toEqual(["Solo"]);
    expect(Font.has("Solo")).toBe(false);
  });

  it("reaches registered bytes through asset:", async () => {
    const engine = new Engine();
    const png = await sone(Column().size(8).bg("red")).png();
    await engine.registerImage("logo", png);
    const out = await sone(Column(Photo("asset:logo").size(8)).size(8), {
      engine,
    }).png();
    expect(startsWith(out, PNG_MAGIC)).toBe(true);
  });

  it("registers a font from raw bytes", async () => {
    const engine = new Engine();
    await engine.registerFont(
      "FromBytes",
      new Uint8Array(await readFile(`${FONTS}/GeistMono-Regular.ttf`)),
    );
    expect(engine.hasFont("FromBytes")).toBe(true);
  });

  it("drops one family with unload", async () => {
    const engine = new Engine();
    await engine.registerFont("Temp", `${FONTS}/GeistMono-Regular.ttf`);
    engine.unregisterFont("Temp");
    expect(engine.hasFont("Temp")).toBe(false);
  });
});
