package tv.dicom.std.core.parser

import org.w3c.dom.Document
import org.w3c.dom.Element
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import tv.dicom.std.core.model.DicomStandard
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.Imd
import java.util.*
import javax.xml.xpath.XPathConstants
import javax.xml.xpath.XPathFactory


val xPathFactory: XPathFactory = XPathFactory.newInstance()

/**
 * Build the DicomStandard model from a set of XML documents.
 *
 * @param documents a list of XML documents from the DICOM standard
 * @return If successful, a [DicomStandard] is returned.
 * Otherwise, an empty [Optional] is returned.
 */
fun build(documents: List<Document>): Optional<DicomStandard> {
    val dicomStandard = DicomStandard()
    for (document in documents) {
        val root = document.documentElement
        val rootId = root.getAttribute("id")

        if (rootId == "PS3.3") {
            buildPart03(document, dicomStandard)
        }
    }
    return Optional.empty()
}


/**
 * Build the DicomStandard model using the XML document from part 03 of the DICOM standard.
 *
 * @param document XML document of the DICOM standard
 * @return If successful, a [DicomStandard] is returned.
 * Otherwise, an empty [Optional] is returned.
 */
fun buildPart03(document: Document, dicomStandard: DicomStandard) {
    val root = document.documentElement
    // Get Chapter A
    // Composite Information Object Definitions (Normative)
    val optChapterA = findNode(root, "//chapter[@id='chapter_A']")
    if (optChapterA.isEmpty) {

        return
    }
    val chapterA = optChapterA.get()

    // Get Chapter C
    // Information Module Definitions (Normative)
    val optChapterC = findNode(root, "//chapter[@id='chapter_C']")
    if (optChapterC.isEmpty) {

        return
    }
    val chapterC = optChapterC.get()

}


fun buildCiods(root: Element, parent: Element): List<Ciod> {
    val ciods = mutableListOf<Ciod>()

    return ciods
}

fun buildCiod(root: Element, parent: Element): Ciod {
    val ciod = Ciod()

    return ciod
}

fun buildImds(root: Element, parent: Element): List<Imd> {
    val imds = mutableListOf<Imd>()

    return imds
}

fun buildImd(root: Element, parent: Element): Imd {
    val imd = Imd()

    return imd
}

private fun isElementType(node: Node?): Boolean {
    return if (node == null) {
        false
    } else {
        node.nodeType == Node.ELEMENT_NODE
    }
}

private fun findNode(element: Element, expression: String): Optional<Node> {
    return try {
        val node = xPathFactory.newXPath().compile(expression).evaluate(element, XPathConstants.NODE) as Node
        Optional.of(node)
    } catch (ex: NullPointerException) {
        Optional.empty()
    }
}

private fun findElement(node: Node, expression: String): Optional<Element> {
    return try {
        val result = xPathFactory.newXPath().compile(expression).evaluate(node, XPathConstants.NODE) as Node
        if (isElementType(result)) {
            return Optional.of(result as Element)
        } else {
            return Optional.empty()
        }
    } catch (ex: NullPointerException) {
        Optional.empty()
    }
}

private fun findNodes(element: Element, expression: String): Optional<NodeList> {
    return try {
        val node = xPathFactory.newXPath().compile(expression).evaluate(element, XPathConstants.NODESET) as NodeList
        Optional.of(node)
    } catch (ex: NullPointerException) {
        Optional.empty()
    }
}

private fun findElements(element: Element, expression: String): Optional<List<Element>> {
    return try {
        var elements = mutableListOf<Element>()
        val opt = findNodes(element, expression)
        if (opt.isEmpty) {
            return Optional.empty()
        }
        val nodes = opt.get()
        val length = nodes.length
        for (i in 0 until length) {
            val node = nodes.item(i)
            if (isElementType(node)) {
                elements.add(node as Element)
            }
        }
        Optional.of(elements)
    } catch (ex: NullPointerException) {
        Optional.empty()
    }
}