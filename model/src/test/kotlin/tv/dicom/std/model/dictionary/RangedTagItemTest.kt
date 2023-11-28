package tv.dicom.std.model.dictionary

import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class RangedTagItemTest {

    @Test
    fun isRanged() {
        assertTrue(RangedTagItem(0x1234u, 0x5678u).isRanged())
        assertFalse(RangedTagItem(0x1234u).isRanged())
        assertFalse(RangedTagItem(0x1234u, 0x1234u).isRanged())
    }

    @Test
    fun of() {
        var rti = RangedTagItem.of("1234")
        assertNotNull(rti)
        assertFalse(rti.isRanged())
        rti = RangedTagItem.of("12x4")
        assertNotNull(rti)
        assertTrue(rti.isRanged())
        assertEquals(0x1204u, rti.min)
        assertEquals(0x12f4u, rti.max)
    }
}