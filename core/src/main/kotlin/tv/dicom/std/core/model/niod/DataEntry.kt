package tv.dicom.std.core.model.niod

import tv.dicom.std.core.model.Tag

data class DataEntry(var name: String, var tag: Tag= Tag(), var description: String="") : Entry {}