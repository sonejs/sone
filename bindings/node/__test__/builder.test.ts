/**
 * The builder produces the documents the engine expects.
 *
 * `ts/core.ts` and `ts/ir.ts` are sone v2's own files vendored unchanged, so
 * these assertions are really about the vendoring having stayed faithful — see
 * `ir-parity.test.ts` for the check against that package's live serializer.
 */
import { describe, expect, it } from "vitest";

import {
  ClipGroup,
  Column,
  Grid,
  List,
  ListItem,
  PageBreak,
  Path,
  Photo,
  Row,
  Span,
  Table,
  TableCell,
  TableRow,
  Text,
  TextDefault,
  toIR,
} from "../ts/index.ts";

describe("the tree matches the TypeScript shape", () => {
  it("nests containers, props and children", () => {
    const root = Column(
      Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white"),
      Row(
        Column().bg("lightgreen").size(50).borderRadius(14),
        Column().bg("salmon").height(50).borderRadius(14).flex(1),
      ).gap(10),
    )
      .gap(20)
      .padding(20)
      .size(420, 300)
      .bg("khaki");

    expect(toIR(root)).toEqual({
      sone: 1,
      config: {},
      fonts: [],
      root: {
        type: "column",
        props: {
          gap: 20,
          padding: 20,
          width: 420,
          height: 300,
          background: ["khaki"],
        },
        children: [
          {
            type: "column",
            props: {
              flex: 1,
              cornerRadius: [20],
              cornerSmoothing: 0.7,
              background: ["white"],
            },
          },
          {
            type: "row",
            props: { gap: 10 },
            children: [
              {
                type: "column",
                props: {
                  background: ["lightgreen"],
                  width: 50,
                  height: 50,
                  cornerRadius: [14],
                },
              },
              {
                type: "column",
                props: {
                  background: ["salmon"],
                  height: 50,
                  cornerRadius: [14],
                  flex: 1,
                },
              },
            ],
          },
        ],
      },
    });
  });

  it("treats Text.size as the font size, not the box size", () => {
    expect(toIR(Text("hi").size(28)).root.props).toEqual({ size: 28 });
    expect(toIR(Column().size(28)).root.props).toEqual({ width: 28, height: 28 });
  });

  it("nests spans inline", () => {
    expect(toIR(Text("a ", Span("b").weight("bold"), " c")).root).toEqual({
      type: "text",
      inline: ["a ", { type: "span", props: { weight: "bold" }, inline: ["b"] }, " c"],
    });
  });

  it("follows the CSS 1-4 value shorthand", () => {
    expect(toIR(Column().padding(1)).root.props).toEqual({ padding: 1 });
    expect(toIR(Column().padding(1, 2)).root.props).toEqual({
      paddingTop: 1,
      paddingRight: 2,
      paddingBottom: 1,
      paddingLeft: 2,
    });
    expect(toIR(Column().padding(1, 2, 3)).root.props).toEqual({
      paddingTop: 1,
      paddingRight: 2,
      paddingBottom: 3,
      paddingLeft: 2,
    });
    expect(toIR(Column().padding(1, 2, 3, 4)).root.props).toEqual({
      paddingTop: 1,
      paddingRight: 2,
      paddingBottom: 3,
      paddingLeft: 4,
    });
  });

  it("accumulates filters in call order", () => {
    expect(toIR(Column().blur(2).grayscale(0.5).invert(1)).root.props).toEqual({
      filters: ["blur(2px)", "grayscale(0.5)", "invert(1)"],
    });
  });

  it("appends backgrounds and shadows", () => {
    const node = Column().bg("red").bg("blue").shadow("0 1px 2px black");
    expect(node.props.background).toEqual(["red", "blue"]);
    expect(node.props.shadows).toEqual(["0 1px 2px black"]);
  });

  it("turns Photo bytes into a data URL and leaves strings alone", () => {
    const bytes = toIR(Photo(new Uint8Array([1, 2, 3]))).root.props?.src;
    expect(bytes).toBe("data:application/octet-stream;base64,AQID");
    expect(toIR(Photo("logo.png")).root.props?.src).toBe("logo.png");
    expect(toIR(Photo("asset:logo")).root.props?.src).toBe("asset:logo");
  });

  it("serializes a nested node inside props", () => {
    const ir = toIR(Column().bg(Photo("bg.png")));
    expect(ir.root.props?.background).toEqual([
      { type: "photo", props: { src: "bg.png" } },
    ]);
  });

  it("emits a zero-height break for PageBreak", () => {
    expect(toIR(PageBreak()).root.props).toEqual({ height: 0, pageBreak: "before" });
  });

  it("serializes every node type", () => {
    const types = [
      Column(),
      Row(),
      Grid(),
      Text("x"),
      TextDefault(),
      Photo("a.png"),
      Path("M0 0 L1 1"),
      Table(TableRow(TableCell())),
      List(ListItem()),
      ClipGroup("M0 0 L1 1"),
    ].map((node) => toIR(node).root.type);

    expect(types).toEqual([
      "column",
      "row",
      "grid",
      "text",
      "text-default",
      "photo",
      "path",
      "table",
      "list",
      "clip-group",
    ]);
  });

  it("materializes callback list markers per item", () => {
    const ir = toIR(
      List(ListItem(Text("a")), ListItem(Text("b"))).listStyle((i) =>
        Span(`${i + 1})`),
      ),
    );
    const markers = ir.root.children?.map((child) => child.props?.marker);
    expect(markers).toEqual([
      { type: "span", inline: ["1)"] },
      { type: "span", inline: ["2)"] },
    ]);
  });
});

describe("render config", () => {
  it("carries page setup into config", () => {
    const ir = toIR(Column(), {
      width: 794,
      pageHeight: 1123,
      margin: 64,
      lastPageHeight: "content",
    });
    expect(ir.config).toEqual({
      width: 794,
      pageHeight: 1123,
      margin: 64,
      lastPageHeight: "content",
    });
  });

  it("replaces header/footer callbacks with page tokens", () => {
    const ir = toIR(Column(), {
      pageHeight: 400,
      footer: ({ pageNumber, totalPages }) =>
        Row(Text(`${pageNumber} of ${totalPages}`)),
    });
    expect(ir.config.footer).toEqual({
      type: "row",
      children: [{ type: "text", inline: ["{pageNumber} of {totalPages}"] }],
    });
  });
});
