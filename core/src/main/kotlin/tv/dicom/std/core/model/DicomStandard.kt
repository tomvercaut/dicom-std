package tv.dicom.std.core.model

import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.Imd

class DicomStandard {
    val ciods: MutableList<Ciod> = mutableListOf()
    val imds: MutableList<Imd> = mutableListOf()
}
