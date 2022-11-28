package tv.dicom.std.core.model

import tv.dicom.std.core.model.coid.Definition

class DicomStandard {
    val ciods: MutableList<Definition> = mutableListOf()
    val niods: MutableList<tv.dicom.std.core.model.niod.Definition> = mutableListOf()
}
