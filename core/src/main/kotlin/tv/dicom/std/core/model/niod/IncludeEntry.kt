package tv.dicom.std.core.model.niod

import tv.dicom.std.core.model.XRef

class IncludeEntry(var seqIndent: UShort=0u, var xref: XRef =XRef(), var description: String="") : Entry {
    override fun isSequence(): Boolean {
        return false
    }
}
