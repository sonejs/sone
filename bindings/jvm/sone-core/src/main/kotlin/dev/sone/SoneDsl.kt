@file:JvmName("SoneDsl")
@file:Suppress("FunctionName")

package dev.sone.dsl

import dev.sone.Bullets
import dev.sone.ClipGroup
import dev.sone.Column
import dev.sone.Grid
import dev.sone.ListItem
import dev.sone.Node
import dev.sone.Photo
import dev.sone.Rendering
import dev.sone.Sone
import dev.sone.Span
import dev.sone.SvgPath
import dev.sone.Table
import dev.sone.TableCell
import dev.sone.TableRow
import dev.sone.Text
import dev.sone.TextDefault

/**
 * A block DSL over the Java core.
 *
 * ```
 * import dev.sone.dsl.*
 *
 * val root = Column {
 *     gap(20.0); padding(20.0); size(420.0, 300.0); bg("khaki")
 *
 *     Column { flex(1.0); cornerRadius(20.0); cornerSmoothing(0.7); bg("white") }
 *
 *     Row {
 *         gap(10.0)
 *         Column { bg("lightgreen"); size(50.0); borderRadius(14.0) }
 *         Column { bg("salmon"); height(50.0); borderRadius(14.0); flex(1.0) }
 *     }
 * }
 *
 * Sone.render(root).density(2.0).save(Path.of("card.png"))
 * ```
 *
 * The core is Java rather than Kotlin, which is the reverse of what
 * `docs/bindings.md` first sketched. The reason is the same one the sketch
 * gave: DSL receivers do not translate to Java. Keeping the fluent surface in
 * Java means both languages get a first-class API, and Kotlin gets this layer
 * on top for free.
 *
 * The cost is that `@DslMarker` cannot be applied — it has to annotate the
 * receiver type, and these receivers are Java classes. A nested block can
 * therefore still reach the enclosing builder's methods by accident. Kotlin's
 * labelled `this` is the way to reach it on purpose: `this@Column.gap(4.0)`
 * from inside a nested `Row { }`.
 */

// ── roots ───────────────────────────────────────────────────────────────────

fun Column(block: Column.() -> Unit): Column = dev.sone.Column().apply(block)

fun Row(block: dev.sone.Row.() -> Unit): dev.sone.Row = dev.sone.Row().apply(block)

fun Grid(block: Grid.() -> Unit): Grid = dev.sone.Grid().apply(block)

fun TextDefault(block: TextDefault.() -> Unit): TextDefault = dev.sone.TextDefault().apply(block)

fun Table(block: Table.() -> Unit): Table = dev.sone.Table().apply(block)

fun TableRow(block: TableRow.() -> Unit): TableRow = dev.sone.TableRow().apply(block)

fun TableCell(block: TableCell.() -> Unit): TableCell = dev.sone.TableCell().apply(block)

fun Bullets(block: Bullets.() -> Unit): Bullets = dev.sone.Bullets().apply(block)

fun ListItem(block: ListItem.() -> Unit): ListItem = dev.sone.ListItem().apply(block)

fun Photo(src: String, block: Photo.() -> Unit = {}): Photo = dev.sone.Photo(src).apply(block)

fun SvgPath(d: String, block: SvgPath.() -> Unit = {}): SvgPath = dev.sone.SvgPath(d).apply(block)

fun ClipGroup(clipPath: String, block: ClipGroup.() -> Unit = {}): ClipGroup =
    dev.sone.ClipGroup(clipPath).apply(block)

fun Text(text: String? = null, block: Text.() -> Unit = {}): Text =
    (if (text == null) dev.sone.Text() else dev.sone.Text(text)).apply(block)

fun Span(text: String, block: Span.() -> Unit = {}): Span = dev.sone.Span(text).apply(block)

/** Wrap a node with render configuration. */
fun Node.render(): Rendering = Sone.render(this)

// ── children ────────────────────────────────────────────────────────────────
//
// Each appends to the receiver and returns the child, so a block reads as
// "what this box is, then what is in it".

fun Node.Column(block: Column.() -> Unit = {}): Column = adopt(dev.sone.Column().apply(block))

fun Node.Row(block: dev.sone.Row.() -> Unit = {}): dev.sone.Row =
    adopt(dev.sone.Row().apply(block))

fun Node.Grid(block: Grid.() -> Unit = {}): Grid = adopt(dev.sone.Grid().apply(block))

fun Node.Text(text: String? = null, block: Text.() -> Unit = {}): Text =
    adopt((if (text == null) dev.sone.Text() else dev.sone.Text(text)).apply(block))

fun Node.TextDefault(block: TextDefault.() -> Unit = {}): TextDefault =
    adopt(dev.sone.TextDefault().apply(block))

fun Node.Photo(src: String, block: Photo.() -> Unit = {}): Photo =
    adopt(dev.sone.Photo(src).apply(block))

fun Node.SvgPath(d: String, block: SvgPath.() -> Unit = {}): SvgPath =
    adopt(dev.sone.SvgPath(d).apply(block))

fun Node.Table(block: Table.() -> Unit = {}): Table = adopt(dev.sone.Table().apply(block))

fun Node.TableRow(block: TableRow.() -> Unit = {}): TableRow =
    adopt(dev.sone.TableRow().apply(block))

fun Node.TableCell(block: TableCell.() -> Unit = {}): TableCell =
    adopt(dev.sone.TableCell().apply(block))

fun Node.Bullets(block: Bullets.() -> Unit = {}): Bullets = adopt(dev.sone.Bullets().apply(block))

fun Node.ListItem(block: ListItem.() -> Unit = {}): ListItem =
    adopt(dev.sone.ListItem().apply(block))

fun Node.ClipGroup(clipPath: String, block: ClipGroup.() -> Unit = {}): ClipGroup =
    adopt(dev.sone.ClipGroup(clipPath).apply(block))

/** An explicit page break. Only meaningful with a page height set. */
fun Node.PageBreak(): Column = adopt(Sone.pageBreak())

/** Append an already-built subtree — the hook for helper functions. */
fun <T : Node> Node.adopt(child: T): T {
    children().add(child)
    return child
}

// ── paragraph content ───────────────────────────────────────────────────────

/** Append raw text to a `Text` or `Span`. */
fun Node.content(vararg items: String) {
    items.forEach { inline().add(it) }
}

/** Append a styled run. */
fun Node.Span(text: String, block: Span.() -> Unit = {}): Span {
    val span = dev.sone.Span(text).apply(block)
    inline().add(span)
    return span
}
