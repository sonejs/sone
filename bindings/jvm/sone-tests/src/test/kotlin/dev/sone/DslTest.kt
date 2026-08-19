package dev.sone

import dev.sone.dsl.Bullets
import dev.sone.dsl.Column
import dev.sone.dsl.ListItem
import dev.sone.dsl.Row
import dev.sone.dsl.Span
import dev.sone.dsl.Table
import dev.sone.dsl.TableCell
import dev.sone.dsl.TableRow
import dev.sone.dsl.Text
import dev.sone.dsl.render
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import org.junit.jupiter.api.Test

/** The Kotlin layer: the same core, reached through blocks instead of chaining. */
class DslTest {

    @Test
    fun `a block reads as the document it builds`() {
        val root = Column {
            gap(20.0)
            padding(20.0)
            size(420.0, 300.0)
            bg("khaki")
            cornerRadius(28.0)

            Column { flex(1.0); cornerRadius(20.0); cornerSmoothing(0.7); bg("white") }

            Row {
                gap(10.0)
                Column { bg("lightgreen"); size(50.0); borderRadius(14.0) }
                Column { bg("salmon"); height(50.0); borderRadius(14.0); flex(1.0) }
            }
        }

        assertEquals(2, root.children().size)
        assertEquals("row", root.children()[1].type())
        assertEquals(2, root.children()[1].children().size)
        assertContains(root.toJson(), "\"gap\":20")
    }

    @Test
    fun `loops generate children`() {
        val cells = listOf("a", "b", "c")
        val table = Table {
            TableRow {
                cells.forEach { cell -> TableCell { Text(cell) } }
            }
        }
        assertEquals(3, table.children()[0].children().size)
    }

    @Test
    fun `a labelled this reaches past a nested block`() {
        // No @DslMarker is possible over Java receivers, so a labelled `this`
        // is how you reach the outer builder on purpose rather than by accident.
        val root = Column {
            gap(4.0)
            Row {
                gap(8.0)
                this@Column.padding(2.0)
            }
        }
        assertContains(root.toJson(), "\"gap\":4")
        assertContains(root.toJson(), "\"padding\":2")
    }

    @Test
    fun `spans nest inside a paragraph`() {
        val text = Text("Hello ") {
            font("Inter")
            size(28.0)
            Span("world") { weight("bold"); color("salmon") }
        }
        val json = text.toJson()
        assertContains(json, "\"inline\":[\"Hello \",{\"type\":\"span\"")
        assertContains(json, "\"size\":28")
    }

    @Test
    fun `lists build from a collection`() {
        val list = Bullets {
            listStyle("disc")
            markerGap(8.0)
            listOf("one", "two").forEach { item -> ListItem { Text(item) } }
        }
        assertEquals(2, list.children().size)
    }

    @Test
    fun `render is reachable as an extension`() {
        val json = Column { size(16.0) }.render().density(2.0).toJson()
        assertTrue(json.startsWith("{\"sone\":1"))
        assertContains(json, "\"density\":2")
    }
}
