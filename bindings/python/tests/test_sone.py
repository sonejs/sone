"""Tests for the Python bindings.

Run with:  maturin develop --release && pytest
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

import sone
from sone import (
    AssetError,
    ClipGroup,
    Column,
    Engine,
    Font,
    Grid,
    IrError,
    List,
    ListItem,
    Path as SvgPath,
    Photo,
    Row,
    Span,
    Table,
    TableCell,
    TableRow,
    Text,
    TextDefault,
)

REPO = Path(__file__).resolve().parents[3]
FONTS = REPO / "fixtures" / "font"


@pytest.fixture(scope="session", autouse=True)
def _fonts():
    Font.reset()
    Font.load("GeistMono", FONTS / "GeistMono-Regular.ttf")
    Font.load("NotoSansKhmer", FONTS / "NotoSansKhmer.ttf")


# ── builder API ──────────────────────────────────────────────────────────────


def test_tree_matches_the_typescript_shape():
    root = (
        Column(
            Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white"),
            Row(
                Column().bg("lightgreen").size(50).borderRadius(14).borderColor("teal").borderWidth(2),
                Column().bg("salmon").height(50).borderRadius(14).flex(1),
                Column().bg("orange").size(50).borderRadius(14),
            ).gap(10),
        )
        .gap(20)
        .padding(20)
        .size(420, 300)
        .bg("khaki")
        .cornerRadius(28)
        .borderColor("chocolate")
        .borderWidth(4)
        .rotate(20)
    )

    ir = root.to_ir()
    assert ir["type"] == "column"
    assert ir["props"] == {
        "gap": 20,
        "padding": 20,
        "width": 420,
        "height": 300,
        "background": ["khaki"],
        "cornerRadius": [28],
        "borderColor": "chocolate",
        "borderWidth": 4,
        "rotation": 20,
    }
    assert len(ir["children"]) == 2
    first, row = ir["children"]
    assert first["props"]["cornerSmoothing"] == 0.7
    assert row["type"] == "row"
    assert row["props"]["gap"] == 10
    assert [c["props"]["background"][0] for c in row["children"]] == ["lightgreen", "salmon", "orange"]


def test_snake_case_aliases_are_the_same_calls():
    camel = Column().cornerRadius(20).borderColor("red").maxWidth(100).flexDirection("row")
    snake = Column().corner_radius(20).border_color("red").max_width(100).flex_direction("row")
    assert camel.to_ir() == snake.to_ir()


def test_size_sets_both_dimensions():
    assert Column().size(50).props == {"width": 50, "height": 50}
    assert Column().size(50, 20).props == {"width": 50, "height": 20}


@pytest.mark.parametrize(
    "args,expected",
    [
        ((10,), {"padding": 10}),
        ((10, 20), {"paddingTop": 10, "paddingRight": 20, "paddingBottom": 10, "paddingLeft": 20}),
        ((10, 20, 30), {"paddingTop": 10, "paddingRight": 20, "paddingBottom": 30, "paddingLeft": 20}),
        ((10, 20, 30, 40), {"paddingTop": 10, "paddingRight": 20, "paddingBottom": 30, "paddingLeft": 40}),
    ],
)
def test_box_shorthand_follows_css(args, expected):
    assert Column().padding(*args).props == expected


def test_text_size_is_the_font_size():
    # `TextPropsBuilder` omits the layout `size`, so this must not set width.
    assert Text("hi").size(14).props == {"size": 14}
    assert Column().size(14).props == {"width": 14, "height": 14}


def test_spans_nest_inside_text():
    node = Text("hello ", Span("world").color("red").weight("bold")).size(12)
    ir = node.to_ir()
    assert ir["inline"][0] == "hello "
    assert ir["inline"][1] == {"type": "span", "props": {"color": "red", "weight": "bold"}, "inline": ["world"]}


def test_filters_accumulate_in_order():
    node = Column().blur(4).grayscale(0.5).hueRotate(90)
    assert node.props["filters"] == ["blur(4px)", "grayscale(0.5)", "hue-rotate(90)"]


def test_shadows_and_backgrounds_append():
    node = Column().bg("red").bg("blue").shadow("1px 1px 2px black").shadow("0 0 4px red")
    assert node.props["background"] == ["red", "blue"]
    assert len(node.props["shadows"]) == 2


def test_decoration_colours_accept_none():
    assert Span("x").underline().underlineColor(None).props == {"underline": 1.0, "underlineColor": None}


def test_photo_accepts_bytes():
    ir = Photo(b"\x89PNG\r\n\x1a\n").to_ir()
    assert ir["props"]["src"].startswith("data:application/octet-stream;base64,")


def test_photo_scale_alignment_keywords():
    assert Photo("x.png").scaleType("cover", "end").props["scaleAlignment"] == 1.0


def test_nested_nodes_serialize_inside_props():
    node = Column().bg(Photo("x.png").scaleType("cover"))
    assert node.to_ir()["props"]["background"][0]["type"] == "photo"


def test_every_node_type_serializes():
    tree = Column(
        Grid(Column()).columns("1fr", "1fr"),
        Table(TableRow(TableCell(Text("a")).colspan(2))).spacing(4, 2),
        List(ListItem(Text("x"))).listStyle("decimal"),
        ClipGroup("M0,0 L10,0 L10,10 Z", Column()),
        SvgPath("M0,0 L10,10").fill("red").strokeDashArray(4, 2),
        TextDefault(Text("y")).size(9),
        Photo("x.png"),
    )
    types = [c["type"] for c in tree.to_ir()["children"]]
    assert types == ["grid", "table", "list", "clip-group", "path", "text-default", "photo"]


# ── rendering ────────────────────────────────────────────────────────────────


CARD = Column(Text("Hello").font("GeistMono").size(24).color("#111")).padding(20).bg("white")


def test_png_render():
    data = sone.sone(CARD).png()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"


def test_jpeg_and_webp_render():
    assert sone.sone(CARD).jpg()[:2] == b"\xff\xd8"
    assert sone.sone(CARD).webp()[:4] == b"RIFF"


def test_svg_render_has_live_text():
    svg = sone.sone(CARD).svg().decode("utf-8")
    assert svg.startswith("<?xml")
    assert "<svg" in svg


def test_pdf_render():
    assert sone.sone(CARD).pdf()[:4] == b"%PDF"


def test_raw_render_is_rgba():
    doc = Column().size(4, 2).bg("red")
    raw = sone.sone(doc).raw()
    assert len(raw) == 4 * 2 * 4
    assert raw[:4] == b"\xff\x00\x00\xff"


def test_density_scales_the_raster():
    doc = Column().size(10, 10).bg("red")
    assert len(sone.sone(doc).raw(density=1)) == 10 * 10 * 4
    assert len(sone.sone(doc).raw(density=3)) == 30 * 30 * 4


def test_save_infers_the_format(tmp_path):
    out = sone.sone(CARD).save(tmp_path / "card.jpg")
    assert Path(out).read_bytes()[:2] == b"\xff\xd8"


def test_save_rejects_an_unknown_suffix(tmp_path):
    with pytest.raises(ValueError, match="cannot infer"):
        sone.sone(CARD).save(tmp_path / "card.tiff")


def test_one_shot_render_helper():
    assert sone.render(CARD, "png")[:8] == b"\x89PNG\r\n\x1a\n"


# ── pagination ───────────────────────────────────────────────────────────────


def _long_document():
    lines = [Text(f"line {i}").font("GeistMono").size(12) for i in range(60)]
    return Column(*lines).gap(4).width(300)


def test_pages_split_and_carry_page_tokens(tmp_path):
    doc = sone.sone(
        _long_document(),
        width=300,
        page_height=200,
        header=Row(Text("Report").font("GeistMono").size(9)).padding(4),
        footer=Row(Text("{pageNumber} / {totalPages}").font("GeistMono").size(9)).padding(4),
    )
    pages = doc.pages()
    assert len(pages) > 1
    assert all(p[:8] == b"\x89PNG\r\n\x1a\n" for p in pages)

    written = doc.save_pages(tmp_path / "p.png")
    assert len(written) == len(pages)


def test_paginated_pdf_has_one_page_per_break():
    pdf = sone.sone(_long_document(), width=300, page_height=200).pdf()
    assert pdf[:4] == b"%PDF"
    assert pdf.count(b"/Type /Page\n") >= 2 or pdf.count(b"/Page") >= 2


# ── introspection ────────────────────────────────────────────────────────────


def test_layout_tree_reports_computed_boxes():
    tree = sone.sone(Column(Column().size(40, 10)).padding(5)).layout()
    assert tree["width"] == 50
    assert tree["height"] == 20
    assert tree["children"][0]["x"] == 5


def test_metadata_line_granularity_returns_text_boxes():
    meta = sone.sone(Text("hello world").font("GeistMono").size(12)).metadata("line")
    assert meta["segments"], meta
    assert meta["segments"][0]["text"] == "hello world"


def test_document_round_trips_through_json():
    doc = sone.sone(CARD, width=100).document()
    assert doc["sone"] == 1
    assert doc["config"] == {"width": 100}
    assert json.loads(sone.sone(CARD, width=100).json()) == doc


def test_unknown_config_key_is_rejected():
    with pytest.raises(TypeError, match="unknown render option"):
        sone.sone(CARD, nonsense=1).document()


# ── engines, fonts and assets ────────────────────────────────────────────────


def test_engine_isolates_fonts():
    engine = Engine(str(REPO))
    assert not engine.has_font("GeistMono")
    engine.register_font_file("GeistMono", str(FONTS / "GeistMono-Regular.ttf"))
    assert engine.has_font("GeistMono")
    assert "GeistMono" in engine.font_families()


def test_registered_image_is_reachable_as_an_asset():
    engine = Engine(str(REPO))
    engine.register_font_file("GeistMono", str(FONTS / "GeistMono-Regular.ttf"))
    png = sone.sone(Column().size(8, 8).bg("red")).png()
    engine.register_image("swatch", png)
    data = sone.sone(Column(Photo("asset:swatch").size(8, 8)), engine=engine).png()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"


def test_font_bytes_can_be_registered_directly():
    engine = Engine(str(REPO))
    engine.register_font("Geist", (FONTS / "GeistMono-Regular.ttf").read_bytes())
    assert engine.has_font("Geist")


# ── errors ───────────────────────────────────────────────────────────────────


def test_bad_document_raises_ir_error():
    engine = Engine(str(REPO))
    with pytest.raises(IrError):
        engine.render('{"sone": 99, "root": {"type": "column"}}', "png", None, 1.0, False)


def test_missing_image_raises_asset_error():
    with pytest.raises(AssetError):
        sone.sone(Column(Photo("does-not-exist.png"))).png()


def test_remote_assets_are_refused():
    with pytest.raises(AssetError, match="asset:"):
        sone.sone(Column(Photo("https://example.com/logo.svg"))).png()


def test_unknown_format_raises_value_error():
    with pytest.raises(ValueError, match="unknown output format"):
        sone.render(CARD, "tiff")


def test_error_hierarchy():
    assert issubclass(IrError, sone.SoneError)
    assert issubclass(AssetError, sone.SoneError)
    assert issubclass(sone.RenderError, sone.SoneError)


# ── parity with the CLI ──────────────────────────────────────────────────────


def test_matches_the_cli_byte_for_byte(tmp_path):
    """The Python API and the CLI must agree on the same document."""
    cli = REPO / "target" / "release" / "sone"
    if not cli.exists():
        pytest.skip("release CLI not built")

    engine = Engine(str(FONTS))
    engine.register_font_file("GeistMono", str(FONTS / "GeistMono-Regular.ttf"))
    node = Column(Text("parity").font("GeistMono").size(20).color("#222")).padding(12).bg("white")

    document = sone.sone(node, engine=engine).document()
    # The CLI resolves font paths against the document's directory, so give it
    # an absolute one.
    document["fonts"] = [{"name": "GeistMono", "src": str(FONTS / "GeistMono-Regular.ttf")}]

    doc_path = tmp_path / "doc.json"
    doc_path.write_text(json.dumps(document))
    out_path = tmp_path / "cli.png"
    subprocess.run(
        [str(cli), "render", str(doc_path), "-o", str(out_path), "--density", "2"],
        check=True,
        capture_output=True,
    )

    assert sone.sone(node, engine=engine).png(density=2) == out_path.read_bytes()
