package tv.dicom.std.core.model.imd

import tv.dicom.std.core.model.Tag

data class DataEntry(
    val seqIndent: UShort = 0u,
    var name: String = "",
    var tag: Tag = Tag(),
    var description: String = ""
) : Entry {
    override fun isSequence(): Boolean {
        return name.endsWith(" Sequence")
    }

    override fun isInclude(): Boolean {
        return false
    }

    override fun isData(): Boolean {
        return true
    }
}