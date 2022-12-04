package tv.dicom.std.core.model.ciod

/**
 * Composite Information Object Definition as defined in the DICOM standard part 03 chapter A
 *
 * @property id XML table ID
 * @property parentIds Zero or more XML IDs of parent elements
 * @property items properties of the Composite Information Object Definition
 * @constructor Create Composite Information Object Definition
 */
data class Ciod(
    var id: String = "",
    var parentIds: List<String> = listOf(),
    val items: MutableList<Entry> = mutableListOf()
)
