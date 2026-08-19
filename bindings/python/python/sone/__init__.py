"""sone — a declarative canvas layout engine with rich international text.

Build a tree with the same fluent API as the TypeScript package, then render it:

    from sone import Column, Row, Text, sone, Font

    Font.load("Inter", "fonts/Inter-Regular.ttf")

    root = (
        Column(
            Text("Hello").size(28).weight("bold"),
            Row(
                Column().bg("salmon").size(50).rounded(14),
                Column().bg("orange").size(50).rounded(14),
            ).gap(10),
        )
        .gap(20)
        .padding(20)
        .bg("khaki")
        .cornerRadius(28)
    )

    sone(root).save("card.png", density=2)
"""

from __future__ import annotations

import json
import os
from typing import Any, Iterable

from ._engine import (
    AssetError,
    Engine,
    IrError,
    RenderError,
    SoneError,
    __version__,
)
from ._nodes import (
    ClipGroup,
    ClipGroupNode,
    Column,
    ColumnNode,
    Grid,
    GridNode,
    List,
    ListItem,
    ListItemNode,
    ListNode,
    Node,
    PageBreak,
    Path,
    PathNode,
    Photo,
    PhotoNode,
    Row,
    RowNode,
    Span,
    SpanNode,
    Table,
    TableCell,
    TableCellNode,
    TableNode,
    TableRow,
    TableRowNode,
    Text,
    TextDefault,
    TextDefaultNode,
    TextNode,
)

__all__ = [
    "ClipGroup", "Column", "Grid", "List", "ListItem", "PageBreak", "Path",
    "Photo", "Row", "Span", "Table", "TableCell", "TableRow", "Text",
    "TextDefault", "Node", "ClipGroupNode", "ColumnNode", "GridNode",
    "ListItemNode", "ListNode", "PathNode", "PhotoNode", "RowNode", "SpanNode",
    "TableCellNode", "TableNode", "TableRowNode", "TextDefaultNode", "TextNode",
    "sone", "render", "document", "Engine", "Font",
    "SoneError", "IrError", "AssetError", "RenderError", "__version__",
]

IR_VERSION = 1

_default_engine: Engine | None = None


def default_engine() -> Engine:
    """The process-wide engine used when no explicit one is passed."""
    global _default_engine
    if _default_engine is None:
        _default_engine = Engine(os.getcwd())
    return _default_engine


class Font:
    """Font registration on the default engine.

    Skia has no system fonts, so at least one family must be registered before
    rendering any text.
    """

    @staticmethod
    def load(name: str, source: "str | os.PathLike[str] | bytes | bytearray") -> None:
        """Register a family from a path or from raw TTF/OTF bytes."""
        engine = default_engine()
        if isinstance(source, (bytes, bytearray)):
            engine.register_font(name, bytes(source))
        else:
            engine.register_font_file(name, os.fspath(source))

    @staticmethod
    def has(name: str) -> bool:
        return default_engine().has_font(name)

    @staticmethod
    def families() -> list[str]:
        return default_engine().font_families()

    @staticmethod
    def reset() -> None:
        default_engine().reset_fonts()


_CONFIG_KEYS = {
    "width": "width",
    "height": "height",
    "background": "background",
    "density": "density",
    "page_height": "pageHeight",
    "pageHeight": "pageHeight",
    "margin": "margin",
    "last_page_height": "lastPageHeight",
    "lastPageHeight": "lastPageHeight",
    "header": "header",
    "footer": "footer",
}

_FORMAT_BY_SUFFIX = {
    ".png": "png", ".jpg": "jpg", ".jpeg": "jpg", ".webp": "webp",
    ".pdf": "pdf", ".svg": "svg", ".raw": "raw", ".rgba": "raw",
}


def document(node: Node, **config: Any) -> dict[str, Any]:
    """The IR document for `node`, ready to hand to any sone engine."""
    out: dict[str, Any] = {"sone": IR_VERSION}
    resolved: dict[str, Any] = {}
    for key, value in config.items():
        if value is None:
            continue
        name = _CONFIG_KEYS.get(key)
        if name is None:
            raise TypeError(f"unknown render option {key!r}")
        resolved[name] = value.to_ir() if isinstance(value, Node) else value
    if resolved:
        out["config"] = resolved
    out["root"] = node.to_ir()
    return out


class Rendered:
    """A node plus render config, with one method per output format."""

    __slots__ = ("_node", "_config", "_engine")

    def __init__(self, node: Node, config: dict[str, Any], engine: Engine | None) -> None:
        self._node = node
        self._config = config
        self._engine = engine

    # ── the document ─────────────────────────────────────────────────────────

    @property
    def engine(self) -> Engine:
        return self._engine or default_engine()

    def document(self) -> dict[str, Any]:
        """The IR document as a dict."""
        return document(self._node, **self._config)

    def json(self, indent: int | None = None) -> str:
        """The IR document as JSON."""
        return json.dumps(self.document(), indent=indent, ensure_ascii=False)

    # ── outputs ──────────────────────────────────────────────────────────────

    def _render(self, fmt: str, density: float | None, quality: float) -> bytes:
        return self.engine.render(self.json(), fmt, density, quality, False)

    def png(self, density: float | None = None) -> bytes:
        return self._render("png", density, 1.0)

    def jpg(self, quality: float = 1.0, density: float | None = None) -> bytes:
        return self._render("jpg", density, quality)

    jpeg = jpg

    def webp(self, quality: float = 1.0, density: float | None = None) -> bytes:
        return self._render("webp", density, quality)

    def raw(self, density: float | None = None) -> bytes:
        """Raw RGBA pixels, row-major, unpremultiplied."""
        return self._render("raw", density, 1.0)

    def svg(self) -> bytes:
        return self._render("svg", None, 1.0)

    def pdf(self) -> bytes:
        """A PDF. With `page_height` set, one page per break, text selectable."""
        return self._render("pdf", None, 1.0)

    def pages(self, format: str = "png", density: float | None = None, quality: float = 1.0) -> list[bytes]:
        """One raster image per page. Requires `page_height`."""
        return self.engine.render_pages(self.json(), format, density, quality, False)

    def save(self, path: "str | os.PathLike[str]", **kwargs: Any) -> str:
        """Render and write to `path`, inferring the format from its suffix."""
        target = os.fspath(path)
        suffix = os.path.splitext(target)[1].lower()
        fmt = _FORMAT_BY_SUFFIX.get(suffix)
        if fmt is None:
            raise ValueError(f"cannot infer an output format from {target!r}")
        data = getattr(self, "jpg" if fmt == "jpg" else fmt)(**kwargs)
        with open(target, "wb") as handle:
            handle.write(data)
        return target

    def save_pages(self, path: "str | os.PathLike[str]", **kwargs: Any) -> list[str]:
        """Write `name-1.png`, `name-2.png`, … next to `path`."""
        target = os.fspath(path)
        stem, suffix = os.path.splitext(target)
        fmt = _FORMAT_BY_SUFFIX.get(suffix.lower(), "png")
        written = []
        for index, data in enumerate(self.pages(fmt, **kwargs), start=1):
            name = f"{stem}-{index}{suffix}"
            with open(name, "wb") as handle:
                handle.write(data)
            written.append(name)
        return written

    # ── introspection ────────────────────────────────────────────────────────

    def layout(self) -> dict[str, Any]:
        """The computed layout tree."""
        return json.loads(self.engine.dump_layout(self.json()))

    def metadata(self, granularity: str = "node") -> dict[str, Any]:
        """Dataset-style boxes: `"node"`, `"line"` or `"word"`."""
        return json.loads(self.engine.dump_metadata(self.json(), granularity))


def sone(node: Node, *, engine: Engine | None = None, **config: Any) -> Rendered:
    """Wrap a node with render config; call a format method to get bytes."""
    return Rendered(node, config, engine)


def render(node: Node, format: str = "png", *, engine: Engine | None = None, **config: Any) -> bytes:
    """One-shot render to bytes."""
    density = config.pop("density", None)
    quality = config.pop("quality", 1.0)
    return Rendered(node, config, engine)._render(format, density, quality)
