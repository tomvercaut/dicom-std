package tv.dicom.std.model

import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*

class TagTest {

    @Test
    fun toUInt() {
        assertEquals(269488692U, Tag(0x1010, 0x1234).toUInt())
    }

    @Test
    fun of() {
        assertEquals(Tag(0x1010, 0x1234), Tag.of("(1010,1234)"))
        assertEquals(Tag(0x0110, 0x4231), Tag.of("(0110,4231)"))
        assertEquals(Tag(0x011F, 0x42F1), Tag.of("(011F,42F1)"))
        assertEquals(null, Tag.of("0101,4231)"))
        assertEquals(null, Tag.of("(0101,4231"))
        assertEquals(null, Tag.of("(0101 4231)"))
        assertEquals(Tag(0x1010, 0x1234), Tag.of("(0x1010,0x1234)"))
        assertEquals(Tag(0x0110, 0x4231), Tag.of("(0x0110,0x4231)"))
        assertEquals(Tag(0x011F, 0x42F1), Tag.of("(0x011F,0x42F1)"))
    }
}