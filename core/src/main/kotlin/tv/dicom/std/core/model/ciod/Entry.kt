package tv.dicom.std.core.model.ciod

import tv.dicom.std.core.model.Usage
import tv.dicom.std.core.model.XRef

data class Entry(var ie: String = "", var module: String = "", var reference: XRef = XRef(), var usage: Usage = Usage.U)
