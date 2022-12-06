package tv.dicom.std.core.model

import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.Imd

class DicomStandard {
    var ciods: List<Ciod> = listOf()
    var imds: List<Imd> = listOf()
}
