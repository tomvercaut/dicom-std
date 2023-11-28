package tv.dicom.std.model.dictionary

import org.apache.logging.log4j.kotlin.logger

/**
 * Represents a DICOM tag that is ranged, and consists of a ranged group and element.
 *
 * Ranged DICOM tags are used in the DICOM part 06, in the section describing the DICOM data element registry.
 *
 * @property group The ranged group of the DICOM tag.
 * @property element The ranged element of the DICOM tag.
 */
data class RangedTag (
    var group: RangedTagItem,
    var element: RangedTagItem,
) {
    companion object{
        @JvmStatic
        private val log = logger(this::class.java.name)

        /**
         * Creates a RangedTag object from a given string.
         *
         * The string should have the format "(group,element)" where group and element are valid RangedTagItems.
         *
         * @param s The input string representing the RangedTag.
         * @return If the string is in the correct format and the RangedTagItems are valid, a RangedTag object is returned. Otherwise, null is returned.
         */
        @JvmStatic
        fun of(s: String): RangedTag? {
            try {
                val i = s.indexOf('(')
                val j = s.indexOf(',')
                val k = s.indexOf(')')
                if (i == -1 || j == -1 || k == -1) {
                    throw IllegalArgumentException("String [$s] does not have a valid Tag format")
                }
                val g = s.substring(i + 1, j)
                val e = s.substring(j + 1, k)
                val group = RangedTagItem.of(g)
                val element = RangedTagItem.of(e)
                if (group == null || element == null) {
                    throw IllegalArgumentException("Input argument [$s] doesn't have a valid format.")
                }
                return RangedTag(group, element)
            } catch (e: IllegalArgumentException) {
                log.error("Input argument [$s] doesn't have a valid format.")
                return null
            }
        }
    }

    /**
     * Determines if the current RangedTag is ranged or not.
     *
     * @return True if either the group or the element is ranged, indicating that the RangedTag is ranged. False otherwise.
     */
    fun isRanged(): Boolean {
        return group.isRanged() || element.isRanged()
    }
}
