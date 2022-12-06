package tv.dicom.std.core.model.imd

import tv.dicom.std.core.model.AttributeType
import tv.dicom.std.core.model.Tag

data class DataEntry(
    var seqIndent: UShort = 0u,
    // Attribute name
    var name: String = "",
    // Data element Tag
    var tag: Tag = Tag(),
    // Type designation
    var type: AttributeType? = null,
    // Attribute definition
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