/**
 * The vendored builder still agrees with the one it was vendored from.
 *
 * `ts/core.ts` and `ts/ir.ts` are sone v2's files, changed only in their import
 * lines. This suite builds the same tree with both copies and diffs the IR, so
 * a drift in either direction shows up as a failing test rather than as a
 * rendering difference months later.
 *
 * It needs a checkout of the TypeScript engine. Point `SONE_TS_REPO` at one, or
 * leave it next to this repository as `../sone`; otherwise the suite skips.
 */
import { existsSync } from "node:fs";

import { describe, expect, it } from "vitest";

import * as vendored from "../ts/core.ts";
import { toIR as vendoredToIR } from "../ts/ir.ts";
import { REPO } from "./helpers.ts";

const tsRepo = process.env.SONE_TS_REPO ?? `${REPO}../sone`;
const available = existsSync(`${tsRepo}/src/ir.ts`);

type Builders = typeof vendored;

/** Every tree worth diffing, built from whichever copy of the builder. */
function corpus(b: Builders): Array<[string, unknown]> {
  const { ClipGroup, Column, Grid, List, ListItem, PageBreak, Path, Photo, Row, Span, Table, TableCell, TableRow, Text, TextDefault } = b;
  return [
    [
      "card",
      Column(
        Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white"),
        Row(
          Column().bg("lightgreen").size(50).borderRadius(14).borderColor("teal").borderWidth(2),
          Column().bg("salmon").height(50).borderRadius(14).flex(1),
        ).gap(10),
      )
        .gap(20)
        .padding(20, 16)
        .size(420, 300)
        .bg("khaki")
        .rotate(20)
        .scale(1.1)
        .opacity(0.9)
        .blur(2)
        .saturate(1.4)
        .shadow("0 2px 8px rgba(0,0,0,.3)"),
    ],
    [
      "text",
      Text("a ", Span("bold").weight("bold").underline(), " c")
        .size(18)
        .font("Inter", "serif")
        .lineHeight(1.4)
        .align("center")
        .maxLines(3)
        .textOverflow("ellipsis")
        .indent(12)
        .tabStops(20, 40)
        .letterSpacing(0.5),
    ],
    ["text-default", TextDefault(Text("x")).size(11).color("gray").weight(500)],
    ["grid", Grid(Column(), Column()).columns(1, "auto", "2fr").rows(10).gap(4)],
    ["photo", Photo("logo.png").scaleType("cover", "center").flipHorizontal().preserveAspectRatio()],
    ["photo-bytes", Photo(new Uint8Array([0, 1, 2, 250, 251]))],
    [
      "path",
      Path("M0 0 L10 10 Z")
        .fill("red")
        .stroke("blue")
        .strokeWidth(2)
        .strokeLineCap("round")
        .strokeDashArray(2, 4)
        .fillRule("evenodd"),
    ],
    [
      "table",
      Table(
        TableRow(TableCell(Text("a")).colspan(2), TableCell(Text("b"))),
        TableRow(TableCell(Text("c")).rowspan(2)),
      ).spacing(4, 8),
    ],
    [
      "list",
      List(ListItem(Text("one")), ListItem(Text("two")))
        .listStyle("decimal")
        .markerGap(6)
        .startIndex(3),
    ],
    ["list-callback", List(ListItem(Text("one")), ListItem(Text("two"))).listStyle((i) => Span(`${i}.`))],
    ["clip-group", ClipGroup("M0 0 L10 10 Z", Column().size(10))],
    ["page-break", Column(PageBreak(), Column().height(10))],
    ["nested-bg", Column().bg(Photo("bg.png")).bg("red")],
  ];
}

const config = {
  width: 794,
  height: 1123,
  background: "white",
  pageHeight: 1123,
  margin: { top: 10, right: 20 },
  lastPageHeight: "content",
} as const;

describe.skipIf(!available)("IR parity with the TypeScript engine", async () => {
  const upstream = available
    ? ((await import(/* @vite-ignore */ `${tsRepo}/src/core.ts`)) as unknown as Builders)
    : vendored;
  const upstreamToIR = available
    ? ((await import(/* @vite-ignore */ `${tsRepo}/src/ir.ts`)) as { toIR: typeof vendoredToIR })
        .toIR
    : vendoredToIR;

  const ours = corpus(vendored);
  const theirs = corpus(upstream);

  it.each(ours.map(([name], i) => [name, i] as const))(
    "%s serializes identically",
    (_name, index) => {
      expect(vendoredToIR(ours[index][1] as never)).toEqual(
        upstreamToIR(theirs[index][1] as never),
      );
    },
  );

  it("serializes render config identically", () => {
    expect(vendoredToIR(vendored.Column(), config as never)).toEqual(
      upstreamToIR(upstream.Column(), config as never),
    );
  });

  it("substitutes page tokens identically", () => {
    const footer =
      (b: Builders) =>
      ({ pageNumber, totalPages }: { pageNumber: number; totalPages: number }) =>
        b.Row(b.Text(`${pageNumber} of ${totalPages}`));
    expect(
      vendoredToIR(vendored.Column(), {
        pageHeight: 100,
        footer: footer(vendored),
      } as never),
    ).toEqual(
      upstreamToIR(upstream.Column(), {
        pageHeight: 100,
        footer: footer(upstream),
      } as never),
    );
  });
});
