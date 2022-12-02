package tv.dicom.std.core.parser

import org.apache.logging.log4j.kotlin.logger
import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeAll
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

class DicomStdParserKtTest {
    companion object {
        @JvmStatic
        private var emptyXmlString: String = ""

        @JvmStatic
        @BeforeAll
        internal fun beforeAll() {
            emptyXmlString = """
    <book xmlns="http://docbook.org/ns/docbook" xmlns:xl="http://www.w3.org/1999/xlink" label="PS3.3" version="5.0" xml:id="PS3.3">
    </book> 
            """.trimIndent()
        }
    }

    private val logger = logger()

    @Test
    fun parseXmlString() {
        val opt = parse(emptyXmlString)
        logger.info(emptyXmlString)
        assertTrue(opt.isEmpty)
    }

    @Test
    fun parsePart03SectA2() {
        val url = this::class.java.getResource("part_03_chapter_A.xml") ?: throw NullPointerException("Failed to obtain test resource (part_03_chapter_A.xml)")
        val file = File(url.toURI())
        val opt = parse(file)
        assertTrue(opt.isEmpty)
    }

    @Test
    fun buildCiod() {
        val url = this::class.java.getResource("part_03_table_A.2-1.xml") ?: throw NullPointerException("Failed to obtain test resource (part_03_table_A.2-1.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = buildCiod(root, root)
        assertTrue(opt.isPresent)
    }
}