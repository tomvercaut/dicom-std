package tv.dicom.std.core.model.imd

/**
 * Information Module Definition as defined in the DICOM standard part 03 chapter C.
 *
 * @property ids One or more IDs (first ID could be the matching table ID)
 * @property items properties of the Information Module Definition
 * @constructor Create an Information Module Definition
 */
data class Imd(
    var ids: MutableList<String> = mutableListOf(),
    var items: MutableList<Entry> = mutableListOf()
)