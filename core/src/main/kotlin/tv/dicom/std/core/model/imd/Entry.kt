package tv.dicom.std.core.model.imd

interface Entry {
    fun isSequence(): Boolean
    fun isInclude(): Boolean
    fun isData(): Boolean
}