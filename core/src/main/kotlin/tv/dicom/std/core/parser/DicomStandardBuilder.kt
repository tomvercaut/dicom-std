package tv.dicom.std.core.parser

import org.apache.logging.log4j.kotlin.logger
import org.w3c.dom.Document
import org.w3c.dom.Element
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import tv.dicom.std.core.Resource
import tv.dicom.std.core.model.DicomStandard
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.imd.Imd
import java.util.*
import javax.xml.xpath.XPathConstants
import javax.xml.xpath.XPathFactory


val xPathFactory: XPathFactory = XPathFactory.newInstance()
private val log = logger("DicomStandardBuilder")

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
            if (!buildPart03(document, dicomStandard)) {
                return Optional.empty()
            }
        }
    }
    return Optional.of(dicomStandard)
}


/**
 * Build the DicomStandard model using the XML document from part 03 of the DICOM standard.
 *
 * @param document XML document of the DICOM standard
 * @return If successful, true is returned. If something went wrong false is returned.
 */
fun buildPart03(document: Document, dicomStandard: DicomStandard): Boolean {
    val root = document.documentElement
    // Get Chapter A
    // Composite Information Object Definitions (Normative)
    val optChapterA = findElement(root, "//chapter[@id='chapter_A']")
    if (optChapterA.isEmpty) {
        log.error(Resource.errorMessage("DicomPart03ChapterAMissing"))
        return false
    }
    val chapterA = optChapterA.get()
    buildCiods(root, chapterA)

    // Get Chapter C
    // Information Module Definitions (Normative)
    val optChapterC = findElement(root, "//chapter[@id='chapter_C']")
    if (optChapterC.isEmpty) {
        log.error(Resource.errorMessage("DicomPart03ChapterCMissing"))
        return false
    }
    val chapterC = optChapterC.get()
    buildImds(root, chapterC)

    return true
}


fun buildCiods(root: Element, parent: Element): List<Ciod> {
    val ciods = mutableListOf<Ciod>()

    val optTables = findElements(parent, "//table")
    if (optTables.isEmpty) {
        return ciods
    }
    val tables = optTables.get()
    for (table in tables) {
        val opt = buildCiod(root, table)
        if (opt.isEmpty) {
            continue
        }
        val ciod = opt.get()
        ciods.add(ciod)
    }

    return ciods
}

fun buildCiod(root: Element, table: Element): Optional<Ciod> {
    val ciod = Ciod()
    if (table.nodeName != "table") {
        return Optional.empty()
    }
    if (hasCiodTableHeader(table)) {

    }

    return Optional.of(ciod)
}

/**
 * Check if a table header matches with the one from a CIOD table.
 *
 * @param table CIOD XML table
 * @return True if it matches the following criteria:
 *   * no thead element
 *   * no tr element
 *   * the first tr element doesn't contain the
 */
private fun hasCiodTableHeader(table: Element): Boolean {
    val optThead = findElement(table, "thead")
    if (optThead.isEmpty) {
        return false
    }
    val thead = optThead.get()
    val optTrs = findElements(thead, "tr")
    if (optTrs.isEmpty) {
        return false
    }
    val trs = optTrs.get()
    if (trs.isEmpty()) {
        return false
    }
    for (i in trs.indices) {
        val optTds = findElements(trs[i], "th")
        if (optTds.isEmpty) {
            continue
        }
        val tds = optTds.get()
        if (tds.size != 4) {
            continue
        }
        val ie = tds[0].toString().lowercase()
        val module = tds[1].toString().lowercase()
        val reference = tds[2].toString().lowercase()
        val usage = tds[3].toString().lowercase()
        if (ie == "ie" || module == "module" || reference == "reference" || usage == "usage") {
            return true
        }
    }

    return true
}

fun buildImds(root: Element, parent: Element): List<Imd> {
    val imds = mutableListOf<Imd>()

    return imds
}

fun buildImd(root: Element, parent: Element): Optional<Imd> {
    val imd = Imd()

    return Optional.of(imd)
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