package tv.dicom.std.core.model.dictionary

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test

class RangedTagTest {

    @Test
    fun isRanged() {
        var rt: RangedTag? = RangedTag(RangedTagItem(0x1234u), RangedTagItem(0x5678u))
        kotlin.test.assertNotNull(rt)
        assertFalse(rt.isRanged())
        rt = RangedTag.of("(1234,5678)")
        kotlin.test.assertNotNull(rt)
        assertFalse(rt.isRanged())

        rt = RangedTag.of("(12x4,5678)")
        kotlin.test.assertNotNull(rt)
        assertTrue(rt.isRanged())

        rt = RangedTag.of("(1234,5x78)")
        kotlin.test.assertNotNull(rt)
        assertTrue(rt.isRanged())
    }

    @Test
    fun of() {
        var rt: RangedTag? = RangedTag(RangedTagItem(0x1234u), RangedTagItem(0x5678u))
        kotlin.test.assertNotNull(rt)

        assertEquals(rt.group, RangedTagItem(0x1234u))
        assertEquals(rt.element, RangedTagItem(0x5678u))

        rt = RangedTag.of("(12x4,5678)")
        kotlin.test.assertNotNull(rt)
        kotlin.test.assertNotNull(rt.group)
        assertEquals(rt.group, RangedTagItem.of("12x4"))
        assertEquals(rt.element, RangedTagItem(0x5678u))

        rt = RangedTag.of("(1234,5x78)")
        kotlin.test.assertNotNull(rt)
        kotlin.test.assertNotNull(rt.element)
        assertEquals(rt.group, RangedTagItem(0x1234u))
        assertEquals(rt.element, RangedTagItem.of("5x78"))
    }
}