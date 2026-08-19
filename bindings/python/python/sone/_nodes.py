"""Fluent node builders.

A direct port of the TypeScript builder API in `src/core.ts`: the same factory
names, the same method names, the same chaining. Every method returns `self`, so
a tree reads exactly as it does in TypeScript.

Method names are camelCase to match the TypeScript API one for one. A snake_case
alias is generated for each, so `.cornerRadius(20)` and `.corner_radius(20)` are
the same call.
"""

from __future__ import annotations

import base64
import re
from typing import Any, Iterable, TypeVar

# Bound to Node so every fluent method infers its own type back to the caller.
_N = TypeVar("_N", bound="Node")

_MISSING = object()


def _encode(value: Any) -> Any:
    if isinstance(value, Node):
        return value.to_ir()
    if isinstance(value, (list, tuple)):
        return [_encode(v) for v in value]
    return value


class Node:
    """A node in the document tree. Build with the factories below."""

    __slots__ = ("type", "props", "children", "inline")

    def __init__(self, node_type: str, *children: "Node | None") -> None:
        self.type = node_type
        self.props: dict[str, Any] = {}
        self.children: list[Node] = [c for c in children if c is not None]
        self.inline: list[str | Node] = []

    # ── serialization ────────────────────────────────────────────────────────

    def to_ir(self) -> dict[str, Any]:
        """This node as an IR document fragment."""
        out: dict[str, Any] = {"type": self.type}
        if self.props:
            out["props"] = {k: _encode(v) for k, v in self.props.items()}
        if self.children:
            out["children"] = [c.to_ir() for c in self.children]
        if self.inline:
            out["inline"] = [i if isinstance(i, str) else i.to_ir() for i in self.inline]
        return out

    def __repr__(self) -> str:
        tag = self.props.get("tag")
        label = f" {tag!r}" if tag else ""
        return f"<{self.type}{label} props={len(self.props)} children={len(self.children)}>"

    # ── internals ────────────────────────────────────────────────────────────

    def _set(self: _N, **kwargs: Any) -> _N:
        for key, value in kwargs.items():
            if value is not None:
                self.props[key] = value
        return self

    def _push(self: _N, key: str, values: Iterable[Any]) -> _N:
        self.props.setdefault(key, []).extend(values)
        return self

    def _box(
        self: _N,
        keys: tuple[str, str, str, str, str],
        top,
        right=_MISSING,
        bottom=_MISSING,
        left=_MISSING,
    ) -> _N:
        """CSS 1–4 value shorthand: [top, right, bottom ?? top, left ?? right]."""
        if right is _MISSING and bottom is _MISSING and left is _MISSING:
            self.props[keys[0]] = top
            return self
        self.props[keys[1]] = top
        self.props[keys[2]] = right
        self.props[keys[3]] = top if bottom is _MISSING else bottom
        self.props[keys[4]] = right if left is _MISSING else left
        return self


_BORDER = ("borderWidth", "borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth")
_MARGIN = ("margin", "marginTop", "marginRight", "marginBottom", "marginLeft")
_PADDING = ("padding", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft")


class _Layout(Node):
    """Flexbox, sizing, spacing, and the visual box properties."""

    def tag(self: _N, value) -> _N: return self._set(tag=value)
    def apply(self: _N, values: dict) -> _N: self.props.update(values); return self

    # flexbox
    def alignContent(self, value): return self._set(alignContent=value)
    def alignItems(self, value): return self._set(alignItems=value)
    def alignSelf(self, value): return self._set(alignSelf=value)
    def aspectRatio(self, value): return self._set(aspectRatio=value)
    def boxSizing(self, value): return self._set(boxSizing=value)
    def direction(self, value): return self._set(direction=value)
    def display(self, value): return self._set(display=value)
    def flex(self, value): return self._set(flex=value)
    def basis(self, value): return self._set(flexBasis=value)
    def flexDirection(self, value): return self._set(flexDirection=value)
    def grow(self, value): return self._set(flexGrow=value)
    def shrink(self, value): return self._set(flexShrink=value)
    def wrap(self, value): return self._set(flexWrap=value)
    def justifyContent(self, value): return self._set(justifyContent=value)
    def overflow(self, value): return self._set(overflow=value)
    def position(self, value): return self._set(position=value)

    def gap(self, value): return self._set(gap=value)
    def rowGap(self, value): return self._set(rowGap=value)
    def columnGap(self, value): return self._set(columnGap=value)

    # sizing
    def size(self, width, height=None):
        return self._set(width=width, height=width if height is None else height)

    def width(self, value): return self._set(width=value)
    def height(self, value): return self._set(height=value)
    def minWidth(self, value): return self._set(minWidth=value)
    def minHeight(self, value): return self._set(minHeight=value)
    def maxWidth(self, value): return self._set(maxWidth=value)
    def maxHeight(self, value): return self._set(maxHeight=value)

    # box edges
    def borderWidth(self, top, right=_MISSING, bottom=_MISSING, left=_MISSING):
        return self._box(_BORDER, top, right, bottom, left)

    def borderColor(self, value): return self._set(borderColor=value)

    def margin(self, top, right=_MISSING, bottom=_MISSING, left=_MISSING):
        return self._box(_MARGIN, top, right, bottom, left)

    def marginTop(self, value): return self._set(marginTop=value)
    def marginRight(self, value): return self._set(marginRight=value)
    def marginBottom(self, value): return self._set(marginBottom=value)
    def marginLeft(self, value): return self._set(marginLeft=value)

    def padding(self, top, right=_MISSING, bottom=_MISSING, left=_MISSING):
        return self._box(_PADDING, top, right, bottom, left)

    def paddingTop(self, value): return self._set(paddingTop=value)
    def paddingRight(self, value): return self._set(paddingRight=value)
    def paddingBottom(self, value): return self._set(paddingBottom=value)
    def paddingLeft(self, value): return self._set(paddingLeft=value)

    # insets
    def top(self, value): return self._set(top=value)
    def right(self, value): return self._set(right=value)
    def bottom(self, value): return self._set(bottom=value)
    def left(self, value): return self._set(left=value)
    def start(self, value): return self._set(start=value)
    def end(self, value): return self._set(end=value)
    def inset(self, value): return self._set(inset=value)

    # grid placement
    def gridColumn(self, start, span=None): return self._set(gridColumnStart=start, gridColumnSpan=span)
    def gridRow(self, start, span=None): return self._set(gridRowStart=start, gridRowSpan=span)

    # pagination
    def pageBreak(self, value): return self._set(pageBreak=value)

    # transforms
    def translateX(self, value): return self._set(translateX=value)
    def translateY(self, value): return self._set(translateY=value)
    def rotate(self, value): return self._set(rotation=value)
    def scale(self, x, y=None): return self._set(scale=[x, x if y is None else y])

    # paint
    def bg(self, *values): return self._push("background", values)
    def background(self, *values): return self._push("background", values)
    def opacity(self, value): return self._set(opacity=value)
    def cornerRadius(self, *values): return self._set(cornerRadius=list(values))
    def rounded(self, *values): return self._set(cornerRadius=list(values))
    def borderRadius(self, *values): return self._set(cornerRadius=list(values))
    def cornerSmoothing(self, value): return self._set(cornerSmoothing=value)
    def borderSmoothing(self, value): return self._set(cornerSmoothing=value)
    def corner(self, value): return self._set(corner=value)
    def shadow(self, *values): return self._push("shadows", values)

    # CSS filters, in the order they are applied
    def blur(self, value): return self._push("filters", [f"blur({value}px)"])
    def brightness(self, value): return self._push("filters", [f"brightness({value})"])
    def contrast(self, value): return self._push("filters", [f"contrast({value})"])
    def grayscale(self, value): return self._push("filters", [f"grayscale({value})"])
    def hueRotate(self, value): return self._push("filters", [f"hue-rotate({value})"])
    def invert(self, value): return self._push("filters", [f"invert({value})"])
    def saturate(self, value): return self._push("filters", [f"saturate({value})"])
    def sepia(self, value): return self._push("filters", [f"sepia({value})"])


class _SpanStyle(Node):
    """Text styling shared by `Text`, `Span` and `TextDefault`."""

    def tag(self, value): return self._set(tag=value)
    def color(self, value): return self._set(color=value)
    def size(self, value): return self._set(size=value)
    def font(self, *values): return self._set(font=list(values))
    def style(self, value): return self._set(style=value)
    def weight(self, value): return self._set(weight=value)
    def letterSpacing(self, value): return self._set(letterSpacing=value)
    def wordSpacing(self, value): return self._set(wordSpacing=value)

    def underline(self, value=1.0): return self._set(underline=value)
    def underlineColor(self: _N, value=None) -> _N: self.props["underlineColor"] = value; return self
    def overline(self, value=1.0): return self._set(overline=value)
    def overlineColor(self: _N, value=None) -> _N: self.props["overlineColor"] = value; return self
    def lineThrough(self, value=1.0): return self._set(lineThrough=value)
    def lineThroughColor(self: _N, value=None) -> _N: self.props["lineThroughColor"] = value; return self
    def highlight(self: _N, value=None) -> _N: self.props["highlightColor"] = value; return self

    def dropShadow(self, *values): return self._push("dropShadows", values)
    def strokeColor(self, value): return self._set(strokeColor=value)
    def strokeWidth(self, value): return self._set(strokeWidth=value)
    def offsetY(self, value): return self._set(offsetY=value)
    def textDir(self, value): return self._set(textDir=value)


class _TextBlock(Node):
    """Paragraph-level properties. `Text` and `TextDefault` share these."""

    def nowrap(self): return self._set(nowrap=True)
    def wrap(self, value=True): return self._set(nowrap=not value)
    def maxLines(self, value): return self._set(maxLines=value)
    def lineBreak(self, value): return self._set(lineBreak=value)
    def textOverflow(self, value): return self._set(textOverflow=value)
    def lineHeight(self, value): return self._set(lineHeight=value)
    def align(self, value): return self._set(align=value)
    def indent(self, value): return self._set(indentSize=value)
    def hangingIndent(self, value): return self._set(hangingIndentSize=value)
    def tabStops(self, *values): return self._set(tabStops=list(values))
    def tabLeader(self, value): return self._set(tabLeader=value)
    def autofit(self, value=True): return self._set(autofit=value)
    def orientation(self, value): return self._set(orientation=value)
    def clipImage(self, value): return self._set(clipImage=value)
    def baseDir(self, value): return self._set(baseDir=value)
    def textWrap(self, value): return self._set(textWrap=value)


# ── concrete node types ──────────────────────────────────────────────────────


class ColumnNode(_Layout):
    pass


class RowNode(_Layout):
    pass


class GridNode(_Layout):
    def columns(self, *values): return self._set(columns=list(values))
    def rows(self, *values): return self._set(rows=list(values))
    def autoRows(self, *values): return self._set(autoRows=list(values))
    def autoColumns(self, *values): return self._set(autoColumns=list(values))


class SpanNode(_SpanStyle):
    pass


class TextNode(_SpanStyle, _TextBlock, _Layout):
    """Both a box and a paragraph.

    `size()` sets the font size here, not the box size — matching the
    TypeScript API, where `TextPropsBuilder` omits the layout `size`.
    """

    size = _SpanStyle.size
    wrap = _TextBlock.wrap
    tag = _SpanStyle.tag


class TextDefaultNode(_SpanStyle, _TextBlock):
    """Cascades text styling onto its children without drawing a box."""


class PhotoNode(_Layout):
    def scaleType(self, value, alignment=None):
        keywords = {"start": 0.0, "center": 0.5, "end": 1.0}
        if isinstance(alignment, str):
            alignment = keywords[alignment]
        return self._set(scaleType=value, scaleAlignment=alignment)

    def preserveAspectRatio(self, value=True): return self._set(preserveAspectRatio=value)
    def flipHorizontal(self, value=True): return self._set(flipHorizontal=value)
    def flipVertical(self, value=True): return self._set(flipVertical=value)
    def fill(self, value): return self._set(fill=value)
    def clipPath(self, value): return self._set(clipPath=value)


class PathNode(_Layout):
    def stroke(self, value): return self._set(stroke=value)
    def strokeWidth(self, value): return self._set(strokeWidth=value)
    def strokeLineCap(self, value): return self._set(strokeLineCap=value)
    def strokeLineJoin(self, value): return self._set(strokeLineJoin=value)
    def strokeMiterLimit(self, value): return self._set(strokeMiterLimit=value)
    def strokeDashArray(self, *values): return self._set(strokeDashArray=list(values))
    def strokeDashOffset(self, value): return self._set(strokeDashOffset=value)
    def fill(self, value): return self._set(fill=value)
    def fillOpacity(self, value): return self._set(fillOpacity=value)
    def fillRule(self, value): return self._set(fillRule=value)
    def scalePath(self, value): return self._set(scalePath=value)


class TableNode(_Layout):
    def spacing(self, *values): return self._set(spacing=list(values))


class TableRowNode(_Layout):
    pass


class TableCellNode(_Layout):
    def colspan(self, value): return self._set(colspan=value)
    def rowspan(self, value): return self._set(rowspan=value)


class ListNode(_Layout):
    def listStyle(self, value): return self._set(listStyle=value)
    def markerGap(self, value): return self._set(markerGap=value)
    def markerOffset(self, value): return self._set(markerOffset=value)
    def startIndex(self, value): return self._set(startIndex=value)


class ListItemNode(_Layout):
    def marker(self, value):
        """Use a specific marker for this item, overriding the list style."""
        return self._set(marker=value)


class ClipGroupNode(_Layout):
    def clipPath(self, value): return self._set(clipPath=value)


# ── factories ────────────────────────────────────────────────────────────────


def Column(*children) -> ColumnNode:
    """A vertical container."""
    return ColumnNode("column", *children)


def Row(*children) -> RowNode:
    """A horizontal container."""
    return RowNode("row", *children)


def Grid(*children) -> GridNode:
    """A grid container with row-major auto placement."""
    return GridNode("grid", *children)


def Span(text: str) -> SpanNode:
    """A styled run inside a `Text`."""
    node = SpanNode("span")
    node.inline = [text]
    return node


def Text(*children) -> TextNode:
    """A paragraph of strings and `Span`s."""
    node = TextNode("text")
    node.inline = [c for c in children if c is not None]
    return node


def TextDefault(*children) -> TextDefaultNode:
    """Cascade text styling onto every descendant."""
    return TextDefaultNode("text-default", *children)


def PageBreak() -> ColumnNode:
    """An explicit page break. Only meaningful with `page_height` set."""
    return Column().height(0).pageBreak("before")


_DATA_URL = re.compile(r"^[a-z][a-z0-9+.-]*:", re.I)


def Photo(src: "str | bytes | bytearray") -> PhotoNode:
    """An image, from a path, a URL, `asset:<name>`, or raw bytes."""
    node = PhotoNode("photo")
    if isinstance(src, (bytes, bytearray)):
        encoded = base64.b64encode(bytes(src)).decode("ascii")
        src = f"data:application/octet-stream;base64,{encoded}"
    return node._set(src=src)


def Path(d: str) -> PathNode:
    """An SVG path."""
    return PathNode("path")._set(d=d)


def Table(*children) -> TableNode:
    return TableNode("table", *children)


def TableRow(*children) -> TableRowNode:
    return TableRowNode("table-row", *children)


def TableCell(*children) -> TableCellNode:
    return TableCellNode("table-cell", *children)


def List(*children) -> ListNode:
    return ListNode("list", *children)


def ListItem(*children) -> ListItemNode:
    return ListItemNode("list-item", *children)


def ClipGroup(path: str, *children) -> ClipGroupNode:
    """Clip every child to an SVG path."""
    return ClipGroupNode("clip-group", *children)._set(clipPath=path)


# ── snake_case aliases ───────────────────────────────────────────────────────

_CAMEL = re.compile(r"(?<!^)(?=[A-Z])")


def _add_snake_case_aliases() -> None:
    for cls in (
        _Layout, _SpanStyle, _TextBlock, ColumnNode, RowNode, GridNode, SpanNode,
        TextNode, TextDefaultNode, PhotoNode, PathNode, TableNode, TableRowNode,
        TableCellNode, ListNode, ListItemNode, ClipGroupNode,
    ):
        for name, member in list(vars(cls).items()):
            if name.startswith("_") or not callable(member):
                continue
            snake = _CAMEL.sub("_", name).lower()
            if snake != name and not hasattr(cls, snake):
                setattr(cls, snake, member)


_add_snake_case_aliases()
