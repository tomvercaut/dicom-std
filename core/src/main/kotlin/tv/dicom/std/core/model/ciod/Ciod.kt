package tv.dicom.std.core.model.ciod

/**
 * Composite Information Object Definition as defined in the DICOM standard part 03 chapter A
 *
 * @property ids One or more IDs (first ID could be the matching table ID)
 * @property items properties of the Composite Information Object Definition
 * @constructor Create Composite Information Object Definition
 */
data class Ciod(
        var ids: MutableList<String> = mutableListOf(),
        val items: List<Entry> = mutableListOf<Entry>()
)
