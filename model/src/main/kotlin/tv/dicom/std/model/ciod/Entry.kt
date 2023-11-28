package tv.dicom.std.model.ciod

import tv.dicom.std.model.Usage
import tv.dicom.std.model.XRef

data class Entry(var ie: String = "", var module: String = "", var reference: XRef = XRef(), var usage: Usage = Usage.U)
