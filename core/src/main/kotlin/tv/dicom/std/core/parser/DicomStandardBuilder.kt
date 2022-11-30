package tv.dicom.std.core.parser

import org.w3c.dom.Document
import tv.dicom.std.core.model.DicomStandard
import java.util.*

/**
 * Build the DicomStandard model from an XML document.
 *
 * @param document XML document of the DICOM standard
 * @return If successful, a [DicomStandard] is returned.
 * Otherwise, an empty [Optional] is returned.
 */
fun build(document: Document): Optional<DicomStandard> {
    val root = document.documentElement


    return Optional.empty()
}
