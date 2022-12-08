package tv.dicom.std.core.parser

import org.apache.logging.log4j.kotlin.logger
import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*
import tv.dicom.std.core.model.*
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.ciod.Entry
import tv.dicom.std.core.model.imd.DataEntry
import tv.dicom.std.core.model.imd.Imd
import tv.dicom.std.core.model.imd.IncludeEntry
import java.io.File
import javax.xml.parsers.DocumentBuilderFactory

class DicomStdParserTest {

    companion object {
        var log = logger(this::class.java.name)
    }

    private fun tableA_2_1(): Ciod {
        val style = "select: labelnumber"
        val ciod = Ciod()
        ciod.id = "table_A.2-1"
        ciod.items.add(Entry("Patient", "Patient", XRef("sect_C.7.1.1", style), Usage.M))
        ciod.items.add(
            Entry(
                "Patient",
                "Clinical Trial Subject",
                XRef("sect_C.7.1.3", style),
                Usage.U
            )
        )
        ciod.items.add(Entry("Study", "General Study", XRef("sect_C.7.2.1", style), Usage.M))
        ciod.items.add(Entry("Study", "Patient Study", XRef("sect_C.7.2.2", style), Usage.U))
        ciod.items.add(Entry("Study", "Clinical Trial Study", XRef("sect_C.7.2.3", style), Usage.U))
        ciod.items.add(Entry("Series", "General Series", XRef("sect_C.7.3.1", style), Usage.M))
        ciod.items.add(Entry("Series", "CR Series", XRef("sect_C.8.1.1", style), Usage.M))
        ciod.items.add(
            Entry(
                "Series",
                "Clinical Trial Series",
                XRef("sect_C.7.3.2", style),
                Usage.U
            )
        )
        ciod.items.add(
            Entry(
                "Equipment",
                "General Equipment",
                XRef("sect_C.7.5.1", style),
                Usage.M
            )
        )
        ciod.items.add(
            Entry(
                "Acquisition",
                "General Acquisition",
                XRef("sect_C.7.10.1", style),
                Usage.M
            )
        )
        ciod.items.add(Entry("Image", "General Image", XRef("sect_C.7.6.1", style), Usage.M))
        ciod.items.add(Entry("Image", "General Reference", XRef("sect_C.12.4", style), Usage.U))
//        ciod.items.add(Entry("Image", "General Plane", XRef("sect_C.7.6.2", style), Usage.M))
        ciod.items.add(Entry("Image", "Image Pixel", XRef("sect_C.7.6.3", style), Usage.M))
        ciod.items.add(Entry("Image", "Contrast/Bolus", XRef("sect_C.7.6.4", style), Usage.C))
        ciod.items.add(Entry("Image", "Display Shutter", XRef("sect_C.7.6.11", style), Usage.U))
        ciod.items.add(Entry("Image", "Device", XRef("sect_C.7.6.12", style), Usage.U))
        ciod.items.add(Entry("Image", "Specimen", XRef("sect_C.7.6.22", style), Usage.U))
        ciod.items.add(Entry("Image", "CR Image", XRef("sect_C.8.1.2", style), Usage.M))
        ciod.items.add(Entry("Image", "Overlay Plane", XRef("sect_C.9.2", style), Usage.U))
        ciod.items.add(Entry("Image", "Modality LUT", XRef("sect_C.11.1", style), Usage.U))
        ciod.items.add(Entry("Image", "VOI LUT", XRef("sect_C.11.2", style), Usage.U))
        ciod.items.add(Entry("Image", "SOP Common", XRef("sect_C.12.1", style), Usage.M))
        ciod.items.add(
            Entry(
                "Image",
                "Common Instance Reference",
                XRef("sect_C.12.2", style),
                Usage.U
            )
        )
        return ciod
    }

    private fun tableA_3_1(): Ciod {
        val style = "select: labelnumber"
        val ciod = Ciod()
        ciod.id = "table_A.3-1"
        ciod.items.add(Entry("Patient", "Patient", XRef("sect_C.7.1.1", style), Usage.M))
        ciod.items.add(Entry("Patient", "Clinical Trial Subject", XRef("sect_C.7.1.3", style), Usage.U))
        ciod.items.add(Entry("Study", "General Study", XRef("sect_C.7.2.1", style), Usage.M))
        ciod.items.add(Entry("Study", "Patient Study", XRef("sect_C.7.2.2", style), Usage.U))
        ciod.items.add(Entry("Study", "Clinical Trial Study", XRef("sect_C.7.2.3", style), Usage.U))
        ciod.items.add(Entry("Series", "General Series", XRef("sect_C.7.3.1", style), Usage.M))
        ciod.items.add(Entry("Series", "Clinical Trial Series", XRef("sect_C.7.3.2", style), Usage.U))
        ciod.items.add(Entry("Frame of Reference", "Frame of Reference", XRef("sect_C.7.4.1", style), Usage.M))
        ciod.items.add(Entry("Frame of Reference", "Synchronization", XRef("sect_C.7.4.2", style), Usage.C))
        ciod.items.add(Entry("Equipment", "General Equipment", XRef("sect_C.7.5.1", style), Usage.M))
        ciod.items.add(Entry("Acquisition", "General Acquisition", XRef("sect_C.7.10.1", style), Usage.M))
        ciod.items.add(Entry("Image", "General Image", XRef("sect_C.7.6.1", style), Usage.M))
        ciod.items.add(Entry("Image", "General Reference", XRef("sect_C.12.4", style), Usage.U))
        ciod.items.add(Entry("Image", "Image Plane", XRef("sect_C.7.6.2", style), Usage.M))
        ciod.items.add(Entry("Image", "Image Pixel", XRef("sect_C.7.6.3", style), Usage.M))
        ciod.items.add(Entry("Image", "Contrast/Bolus", XRef("sect_C.7.6.4", style), Usage.C))
        ciod.items.add(Entry("Image", "Device", XRef("sect_C.7.6.12", style), Usage.U))
        ciod.items.add(Entry("Image", "Specimen", XRef("sect_C.7.6.22", style), Usage.U))
        ciod.items.add(Entry("Image", "CT Image", XRef("sect_C.8.2.1", style), Usage.M))
        ciod.items.add(Entry("Image", "Multi-energy CT Image", XRef("sect_C.8.2.2", style), Usage.C))
        ciod.items.add(Entry("Image", "Overlay Plane", XRef("sect_C.9.2", style), Usage.U))
        ciod.items.add(Entry("Image", "VOI LUT", XRef("sect_C.11.2", style), Usage.U))
        ciod.items.add(Entry("Image", "SOP Common", XRef("sect_C.12.1", style), Usage.M))
        ciod.items.add(Entry("Image", "Common Instance Reference", XRef("sect_C.12.2", style), Usage.U))
        return ciod
    }

    private fun tableA(): Map<String, Ciod> {
        val t0 = tableA_2_1()
        val t1 = tableA_3_1()
        val ciods = mutableMapOf<String, Ciod>()
        ciods[t0.id] = t0
        ciods[t1.id] = t1
        return ciods
    }

    private fun tableC_2_1(): Imd {
        val xstyle = "select: label quotedtitle"
        val imd = Imd()
        imd.id = "table_C.2-1"
        imd.parentIds = listOf("sect_C.2.1", "sect_C.2", "chapter_C", "PS3.3")
        imd.items.add(
            DataEntry(
                0u,
                "Referenced Study Sequence",
                Tag(0x0008, 0x1110),
                null,
                "Uniquely identifies the Study"
            )
        )
        imd.items.add(IncludeEntry(1u, XRef("table_10-11", xstyle), ""))
        imd.items.add(
            DataEntry(
                0u,
                "Referenced Visit Sequence",
                Tag(0x0008, 0x1125),
                null,
                "Uniquely identifies the Visit"
            )
        )
        imd.items.add(IncludeEntry(1u, XRef("table_10-11", xstyle), ""))
        imd.items.add(
            DataEntry(
                0u,
                "Referenced Patient Alias Sequence",
                Tag(0x0038, 0x0004),
                null,
                "Uniquely identifies any Patient"
            )
        )
        imd.items.add(IncludeEntry(1u, XRef("table_10-11", xstyle), ""))
        return imd
    }

    private fun tableC_2_2(): Imd {
        val xstyle = "select: label quotedtitle"
        val imd = Imd()
        imd.id = "table_C.2-2"
        imd.parentIds = listOf("sect_C.2.2", "sect_C.2", "chapter_C", "PS3.3")
        imd.items.add(DataEntry(0u, "Patient's Name", Tag(0x0010, 0x0010), null, "Patient's full name."))
        imd.items.add(DataEntry(0u, "Patient ID", Tag(0x0010, 0x0020), null, "Primary identifier for the"))
        imd.items.add(IncludeEntry(0u, XRef("table_10-18", xstyle), ""))
        imd.items.add(DataEntry(0u, "Type of Patient ID", Tag(0x0010, 0x0022), null, "The type of identifier in"))
        imd.items.add(
            DataEntry(
                0u,
                "Other Patient IDs Sequence",
                Tag(0x0010, 0x1002),
                null,
                "A Sequence of identification"
            )
        )
        imd.items.add(DataEntry(1u, "Patient ID", Tag(0x0010, 0x0020), null, "An identifier for the"))
        imd.items.add(IncludeEntry(1u, XRef("table_10-18", xstyle), ""))
        imd.items.add(DataEntry(1u, "Type of Patient ID", Tag(0x0010, 0x0022), null, "The type of identifier in"))
        imd.items.add(DataEntry(0u, "Other Patient Names", Tag(0x0010, 0x1001), null, "Other names used to identify"))
        imd.items.add(DataEntry(0u, "Patient's Birth Name", Tag(0x0010, 0x1005), null, "Patient's birth name."))
        imd.items.add(
            DataEntry(
                0u,
                "Patient's Mother's Birth Name",
                Tag(0x0010, 0x1060),
                null,
                "Birth name of Patient's"
            )
        )
        imd.items.add(
            DataEntry(
                0u,
                "Referenced Patient Photo Sequence",
                Tag(0x0010, 0x1100),
                null,
                "A photo to confirm"
            )
        )
        imd.items.add(IncludeEntry(1u, XRef("table_10-3b", xstyle), ""))
        imd.items.add(IncludeEntry(0u, XRef("table_C.7.1.4-1", xstyle), ""))
        return imd
    }

    private fun tableC(): Map<String, Imd> {
        val t0 = tableC_2_1()
        val t1 = tableC_2_2()
        val imds = mutableMapOf<String, Imd>()
        imds[t0.id] = t0
        imds[t1.id] = t1
        return imds
    }

    @Test
    fun parsePart03SectA2() {
        val url = this::class.java.getResource("part_03_extract.xml")
            ?: throw NullPointerException("Failed to obtain test resource (part_03_extract.xml)")
        val file = File(url.toURI())

        val eds = DicomStandard()
        eds.ciods = tableA()
        eds.imds = tableC()

        val opt = parse(file)
        assertTrue(opt.isPresent)
        val ds = opt.get()
        assertEquals(eds.ciods.size, ds.ciods.size)
        assertEquals(eds.imds.size, ds.imds.size)
        for (i in eds.ciods.keys.indices) {
            val eKey = eds.ciods.keys.elementAt(i)
            val key = ds.ciods.keys.elementAt(i)
            assertEquals(eKey, key)
            assertNotNull(eds.ciods[eKey])
            assertNotNull(ds.ciods[key])
            val eciod : Ciod = eds.ciods[eKey]!!
            val ciod : Ciod = ds.ciods[key]!!
            val min = if (eciod.items.size < ciod.items.size) {
                eciod.items.size
            } else {
                ciod.items.size
            }
            for (j in 0 until min) {
                if (ciod.items[j].ie != eciod.items[j].ie ||
                    ciod.items[j].module != eciod.items[j].module
                ) {
                    log.warn("\nCIOD entry: ${ciod.items[j]}\neCIOD entry: ${eciod.items[j]}")
                }
            }
            assertEquals(eciod.items.size, ciod.items.size)
            for (j in eciod.items.indices) {
                val eItem = eciod.items[j]
                val item = ciod.items[j]
                assertEquals(
                    eItem.ie,
                    item.ie,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $i, entry $j"
                )
                assertEquals(
                    eItem.module,
                    item.module,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $i, entry $j"
                )
                assertEquals(
                    eItem.usage,
                    item.usage,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $i, entry $j"
                )
                assertEquals(
                    eItem.reference,
                    item.reference,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $i, entry $j"
                )
            }
        }
        for (i in eds.imds.keys.indices) {
            val eKey = eds.imds.keys.elementAt(i)
            val key = ds.imds.keys.elementAt(i)
            assertEquals(eKey, key)
            assertNotNull(eds.imds[eKey])
            assertNotNull(ds.imds[key])
            val eimd = eds.imds[eKey]!!
            val imd = ds.imds[key]!!
            assertEquals(eimd.id, imd.id, "Information Definition Module mismatch at $i")
            assertEquals(eimd.parentIds, imd.parentIds, "Information Definition Module mismatch at $i")
            assertEquals(eimd.items.size, imd.items.size, "Information Definition Module mismatch at $i")
            for (j in eimd.items.indices) {
                val eItem = eimd.items[j]
                val item = imd.items[j]
                assertEquals(eItem.isData(), item.isData())
                assertEquals(eItem.isInclude(), item.isInclude())
                assertEquals(eItem.isSequence(), item.isSequence())
                if (eItem.isData()) {
                    val eEntry = eItem as DataEntry
                    val entry = item as DataEntry
                    assertEquals(
                        eEntry.seqIndent,
                        entry.seqIndent,
                        "Information Definition Module mismatch at $i, entry $j"
                    )
                    assertEquals(eEntry.name, entry.name, "Information Definition Module mismatch at $i, entry $j")
                    assertEquals(eEntry.tag, entry.tag, "Information Definition Module mismatch at $i, entry $j")
                    assertEquals(eEntry.type, entry.type, "Information Definition Module mismatch at $i, entry $j")
                    assertTrue(
                        entry.description.startsWith(eEntry.description),
                        "Information Definition Module mismatch at $i, entry $j.\n" +
                                "eEntry description: ${eEntry.description}\n" +
                                "entry description: ${entry.description}"
                    )
                } else if (eItem.isInclude()) {
                    val eEntry = eItem as IncludeEntry
                    val entry = item as IncludeEntry
                    assertEquals(
                        eEntry.seqIndent,
                        entry.seqIndent,
                        "Information Definition Module mismatch at $i, entry $j"
                    )

                }
            }
        }
    }

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
}