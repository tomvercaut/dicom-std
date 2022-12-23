package tv.dicom.std.core.parser

import org.apache.logging.log4j.kotlin.logger
import org.junit.jupiter.api.Test

import org.junit.jupiter.api.Assertions.*
import tv.dicom.std.core.model.*
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.DataEntry
import tv.dicom.std.core.model.imd.IncludeEntry
import tv.dicom.std.core.parser.TestHelper.Companion.tableA
import tv.dicom.std.core.parser.TestHelper.Companion.tableC
import java.io.File

class DicomStdParserTest {

    companion object {
        var log = logger(this::class.java.name)
    }

    @Test
    fun parsePart03SectA2() {
        val url = this::class.java.getResource("part_03_extract.xml")
            ?: throw NullPointerException("Failed to obtain test resource (part_03_extract.xml)")
        val file = File(url.toURI())

        val eds = DicomStandard()
        for (ciod in tableA()) {
            eds.add(ciod)
        }
        for (imd in tableC()) {
            eds.add(imd)
        }

        val opt = parse(file)
        assertTrue(opt.isPresent)
        val ds = opt.get()
        assertTrue(eds.ciodIds().size <= ds.ciodIds().size)
        assertTrue(eds.imdIds().size <= ds.imdIds().size)
        for (key in eds.ciodIds()) {
            assertTrue(ds.ciodIds().contains(key))
        }
        for (key in eds.imdIds()) {
            assertTrue(ds.imdIds().contains(key))
        }
        for (key in eds.ciodIds()) {
            assertNotNull(eds.ciod(key))
            assertNotNull(ds.ciod(key))
            val eciod: Ciod = eds.ciod(key)!!
            val ciod: Ciod = ds.ciod(key)!!
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
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $key, entry $j"
                )
                assertEquals(
                    eItem.module,
                    item.module,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $key, entry $j"
                )
                assertEquals(
                    eItem.usage,
                    item.usage,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $key, entry $j"
                )
                assertEquals(
                    eItem.reference,
                    item.reference,
                    "Composite Information Module [${eItem.ie}, ${eItem.module}] mismatch at $key, entry $j"
                )
            }
        }
        for (key in eds.imdIds()) {
            assertNotNull(eds.imd(key))
            assertNotNull(ds.imd(key))
            val eimd = eds.imd(key)!!
            val imd = ds.imd(key)!!
            assertEquals(eimd.id, imd.id, "Information Definition Module mismatch at $key")
            assertEquals(eimd.parentIds, imd.parentIds, "Information Definition Module mismatch at $key")
            assertEquals(eimd.items.size, imd.items.size, "Information Definition Module mismatch at $key")
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
                        "Information Definition Module mismatch at $key, entry $j"
                    )
                    assertEquals(eEntry.name, entry.name, "Information Definition Module mismatch at $key, entry $j")
                    assertEquals(eEntry.tag, entry.tag, "Information Definition Module mismatch at $key, entry $j")
                    assertEquals(eEntry.type, entry.type, "Information Definition Module mismatch at $key, entry $j")
                    assertTrue(
                        entry.description.startsWith(eEntry.description),
                        "Information Definition Module mismatch at $key, entry $j.\n" +
                                "eEntry description: ${eEntry.description}\n" +
                                "entry description: ${entry.description}"
                    )
                } else if (eItem.isInclude()) {
                    val eEntry = eItem as IncludeEntry
                    val entry = item as IncludeEntry
                    assertEquals(
                        eEntry.seqIndent,
                        entry.seqIndent,
                        "Information Definition Module mismatch at $key, entry $j"
                    )

                }
            }
        }
    }

}