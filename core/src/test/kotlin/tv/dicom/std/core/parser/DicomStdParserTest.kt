package tv.dicom.std.core.parser

import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*
import tv.dicom.std.core.model.AttributeType
import tv.dicom.std.core.model.Tag
import tv.dicom.std.core.model.Usage
import tv.dicom.std.core.model.XRef
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.ciod.Entry
import tv.dicom.std.core.model.imd.DataEntry
import tv.dicom.std.core.model.imd.IncludeEntry
import java.io.File
import java.util.*
import javax.xml.parsers.DocumentBuilderFactory

class DicomStdParserTest {
    @Test
    fun parsePart03SectA2() {
        val url = this::class.java.getResource("part_03_extract.xml")
            ?: throw NullPointerException("Failed to obtain test resource (part_03_extract.xml)")
        val file = File(url.toURI())
        val opt = parse(file)
        assertTrue(opt.isPresent)
        val ds = opt.get()
        assertTrue(ds.ciods.size>0)
        assertTrue(ds.imds.size>0)
        //TODO test content
    }

    @Test
    fun buildCiod() {
        val url = this::class.java.getResource("part_03_table_A.2-1.xml")
            ?: throw NullPointerException("Failed to obtain test resource (part_03_table_A.2-1.xml)")
        val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
        val document = builder.parse(File(url.toURI()))
        val root = document.documentElement

        val expected = Ciod()
        expected.id = "table_A.2-1"
        expected.items.add(Entry("Patient", "Patient", XRef("sect_C.7.1.1", "select: labelnumber"), Usage.M))
        expected.items.add(
            Entry(
                "Patient",
                "Clinical Trial Subject",
                XRef("sect_C.7.1.3", "select: labelnumber"),
                Usage.U
            )
        )
        expected.items.add(Entry("Study", "General Study", XRef("sect_C.7.2.1", "select: labelnumber"), Usage.M))
        expected.items.add(Entry("Study", "Patient Study", XRef("sect_C.7.2.2", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Study", "Clinical Trial Study", XRef("sect_C.7.2.3", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Series", "General Series", XRef("sect_C.7.3.1", "select: labelnumber"), Usage.M))
        expected.items.add(Entry("Series", "CR Series", XRef("sect_C.8.1.1", "select: labelnumber"), Usage.M))
        expected.items.add(
            Entry(
                "Series",
                "Clinical Trial Series",
                XRef("sect_C.7.3.2", "select: labelnumber"),
                Usage.U
            )
        )
        expected.items.add(
            Entry(
                "Equipment",
                "General Equipment",
                XRef("sect_C.7.5.1", "select: labelnumber"),
                Usage.M
            )
        )
        expected.items.add(
            Entry(
                "Acquisition",
                "General Acquisition",
                XRef("sect_C.7.10.1", "select: labelnumber"),
                Usage.M
            )
        )
        expected.items.add(Entry("Image", "General Image", XRef("sect_C.7.6.1", "select: labelnumber"), Usage.M))
        expected.items.add(Entry("Image", "General Reference", XRef("sect_C.12.4", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "Image Pixel", XRef("sect_C.7.6.3", "select: labelnumber"), Usage.M))
        expected.items.add(Entry("Image", "Contrast/Bolus", XRef("sect_C.7.6.4", "select: labelnumber"), Usage.C))
        expected.items.add(Entry("Image", "Display Shutter", XRef("sect_C.7.6.11", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "Device", XRef("sect_C.7.6.12", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "Specimen", XRef("sect_C.7.6.22", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "CR Image", XRef("sect_C.8.1.2", "select: labelnumber"), Usage.M))
        expected.items.add(Entry("Image", "Overlay Plane", XRef("sect_C.9.2", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "Modality LUT", XRef("sect_C.11.1", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "VOI LUT", XRef("sect_C.11.2", "select: labelnumber"), Usage.U))
        expected.items.add(Entry("Image", "SOP Common", XRef("sect_C.12.1", "select: labelnumber"), Usage.M))
        expected.items.add(
            Entry(
                "Image",
                "Common Instance Reference",
                XRef("sect_C.12.2", "select: labelnumber"),
                Usage.U
            )
        )

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
}