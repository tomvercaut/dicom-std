package tv.dicom.std.core.parser

import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeAll
import java.io.File

class DicomStdParserKtTest {
    companion object {
        @JvmStatic
        private var emptyXmlString: String = ""

        @JvmStatic
        @BeforeAll
        internal fun beforeAll(): Unit {
            emptyXmlString = """
    <book xmlns="http://docbook.org/ns/docbook" xmlns:xl="http://www.w3.org/1999/xlink" label="PS3.3" version="5.0" xml:id="PS3.3">
    </book> 
            """.trimIndent()
        }
    }

    @Test
    fun parseXmlString() {
        val opt = parse(emptyXmlString)
        assertTrue(opt.isEmpty)
    }

    @Test
    fun parsePart03SectA2() {
        val url = this::class.java.getResource("part_03_chapter_A.xml")
        val file =File(url.toURI())
        val opt = parse(file)
        assertTrue(opt.isEmpty)
    }
}