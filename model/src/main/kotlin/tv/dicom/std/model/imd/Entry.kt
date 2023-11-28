package tv.dicom.std.model.imd

interface Entry {
    fun isSequence(): Boolean
    fun isInclude(): Boolean
    fun isData(): Boolean
}