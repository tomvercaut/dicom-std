package tv.dicom.std.core.parser

import org.w3c.dom.Document
import org.xml.sax.InputSource
import tv.dicom.std.core.model.DicomStandard

import java.io.File
import java.io.StringReader
import java.util.Optional
import javax.xml.parsers.DocumentBuilderFactory

/**
 * Parse an XML file from the DICOM standard and create a [DicomStandard].
 *
 * If an error occurs during the build phase of the standard an empty optional is returned.
 *
 * @param file XML file
 * @return If no errors were encountered an [Optional] [DicomStandard] is returned.
 */
fun parse(file: File): Optional<DicomStandard> {
    val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
    val document = builder.parse(file)
    return build(listOf(document))
}

/**
 * Parse an XML string from the DICOM standard and create a [DicomStandard].
 *
 * If an error occurs during the build phase of the standard an empty optional is returned.
 *
 * @param xmlString XML representation of the DICOM standard.
 * @return If no errors were encountered an [Optional] [DicomStandard] is returned.
 */
fun parse(xmlString: String): Optional<DicomStandard> {
    val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
    val document = builder.parse(InputSource(StringReader(xmlString)))
    return build(listOf(document))
}