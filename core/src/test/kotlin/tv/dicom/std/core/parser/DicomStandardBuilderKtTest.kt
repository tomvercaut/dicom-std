package tv.dicom.std.core.parser

import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*
import tv.dicom.std.core.model.*
import tv.dicom.std.core.model.imd.DataEntry
import tv.dicom.std.core.model.imd.Imd
import tv.dicom.std.core.model.imd.IncludeEntry
import tv.dicom.std.core.parser.TestHelper.Companion.tableA_2_1
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

class DicomStandardBuilderKtTest {

    @Test
    fun buildCiod() {
        val url = this::class.java.getResource("part_03_table_A.2-1.xml")
            ?: throw NullPointerException("Failed to obtain test resource (part_03_table_A.2-1.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val expected = tableA_2_1()

        val opt = buildCiod(root)
        assertTrue(opt.isPresent)
        val ciod = opt.get()

        assertEquals(expected.id, ciod.id)
        assertEquals(expected.parentIds, ciod.parentIds)
        assertEquals(expected.items.size, ciod.items.size)
        for (i in expected.items.indices) {
            assertEquals(expected.items[i], ciod.items[i])
        }
    }

    @Test
    fun buildCiodEntry() {
        val url = this::class.java.getResource("build_ciod_entry.xml")
            ?: throw NullPointerException("Failed to obtain test resource (build_ciod_entry.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = buildCiodEntry(root)
        assertTrue(opt.isPresent)
        val entry = opt.get()
        assertEquals("Study", entry.ie)
        assertEquals("General Study", entry.module)
        assertEquals(XRef("sect_C.7.2.1", "select: labelnumber"), entry.reference)
        assertEquals(Usage.M, entry.usage)
    }

    @Test
    fun xref() {
        val url = this::class.java.getResource("para_xref.xml")
            ?: throw NullPointerException("Failed to obtain test resource (para_xref.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val xr = xref(root)
        assertEquals("sect_C.7.1.1", xr.linkend)
        assertEquals("select: labelnumber", xr.style)
    }

    @Test
    fun sequenceItemDepth() {
        assertEquals(0.toUShort(), sequenceItemDepth("abc"))
        assertEquals(0.toUShort(), sequenceItemDepth(" abc"))
        assertEquals(0.toUShort(), sequenceItemDepth(" \n\r abc"))
        assertEquals(1.toUShort(), sequenceItemDepth(">abc"))
        assertEquals(2.toUShort(), sequenceItemDepth(">>abc"))
        assertEquals(2.toUShort(), sequenceItemDepth(" >>abc"))
        assertEquals(2.toUShort(), sequenceItemDepth(" \r\n >>abc"))
    }

    @Test
    fun attributeHasInclude() {
        assertTrue(attributeHasInclude("Include"))
        assertFalse(attributeHasInclude("include"))
        assertTrue(attributeHasInclude(">Include"))
        assertTrue(attributeHasInclude(">>Include"))
        assertTrue(attributeHasInclude(" >>Include"))
        assertTrue(attributeHasInclude(" \n\r >>Include"))
        assertFalse(attributeHasInclude(" \n\r >>include"))
    }

    @Test
    fun getParentXmlId() {

        val url = this::class.java.getResource("para_xref.xml")
            ?: throw NullPointerException("Failed to obtain test resource (para_xref.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = findElement(root, "//xref")
        assertTrue(opt.isPresent)
        val xref = opt.get()
        val pids = getParentXmlId(xref)
        val expected = mutableListOf<String>()
        expected.add("para_a3b139fa-41a0-4df8-808c-38cb364c850d")
        expected.add("PS3.3")
        assertEquals(expected, pids)
    }

    @Test
    fun attributeName() {
        assertEquals("Referenced Study Sequence", attributeName("Referenced Study Sequence"))
        assertEquals("Referenced Study Sequence", attributeName(" >\n\r> Referenced Study Sequence"))
    }

    @Test
    fun buildImdEntry1() {
        val url = this::class.java.getResource("build_imd_entry1.xml")
            ?: throw NullPointerException("Failed to obtain test resource (build_imd_entry1.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = buildImdEntry(root)
        assertTrue(opt.isPresent)
        val entry = opt.get() as DataEntry
        assertTrue(entry.isData())
        assertFalse(entry.isInclude())
        assertEquals("Referenced Study Sequence", entry.name)
        assertNotNull(entry.tag)
        assertEquals(Tag(0x0008, 0x1110), entry.tag)
        assertNull(entry.type)
        assertTrue(entry.description.startsWith("Uniquely identifies"))
    }

    @Test
    fun buildImdEntry2() {
        val url = this::class.java.getResource("build_imd_entry2.xml")
            ?: throw NullPointerException("Failed to obtain test resource (build_imd_entry2.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = buildImdEntry(root)
        assertTrue(opt.isPresent)
        val entry = opt.get() as DataEntry
        assertTrue(entry.isData())
        assertFalse(entry.isInclude())
        assertEquals("Patient's Alternative Calendar", entry.name)
        assertNotNull(entry.tag)
        assertEquals(Tag(0x0010, 0x0035), entry.tag)
        assertEquals(AttributeType.Type1C, entry.type)
        assertTrue(entry.description.startsWith("The Alternative"))
    }

    @Test
    fun buildImdEntry3() {
        val url = this::class.java.getResource("build_imd_entry3.xml")
            ?: throw NullPointerException("Failed to obtain test resource (build_imd_entry3.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = buildImdEntry(root)
        assertTrue(opt.isPresent)
        val entry = opt.get() as IncludeEntry
        assertTrue(entry.isInclude())
        assertFalse(entry.isData())
        assertEquals(0.toUShort(), entry.seqIndent)
        assertEquals(XRef("table_10-16", "select: label quotedtitle"), entry.xref)
        assertTrue(entry.description.startsWith("No Baseline"))
    }

    @Test
    fun buildImdEntry4() {
        val url = this::class.java.getResource("build_imd_entry4.xml")
            ?: throw NullPointerException("Failed to obtain test resource (build_imd_entry4.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val opt = buildImdEntry(root)
        assertTrue(opt.isPresent)
        val entry = opt.get() as IncludeEntry
        assertTrue(entry.isInclude())
        assertFalse(entry.isData())
        assertEquals(1.toUShort(), entry.seqIndent)
        assertEquals(XRef("table_10-11", "select: label quotedtitle"), entry.xref)
        assertTrue(entry.description.isEmpty())
    }

    @Test
    fun findDependents() {
        val ds = DicomStandard()
        ds.add(
            Imd(
                "table_C.2-1", listOf(), mutableListOf(
                    IncludeEntry(0u, XRef("table_10-11")),
                    IncludeEntry(0u, XRef("table_10-18")),
                )
            )
        )
        val url = this::class.java.getResource("find_dependent.xml")
            ?: throw NullPointerException("Failed to obtain test resource (find_dependent.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))

        assertTrue(findDependents(document, ds))
        var imd = ds.imd("table_10-11")
        assertNotNull(imd)
        imd = imd!!

        assertEquals("table_10-11", imd.id)
        assertEquals(2, imd.items.size)
        assertEquals("Referenced SOP Class UID", (imd.items[0] as DataEntry).name)
        assertEquals(Tag(0x0008, 0x1150), (imd.items[0] as DataEntry).tag)
        assertEquals(AttributeType.Type1, (imd.items[0] as DataEntry).type)
        assertTrue((imd.items[0] as DataEntry).description.startsWith("Uniquely identifies the "))

        assertEquals("Referenced SOP Instance UID", (imd.items[1] as DataEntry).name)
        assertEquals(Tag(0x0008, 0x1155), (imd.items[1] as DataEntry).tag)
        assertEquals(AttributeType.Type1, (imd.items[1] as DataEntry).type)
        assertTrue((imd.items[1] as DataEntry).description.startsWith("Uniquely identifies the "))

        imd = ds.imd("table_10-18")
        assertNotNull(imd)
        imd = imd!!
        assertEquals("table_10-18", imd.id)
        assertEquals(11, imd.items.size)
        assertEquals("Issuer of Patient ID", (imd.items[0] as DataEntry).name)
        assertFalse((imd.items[0] as DataEntry).isSequence())
        assertEquals(Tag(0x0010, 0x0021), (imd.items[0] as DataEntry).tag)
        assertEquals(AttributeType.Type3, (imd.items[0] as DataEntry).type)
        assertTrue((imd.items[0] as DataEntry).description.startsWith("Identifier of the Assigning"))

        assertEquals("Issuer of Patient ID Qualifiers Sequence", (imd.items[1] as DataEntry).name)
        assertTrue((imd.items[1] as DataEntry).isSequence())
        assertEquals(Tag(0x0010, 0x0024), (imd.items[1] as DataEntry).tag)
        assertEquals(AttributeType.Type3, (imd.items[1] as DataEntry).type)
        assertTrue((imd.items[1] as DataEntry).description.startsWith("Attributes specifying or"))

    }

    @Test
    fun trimWsNl() {
        val ls = listOf(" abc ", "\n\r abc \r\n ")
        for (s in ls) {
            assertEquals("abc", trimWsNl(s))
        }
    }

}