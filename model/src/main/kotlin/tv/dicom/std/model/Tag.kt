package tv.dicom.std.model

import org.apache.logging.log4j.kotlin.logger
import java.lang.IllegalArgumentException

/**
 * A DICOM tag is used to identify a DICOM data element and is composed of a group and element value.
 *
 * Both the group and element consist of a 16 bit unsigned short.
 *
 * @property group Group number
 * @property element Element number
 * @constructor Create empty Tag where the group and element values are set to 0.
 */
data class Tag(var group: UShort = 0u, var element: UShort = 0u) {

    companion object {

        @JvmStatic
        private val log = logger(this::class.java.name)

        @JvmStatic
        private val pattern1 = """[0-9a-fA-F]{4}""".toRegex()
        private val pattern2 = """0[xX][0-9a-fA-F]{4}""".toRegex()

        /**
         * Create a DICOM Tag from a String.
         *
         * Accepted format for the String are:
         * * (gggg,eeee)
         * * (0xgggg,0xeeee)
         *
         *  Where `gggg` is the 4 digit hex value of the group and `eeee` is the 4 digit hex value of the element of the [Tag].
         *
         * @param s input String
         * @return If no error where detected, a Tag is returned. If an error is detected, null is returned.
         */
        @JvmStatic
        fun of(s: String): Tag? {
            try {
                val i = s.indexOf('(')
                val j = s.indexOf(',')
                val k = s.indexOf(')')
                if (i == -1 || j == -1 || k == -1) {
                    log.error("String [$s] does not have a valid Tag format")
                    return null
                }
                var g = s.substring(i + 1, j)
                var e = s.substring(j + 1, k)
                var groupMatch = false
                if (g.matches(pattern1)) {
                    groupMatch = true
                } else if (g.matches(pattern2)) {
                    g = g.substring(2)
                    groupMatch = true
                }
                var elementMatch = false
                if (e.matches(pattern1)) {
                    elementMatch = true
                } else if (e.matches(pattern2)) {
                    e = e.substring(2)
                    elementMatch = true
                }
                if (!groupMatch) {
                    log.error("Input string [$s] doesn't have a valid Tag format. Unable to determine the group value.")
                }
                if (!elementMatch) {
                    log.error("Input string [$s] doesn't have a valid Tag format. Unable to determine the element value")
                }
                if (!groupMatch || !elementMatch) {
                    return null
                }
                val gr = g.toInt(16)
                val el = e.toInt(16)
                if (gr > UShort.MAX_VALUE.toInt()) {
                    log.error("Input string [$s] doesn't have a valid Tag format. Group value is larger than ${UShort.MAX_VALUE}")
                    groupMatch = false
                }
                if (gr < 0) {
                    log.error("Input string [$s] doesn't have a valid Tag format. Group value must be larger or equal to 0.")
                    groupMatch = false
                }
                if (el > UShort.MAX_VALUE.toInt()) {
                    log.error("Input string [$s] doesn't have a valid Tag format. Element value is larger than ${UShort.MAX_VALUE}")
                    elementMatch = false
                }
                if (el < 0) {
                    log.error("Input string [$s] doesn't have a valid Tag format. Element value must be larger or equal to 0.")
                    elementMatch = false
                }

                if (!groupMatch || !elementMatch) {
                    return null
                }
                return Tag(gr.toUShort(), el.toUShort())
            } catch (e: NumberFormatException) {
                return null
            } catch (e: IllegalArgumentException) {
                return null
            }

        }
    }

    constructor(g: Int, e: Int) : this() {
        if (g > UShort.MAX_VALUE.toInt()) {
            throw IllegalArgumentException("Group ($g) is > ${UShort.MAX_VALUE}")
        }
        if (e > UShort.MAX_VALUE.toInt()) {
            throw IllegalArgumentException("Element ($e) is > ${UShort.MAX_VALUE}")
        }
        if (g < 0) {
            throw IllegalArgumentException("Group ($g) is < 0")
        }
        if (e < 0) {
            throw IllegalArgumentException("Element ($e) is < 0")
        }
        group = g.toUShort()
        element = e.toUShort()
    }

    /**
     * Get an unsigned 32 bit representation of a [Tag]
     *
     * Value is computed by shifting the group 16 bits left followed by a bitwise OR operation with the element value.
     *
     * @return An unsigned 32 bit representation of a [Tag]
     */
    fun toUInt(): UInt {
        return group.toUInt() shl 16 or element.toUInt()
    }
}
