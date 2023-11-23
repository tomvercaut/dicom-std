package tv.dicom.std.core.model.dictionary

import org.apache.logging.log4j.kotlin.logger

/**
 * Represents a ranged DICOM tag item (group or element) with a minimum and maximum value.
 *
 * @property min The minimum value of the ranged tag item.
 * @property max The maximum value of the ranged tag item.
 */
data class RangedTagItem(var min: UShort = 0u, var max: UShort = 0u) {

    companion object {
        @JvmStatic
        private val log = logger(this::class.java.name)

        @JvmStatic
        private val pattern1 = """[0-9a-fA-F]{4}""".toRegex()

        @JvmStatic
        private val pattern2 = """[0-9a-fA-FxX]{4}""".toRegex()

        /**
         * Create a ranged DICOM tag item from a String.
         *
         * Accepted Regex format of the String are:
         *
         *
         * * &#x005B;0-9a-fA-F&#x005D;{4}
         * * &#x005B;0-9a-fA-FxX&#x005D;{4}: where x or X are subsitituted for 0 and F to construct a RangedTagItem
         *
         * @param s input String
         * @return If no errors were detected, a RangedTagItem is returned. If an error is detected, null is returned.
         */
        @JvmStatic
        fun of(s: String): RangedTagItem? {
            try {
                if (s.matches(pattern1)) {
                    val x = s.toInt(16)
                    val y = x.toUShort()
                    if (isValid(s, x)) {
                        return RangedTagItem(y, y)
                    }
                    return null
                } else if (s.matches(pattern2)) {
                    val t = s.lowercase().replace('x', '0')
                    val u = s.lowercase().replace('x', 'f')
                    val rti0 = of(t)
                    val rti1 = of(u)
                    if (rti0 != null && rti1 != null && !rti0.isRanged() && !rti1.isRanged()) {
                        return RangedTagItem(rti0.min, rti1.min)
                    }
                    return null
                } else {
                    throw IllegalArgumentException("Input argument [${s}] doesn't have a valid format.")
                }
            } catch (e: NumberFormatException) {
                log.error(e)
                return null
            } catch (e: IllegalArgumentException) {
                log.error(e)
                return null
            }
        }

        /**
         * Determines if the input integer is within the range of an unsigned short.
         *
         * @param s The input string from which the integer was derived.
         * @param x The integer value to compare against.
         * @return True if the input integer is within the range of an unsigned short , false otherwise.
         */
        @JvmStatic
        private fun isValid(s: String, x: Int): Boolean {
            if (x > UShort.MAX_VALUE.toInt()) {
                log.error("Input string [$s] doesn't have a valid ${this::class.java.name} format. Value is larger than ${UShort.MAX_VALUE}")
                return false
            }
            if (x < 0) {
                log.error("Input string [$s] doesn't have a valid ${this::class.java.name} format. Value must be larger or equal to 0.")
                return false
            }
            return true
        }
    }

    constructor(value: UShort) : this(value, value)

    /**
     * Determines if the ranged tag item is actually ranged or not.
     *
     * @return True if the minimum and maximum values are different, indicating the tag item is ranged. False otherwise.
     */
    fun isRanged(): Boolean {
        return min != max
    }
}
