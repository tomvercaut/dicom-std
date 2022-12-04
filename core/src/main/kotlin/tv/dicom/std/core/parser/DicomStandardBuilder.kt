package tv.dicom.std.core.parser

import org.apache.logging.log4j.kotlin.logger
import org.w3c.dom.*
import tv.dicom.std.core.Resource
import tv.dicom.std.core.model.DicomStandard
import tv.dicom.std.core.model.Usage
import tv.dicom.std.core.model.XRef
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.ciod.Entry
import tv.dicom.std.core.model.imd.Imd
import java.util.*
import javax.xml.xpath.XPathConstants
import javax.xml.xpath.XPathFactory


private val xPathFactory: XPathFactory = XPathFactory.newInstance()
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
internal fun buildPart03(document: Document, dicomStandard: DicomStandard): Boolean {
    val root = document.documentElement
    // Get Chapter A
    // Composite Information Object Definitions (Normative)
    val optChapterA = findElement(root, "//chapter[@id='chapter_A']")
    if (optChapterA.isEmpty) {
        log.error(Resource.errorMessage("DicomPart03ChapterAMissing"))
        return false
    }
    val chapterA = optChapterA.get()
    buildCiods(chapterA)

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


/**
 * Build all Composite Information Object Definitions (CIOD)s.
 *
 * The function looks for all tables within [parent] (for example the XML element matching the expression `//chapter[@id='chapter_A']`) and tries to build [Ciod] instances from them.
 *
 * @param parent XML DOM element that stores all CIOD tables in it's descendants
 * @return A [List] of [Ciod] instances is returned.
 */
internal fun buildCiods(parent: Element): List<Ciod> {
    val ciods = mutableListOf<Ciod>()

    val optTables = findElements(parent, ".//table")
    if (optTables.isEmpty) {
        return ciods
    }
    val tables = optTables.get()
    for (table in tables) {
        val opt = buildCiod(table)
        if (opt.isEmpty) {
            val id = table.getAttribute("xml:id")
            log.error("XML table [${id}] is no valid CIOD table")
            continue
        }
        val ciod = opt.get()
        ciods.add(ciod)
    }

    return ciods
}

/**
 * Build a Composite Information Object Definition (CIOD) from a XML table.
 *
 * The input XML element is expected to be a `table`. The table header is validated to match the pattern of CIOD table header. Each table row is modeled as an [Ciod] entry using the [buildCiodEntry] function.
 *
 * @param table XML DOM element of a table
 * @return If no error were detected, an [Optional] of [Ciod] is returned. If an error is detected an empty [Optional] is returned.
 */
internal fun buildCiod(table: Element): Optional<Ciod> {
    val ciod = Ciod()
    if (table.nodeName != "table") {
        return Optional.empty()
    }
    val id = table.getAttribute("xml:id")
    if (id.isBlank()) {
        log.error("XML table has no id attribute which is required for the implementation.")
        return Optional.empty()
    }
    ciod.id = id
    ciod.parentIds = getParentXmlId(table)
    if (!hasCiodTableHeader(table)) {
        log.error("XML table [${ciod.id}] has no matching CIOD table header")
        return Optional.empty()
    }
    val optTrs = findElements(table, "tbody/tr")
    if (optTrs.isEmpty) {
        log.error("XML table [${ciod.id}] has no rows")
        return Optional.empty()
    }
    val trs = optTrs.get()
    for (i in trs.indices) {
        val optEntry = buildCiodEntry(trs[i])
        if (optEntry.isEmpty) {
            log.error("XML table [${ciod.id}] rows [$i] does not contain a valid CIOD entry")
            return Optional.empty()
        }
        val entry = optEntry.get()
        if (entry.ie.isBlank()) {
            // not all CIOD entries have an IE value.
            // Lookup the IE value by reverse iterating the previously added entries.
            for (j in ciod.items.size - 1 downTo 0) {
                val prev = ciod.items[j]
                if (prev.ie.isNotBlank()) {
                    entry.ie = prev.ie
                    break
                }
            }
        }
        if (entry.ie.isBlank()) {
            log.error("XML table [${ciod.id}] rows [$i] does not contain a valid CIOD entry. Unable to set a matching 'ie' property")
            return Optional.empty()
        }
        ciod.items.add(entry)
    }

    return Optional.of(ciod)
}

/**
 * Create a Composite Information Object Definition from an XML table row.
 *
 * The function validates the input Element name is equal to `tr` and that the number of columns [`td`] is equal to 3 or 4. In case the number of columns equals 3, the function assumes the IE column is empty and not present in the XML structure.
 *
 * @param tr XML table row Element
 * @return An optional CIOD entry is returned.
 */
internal fun buildCiodEntry(tr: Element): Optional<Entry> {
    if (tr.nodeName != "tr") {
        log.error("XML Element is not a table row element (tr)")
        return Optional.empty()
    }
    val optTd = findElements(tr, "td")
    if (optTd.isEmpty) {
        log.error("Table row does not contain table columns.")
        return Optional.empty()
    }
    val tds = optTd.get()
    if (tds.size != 3 && tds.size != 4) {
        log.error("Table row contains an unsupported number of columns [${tds.size}]")
        return Optional.empty()
    }

    val entry = Entry()
    if (tds.size == 3) {
        entry.module = trimWsNl(tds[0].textContent)
        entry.reference = xref(tds[1])
        val usage = ciodUsage(tds[2])
        if (usage == null) {
            log.error("Table row contains an unsupported entry in the usage column [${tds[2]}]")
            return Optional.empty()
        }
        entry.usage = usage
    }
    if (tds.size == 4) {
        entry.ie = trimWsNl(tds[0].textContent)
        entry.module = trimWsNl(tds[1].textContent)
        entry.reference = xref(tds[2])
        val usage = ciodUsage(tds[3])
        if (usage == null) {
            log.error("Table row contains an unsupported entry in the usage column [${tds[3]}]")
            return Optional.empty()
        }
        entry.usage = usage
    }
    return Optional.of(entry)
}

/**
 * Determine the Usage value of an XML [Element].
 *
 * The XML [Element] contains a valid [Usage] if the text content after trimming starts with:
 * * M
 * * U
 * * C
 *
 * The text content is a representation of the [element] and it's descendants.
 *
 * @param element XML DOM element
 * @return If a valid pattern is detected the corresponding [Usage] is returned, otherwise null is returned.
 */
internal fun ciodUsage(element: Element): Usage? {
    val s = trimWsNl(element.textContent)
    return if (s.startsWith('M')) {
        Usage.M
    } else if (s.startsWith('U')) {
        Usage.U
    } else if (s.startsWith('C')) {
        Usage.C
    } else {
        log.error("")
        null
    }
}

/**
 * Create an [XRef] model from an xref XML [element] in the DICOM standard.
 *
 * @param element XML DOM element
 * @param nested search for a nested xref element if the name of [element] is not "xref"
 * @return An [XRef] model is returned. If the attributes `linked` and `xrefstyle` exists the corresponding values are set within the model.
 */
internal fun xref(element: Element, nested: Boolean = true): XRef {
    var xre: Element? = null
    if (element.nodeName == "xref") {
        xre = element
    } else if (nested) {
        val opt = findElement(element, ".//xref")
        if (opt.isPresent) {
            xre = opt.get()
        }
    }
    val xr = XRef()
    xr.linkend = xre?.getAttribute("linkend") ?: ""
    xr.style = xre?.getAttribute("xrefstyle") ?: ""
    return xr
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
internal fun hasCiodTableHeader(table: Element): Boolean {
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

internal fun buildImds(root: Element, parent: Element): List<Imd> {
    val imds = mutableListOf<Imd>()

    return imds
}

internal fun buildImd(root: Element, parent: Element): Optional<Imd> {
    val imd = Imd()

    return Optional.of(imd)
}

/**
 * Get a list of all parent [Element]s of [element] and get the attribute `xml:id`.
 *
 * @param element XML DOM element
 * @return List with all the xml:id attributes from the parent nodes.
 */
internal fun getParentXmlId(element: Element): List<String> {
    val pids = mutableListOf<String>()
    var parent: Node? = element.parentNode
    while (parent != null) {
        val attrs = parent.attributes
        if (attrs != null) {
            val attr = attrs.getNamedItem("xml:id")
            if (attr != null) {
                val value = attr.nodeValue
                if (value != null) {
                    pids.add(value)
                }
            }
        }
        parent = parent.parentNode
    }
    return pids
}

/**
 * Test if an XML node is an [Element].
 *
 * @param node XML DOM node
 * @return True if the node is an [Element], false otherwise.
 */
internal fun isElementType(node: Node?): Boolean {
    return if (node == null) {
        false
    } else {
        node.nodeType == Node.ELEMENT_NODE
    }
}

/**
 * Find an XML [Node] based on an XPath expression.
 *
 * @param node XML DOM Node
 * @param expression XPath expression
 * @return If a matching [Node] is found an [Optional] of the [Node] is returned. Otherwise an [Optional.empty] is returned.
 */
internal fun findNode(node: Element, expression: String): Optional<Node> {
    return try {
        Optional.of(xPathFactory.newXPath().compile(expression).evaluate(node, XPathConstants.NODE) as Node)
    } catch (ex: NullPointerException) {
        Optional.empty()
    }
}

/**
 * Find an XML [Element] based on an XPath expression.
 *
 * @param node XML DOM Node
 * @param expression XPath expression
 * @return If a matching [Element] is found an [Optional] of the [Element] is returned. Otherwise an [Optional.empty] is returned.
 */
internal fun findElement(node: Node, expression: String): Optional<Element> {
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

/**
 * Find a list of XML [Node]s based on an XPath expression.
 *
 * @param element XML DOM [Element]
 * @param expression XPath expression
 * @return If a matching [NodeList] is found an [Optional] of the [NodeList] is returned. Otherwise an [Optional.empty] is returned.
 */
internal fun findNodes(element: Element, expression: String): Optional<NodeList> {
    return try {
        val node = xPathFactory.newXPath().compile(expression).evaluate(element, XPathConstants.NODESET) as NodeList
        Optional.of(node)
    } catch (ex: NullPointerException) {
        Optional.empty()
    }
}

/**
 * Find a list of XML [Element]s based on an XPath expression.
 *
 * @param element XML DOM [Element]
 * @param expression XPath expression
 * @return If a matching [List] is found an [Optional] of the [List] is returned. Otherwise an [Optional.empty] is returned.
 */
internal fun findElements(element: Element, expression: String): Optional<List<Element>> {
    return try {
        val elements = mutableListOf<Element>()
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

/**
 * Trim whitespaces, newline and carriage returns at the beginning and end of a String.
 *
 * @param s string to be trimmed
 * @return A string in which the whitespaces, newline and carriage returns at the start and end of the string are removed.
 */
internal fun trimWsNl(s: String): String {
    return s.trim { c: Char -> c == '\n' || c == ' ' || c == '\r' }
}