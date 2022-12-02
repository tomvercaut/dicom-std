package tv.dicom.std.core.model

data class Tag(var group: UShort, var element: UShort) {
    constructor() : this(0u, 0u)
}
