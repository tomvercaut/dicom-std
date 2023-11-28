package tv.dicom.std.model

// DICOM Attribute Requirement Types
enum class AttributeType {
    // Required to be in the SOP Instance and shall have a valid value.
    Type1,
    // Required to be in the SOP Instance but may contain the value of "unknown", or a zero length value.
    Type2,
    // Optional. May or may not be included and could be zero length.
    Type3,
    // Conditional. If a condition is met, then it is a Type 1 (required, cannot be zero). If condition is not met, then the tag is not sent.
    Type1C,
    // Conditional. If condition is met, then it is a Type 2 (required, zero length OK). If condition is not met, then the tag is not sent.
    Type2C,
}

/**
 * Create a DICOM attribute type from string.
 *
 * Possible matching values:
 * * 1 => [AttributeType.Type1]
 * * 2 => [AttributeType.Type2]
 * * 3 => [AttributeType.Type3]
 * * 1C => [AttributeType.Type1C]
 * * 2C => [AttributeType.Type2C]
 * * other values => null
 *
 * @param s input string
 * @return If a matching pattern is found, an AttributeType is returned, otherwise null is returned.
 */
fun attributeTypeFromString(s: String) : AttributeType? {
    when (s) {
        "1" -> {
            return AttributeType.Type1
        }
        "2" -> {
            return AttributeType.Type2
        }
        "3" -> {
            return AttributeType.Type3
        }
        "1C" -> {
            return AttributeType.Type1C
        }
        "2C" -> {
            return AttributeType.Type2C
        }
        else -> return null
    }
}