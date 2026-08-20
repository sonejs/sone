# sone for the JVM

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

One artifact serves Java and Kotlin.

## Java

```java
import dev.sone.*;
import java.nio.file.Path;

Font.load("Inter", "fonts/Inter-Regular.ttf");

Column root = new Column(
        new Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white"),
        new Row(
                new Column().bg("lightgreen").size(50).borderRadius(14),
                new Column().bg("salmon").height(50).borderRadius(14).flex(1))
                .gap(10))
        .gap(20).padding(20).size(420, 300).bg("khaki");

Sone.render(root).density(2).save(Path.of("card.png"));
```

Chaining off `new` needs no extra parentheses, which is why the constructors
take children.

## Kotlin

```kotlin
import dev.sone.Sone
import dev.sone.dsl.*
import java.nio.file.Path

val root = Column {
    gap(20.0); padding(20.0); size(420.0, 300.0); bg("khaki")

    Column { flex(1.0); cornerRadius(20.0); cornerSmoothing(0.7); bg("white") }

    Row {
        gap(10.0)
        Column { bg("lightgreen"); size(50.0); borderRadius(14.0) }
        Column { bg("salmon"); height(50.0); borderRadius(14.0); flex(1.0) }
    }
}

Sone.render(root).density(2.0).save(Path.of("card.png"))
```

Loops and conditionals are just Kotlin inside the block:

```kotlin
Table {
    records.forEach { record ->
        TableRow { record.cells.forEach { cell -> TableCell { Text(cell) } } }
    }
}
```

## The shape of the API

**Self-typed interfaces with default methods.** Java has single inheritance and
`Text` has to be a box, a styled run and a paragraph at once, so the properties
live on `LayoutProps<SELF>`, `SpanStyleProps<SELF>` and `TextBlockProps<SELF>`.
`SELF` is what keeps `new Column().gap(20).padding(20)` typed as a `Column`.

Two of those interfaces declare `size(double)`, so **the compiler forces `Text`
to override it** — which is a convenient place to write down that `Text.size()`
is the font size, matching the TypeScript API. Use `width()` and `height()` for
the box.

**`Dim` for the length union.** Java has no implicit user conversions, so
`width(100)` takes the number overload and `width(Dim.percent(50))` or
`width(Dim.AUTO)` take the object one.

**Arity overloads stand in for named arguments.** `padding(20)`,
`padding(10, 20)`, `padding(8, 8, 8, 4)`.

**Keywords are enums**: `JustifyContent.SPACE_BETWEEN`.

## Two names that had to move

`List` and `Path` are `java.util.List` and `java.nio.file.Path`, so the list is
**`Bullets`** and the path is **`SvgPath`**.

## Engine and output

```java
try (Engine engine = new Engine("assets")) {
    engine.registerFont("Inter", Files.readAllBytes(Path.of("Inter-Regular.ttf")));
    engine.registerImage("logo", pngBytes);        // reachable as asset:logo

    Rendering rendering = Sone.render(root).engine(engine)
            .width(816).pageHeight(1056)
            .header(new Text("Page {pageNumber} of {totalPages}"));

    rendering.pdf();                               // selectable text, one page per break
    rendering.pages();                             // one raster image per page
    rendering.save(Path.of("report.pdf"));
    rendering.savePages(Path.of("page.png"));      // page-1.png, page-2.png, ...
    rendering.layoutJson();                        // the computed layout tree
    rendering.metadataJson(Granularity.WORD);
    rendering.toJson();                            // the IR itself
}
```

`layoutJson()` and `metadataJson()` hand back the engine's JSON as a `String`
rather than a parsed tree, so this artifact does not hand every consumer a
Jackson or Gson version to reconcile.

Skia carries no system fonts, so at least one family must be registered before
any text renders. Header and footer text uses the literal tokens `{pageNumber}`
and `{totalPages}`; the engine substitutes them.

**One engine per thread.** Skia's font collection is shared inside an engine, so
every call is synchronized. Give each thread its own `Engine` for real
parallelism rather than sharing one.

Failures arrive as `IrException`, `AssetException` or `RenderException`, all
under `SoneException`.

## Modules, and Android

| artifact | what it is | floor |
|---|---|---|
| `sone-core` | the builder, the property interfaces, the IR writer. No native code. | Java 17 |
| `sone-panama` | the desktop backend, over `java.lang.foreign` | Java 22 |
| `sone-jna` | the Android backend, over JNA | Java 17 |

`Engine` picks a backend at runtime through `dev.sone.Backends`: Panama where it
exists, JNA otherwise. Nothing above `dev.sone.Backend` knows which one is
underneath, so the same tree, the same fluent API and the same Kotlin DSL serve
both.

**On a desktop JVM** take `sone-core` + `sone-panama`. Java 22 is the floor
because that is where Panama is final; add `--enable-native-access=ALL-UNNAMED`
to silence the runtime warning.

**On Android** take `sone-core` + `sone-jna`. Android has no
`java.lang.foreign` at any version — ART does not implement Panama, and it
cannot be desugared because it needs VM support — so JNA reaches the same C ABI
instead. Its per-call overhead is irrelevant here: one render is one call
carrying a whole document, and the marshalling sits next to a Skia
rasterization.

Both artifacts dex at `minSdk 21`. Put the native libraries in
`src/main/jniLibs/<abi>/libsone.so`:

```bash
tools/build-android.sh app/src/main/jniLibs
```

arm64-v8a and x86_64 only — rust-skia publishes no 32-bit Android binary.

The JNA backend is not Android-only; it runs on a desktop JVM too, which is how
the Android call path is tested without a device. `BackendTest` asserts the two
backends produce **byte-identical** PNGs and identical layout JSON.

**One process loads one backend.** On Windows, mapping the same DLL through both
Panama and JNA in a single JVM crashes — two loaders, two module handles, two
copies of Skia's global state. That is not a configuration anything ships: a
desktop app takes `sone-panama`, an Android app takes `sone-jna`. CI proves each
one separately on Windows by pinning `-Dsone.backend=…` per JVM, and
`BackendTest`'s cross-backend comparison skips there rather than pretending to
cover it.

## Installing

Panama is bound by hand rather than through `jextract` — sixteen functions and
three structs is less code than the generator's plumbing, and it keeps the build
to a plain `mvn package` with no extra tool on the path.

**The native library is not in the artifact yet** — build it from a checkout:

```bash
cargo build --release -p sone-ffi
```

The binding finds it by walking up to the checkout root. Anywhere else, set
`SONE_NATIVE_LIBRARY` to the file or the directory holding it.

## Development

```bash
cd bindings/jvm
mvn test
```

50 tests: 31 Java, 6 Kotlin, and 13 that run the engine through both backends.
They include the parity gate every sone binding owes — the same document
rendered through this binding and through `sone-cli` must come out byte for byte
identical — and the cross-backend gate, which is the one that keeps Android
honest.

## Why the core is Java

`docs/bindings.md` first sketched the reverse — a Kotlin core with a Java-facing
facade. The reason it gave holds, and it is why this is the other way round: DSL
receivers do not translate to Java at all. Keeping the fluent surface in Java
means both languages get a first-class API, and Kotlin gets `dev.sone.dsl` on
top of it for free.

The cost is that `@DslMarker` cannot be applied, because it has to annotate the
receiver type and these receivers are Java classes. A nested block can therefore
still reach the enclosing builder by accident; Kotlin's labelled `this` is how
you reach it on purpose:

```kotlin
Column {
    gap(4.0)
    Row { this@Column.padding(2.0) }
}
```
