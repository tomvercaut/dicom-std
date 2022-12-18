package tv.dicom.std.core.parser

import tv.dicom.std.core.model.Tag
import tv.dicom.std.core.model.Usage
import tv.dicom.std.core.model.XRef
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.ciod.Entry
import tv.dicom.std.core.model.imd.DataEntry
import tv.dicom.std.core.model.imd.Imd
import tv.dicom.std.core.model.imd.IncludeEntry

class TestHelper {

    companion object {
        @JvmStatic
        fun tableA_2_1(): Ciod {
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

        @JvmStatic
        fun tableA_3_1(): Ciod {
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

        @JvmStatic
        fun tableA(): List<Ciod> {
            return listOf(tableA_2_1(), tableA_3_1())
        }

        @JvmStatic
        fun tableC_2_1(): Imd {
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

        @JvmStatic
        fun tableC_2_2(): Imd {
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
            imd.items.add(
                DataEntry(
                    0u,
                    "Other Patient Names",
                    Tag(0x0010, 0x1001),
                    null,
                    "Other names used to identify"
                )
            )
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

        @JvmStatic
        fun tableC(): List<Imd> {
            return listOf(tableC_2_1(), tableC_2_2())
        }

    }
}