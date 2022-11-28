package tv.dicom.std.core.model.coid

import tv.dicom.std.core.model.coid.Entry

data class Definition(
    val entries: List<Entry> = mutableListOf<Entry>()
)
