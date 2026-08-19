# sone (Python)

A declarative canvas layout engine with rich international text, rendered by
Skia. Same fluent API as the [TypeScript package](https://github.com/seanghay/sone).

```python
from sone import Column, Row, Text, Font, sone

Font.load("Inter", "fonts/Inter-Regular.ttf")

root = (
    Column(
        Text("Hello").size(28).weight("bold"),
        Row(
            Column().bg("lightgreen").size(50).rounded(14),
            Column().bg("salmon").height(50).rounded(14).flex(1),
            Column().bg("orange").size(50).rounded(14),
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

sone(root).save("card.png", density=2)
```

Method names match the TypeScript API exactly; a snake_case alias exists for
each, so `.cornerRadius(20)` and `.corner_radius(20)` are the same call.

## Output formats

```python
page = sone(root, width=794, page_height=1123, margin=64)

page.png(density=2)      # bytes
page.jpg(quality=0.9)    # bytes
page.webp()              # bytes
page.svg()               # bytes, vector with live <text>
page.pdf()               # bytes, one page per break, text selectable
page.raw()               # unpremultiplied RGBA
page.pages()             # list[bytes], one raster per page
page.save("out.pdf")     # format inferred from the suffix
page.save_pages("p.png") # p-1.png, p-2.png, …

page.layout()            # computed layout tree, as a dict
page.metadata("line")    # dataset boxes: "node" | "line" | "word"
page.document()          # the IR document, as a dict
```

Headers and footers may contain the tokens `{pageNumber}` and `{totalPages}`,
substituted per page:

```python
sone(root,
     page_height=1123,
     header=Row(Text("Report")).padding(12),
     footer=Row(Text("{pageNumber} of {totalPages}")).padding(12)).save("report.pdf")
```

## Fonts and assets

Skia has no system fonts, so register at least one family before drawing text.
`Font` uses a process-wide engine; for isolation or for rendering on several
threads at once, create an `Engine` per thread:

```python
from sone import Engine, sone

engine = Engine(base_dir="assets")            # relative image paths resolve here
engine.register_font_file("Moul", "fonts/Moul-Regular.ttf")
engine.register_image("logo", logo_bytes)     # referenced as Photo("asset:logo")

sone(root, engine=engine).png()
```

`Photo` accepts a path, an `asset:<name>` handle, or raw bytes. Remote URLs are
never fetched during a render — download them yourself and register the bytes.

## Errors

`SoneError` is the base class; `IrError`, `AssetError` and `RenderError` are
raised for document, asset and rendering failures respectively.
