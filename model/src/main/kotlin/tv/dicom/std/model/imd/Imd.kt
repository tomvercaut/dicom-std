package tv.dicom.std.model.imd

/**
 * Information Module Definition as defined in the DICOM standard part 03 chapter C.
 *
 * @property id XML table ID
 * @property caption XML table caption
 * @property parentIds zero or more XML IDs of parent elements
 * @property items properties of the Information Module Definition
 * @constructor Create an Information Module Definition
 */
data class Imd(
    var id: String = "",
    var caption: String = "",
    var parentIds: List<String> = listOf(),
    var items: MutableList<Entry> = mutableListOf()
)