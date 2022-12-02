package tv.dicom.std.core.model.imd

import tv.dicom.std.core.model.XRef

class IncludeEntry(var seqIndent: UShort = 0u, var xref: XRef = XRef(), var description: String = "") : Entry {
    override fun isSequence(): Boolean {
        return false
    }

    override fun isInclude(): Boolean {
        return true
    }

    override fun isData(): Boolean {
        return false
    }
}
