package tv.dicom.std.core.parser

import org.apache.logging.log4j.kotlin.logger
import org.w3c.dom.*
import tv.dicom.std.core.Resource
import tv.dicom.std.core.model.*
import tv.dicom.std.core.model.ciod.Ciod
import tv.dicom.std.core.model.ciod.Entry
import tv.dicom.std.core.model.imd.DataEntry
import tv.dicom.std.core.model.imd.Imd
import tv.dicom.std.core.model.imd.IncludeEntry
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
        val rootId = root.getAttribute("xml:id")

        if (rootId == "PS3.3") {
            if (!buildPart03(document, dicomStandard)) {
                return Optional.empty()
            }
            if (!findDependents(document, dicomStandard)) {
                log.error("Unable to find all dependent elements.")
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
    val ciods = buildCiods(chapterA)

    // Get Chapter C
    // Information Module Definitions (Normative)
    val optChapterC = findElement(root, "//chapter[@id='chapter_C']")
    if (optChapterC.isEmpty) {
        log.error(Resource.errorMessage("DicomPart03ChapterCMissing"))
        return false
    }
    val chapterC = optChapterC.get()
    val imds = buildImds(chapterC)

    for (ciod in ciods) {
        dicomStandard.add(ciod)
    }
    for (imd in imds) {
        dicomStandard.add(imd)
    }

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
 * Build a Composite Information Object Definition (CIOD) from an XML table.
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
    val caption = getCaption(table)
    if (caption.isNullOrBlank()) {
        log.error("XML table has no caption which is required for the implementation.")
        return Optional.empty()
    }
    ciod.caption = caption
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
        entry.module = attributeName(trimWsNl(tds[0].textContent))
        entry.reference = xref(tds[1])
        val usage = ciodUsage(tds[2])
        if (usage == null) {
            log.error("Table row contains an unsupported entry in the usage column [${tds[2]}]")
            return Optional.empty()
        }
        entry.usage = usage
    }
    if (tds.size == 4) {
        entry.ie = attributeName(trimWsNl(tds[0].textContent))
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
 * Create an [XRef] model from an {@code xref} XML [element] in the DICOM standard.
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
 * Get table header rows of an XML table header rows.
 *
 * @param table XML table [Element]
 * @return If the table header rows exists they are returned as an Optional. If no thead element or thead/tr elements exists, an empty [Optional] is returned.
 */
private fun getTableHeaderRows(table: Element): Optional<List<Element>> {
    val optThead = findElement(table, "thead")
    if (optThead.isEmpty) {
        return Optional.empty()
    }
    val thead = optThead.get()
    return findElements(thead, "tr")
}

/**
 * Check if a table header matches with the one from a CIOD table.
 *
 * @param table CIOD XML table
 * @return True if it matches the following criteria:
 *   * has a table header
 *   * table header has at least one row with 4 columns
 *   * matching column names (case-insensitive): ie, module, reference, usage
 */
internal fun hasCiodTableHeader(table: Element): Boolean {
    val opt = getTableHeaderRows(table)
    if (opt.isEmpty || opt.get().isEmpty()) {
        return false
    }
    val trs = opt.get()
    for (i in trs.indices) {
        val optTds = findElements(trs[i], "th")
        if (optTds.isEmpty) {
            continue
        }
        val tds = optTds.get()
        if (tds.size != 4) {
            continue
        }
        val ie = trimWsNl(tds[0].textContent.lowercase())
        val module = trimWsNl(tds[1].textContent.lowercase())
        val reference = trimWsNl(tds[2].textContent.lowercase())
        val usage = trimWsNl(tds[3].textContent.lowercase())
        if (ie == "ie" && module == "module" && reference == "reference" && usage == "usage") {
            return true
        }
    }

    return false
}

/**
 * Check if a table header matches with the one from an Information Module Definition (IMD) table.
 *
 * @param table IMD XML table
 * @return True if it matches the following criteria:
 *   * no {@code thead} element
 *   * no {@code tr} element
 *   * not one {@code tr} does not have the following formats:
 *      - | attribute name | tag | type | attribute description
 *      - | attribute name | tag | attribute description
 */
internal fun hasImdTableHeader(table: Element): Boolean {
    val opt = getTableHeaderRows(table)
    if (opt.isEmpty || opt.get().isEmpty()) {
        return false
    }
    val trs = opt.get()
    for (i in trs.indices) {
        val optTds = findElements(trs[i], "th")
        if (optTds.isEmpty) {
            continue
        }
        val tds = optTds.get()
        if (tds.size == 4) {
            val attrName = trimWsNl(tds[0].textContent.lowercase())
            val tag = trimWsNl(tds[1].textContent.lowercase())
            val type = trimWsNl(tds[2].textContent.lowercase())
            val attrDesc = trimWsNl(tds[3].textContent.lowercase())
            if (attrName.contains("attribute name") &&
                tag.contains("tag") &&
                type.contains("type") &&
                attrDesc.contains("attribute description")
            ) {
                return true
            }
        } else if (tds.size == 3) {
            val attrName = trimWsNl(tds[0].textContent.lowercase())
            val tag = trimWsNl(tds[1].textContent.lowercase())
            val attrDesc = trimWsNl(tds[2].textContent.lowercase())
            if (attrName.contains("attribute name") &&
                tag.contains("tag") &&
                attrDesc.contains("attribute description")
            ) {
                return true
            }
        }

    }

    return false
}

/**
 * Build all Information Module Definitions (IMD)s.
 *
 * The function looks for all tables within [parent] (for example the XML element matching the expression `//chapter[@id='chapter_C']`) and tries to build [Imd] instances from them.
 *
 * @param parent XML DOM element that stores all IMD tables in it's descendants
 * @return A [List] of [tv.dicom.std.core.model.imd.Entry] instances is returned.
 */
internal fun buildImds(parent: Element): List<Imd> {
    val imds = mutableListOf<Imd>()
    val optTables = findElements(parent, ".//table")
    if (optTables.isEmpty) {
        return imds
    }
    val tables = optTables.get()
    for (table in tables) {
        val opt = buildImd(table)
        if (opt.isEmpty) {
            val id = table.getAttribute("xml:id")
            log.error("XML table [${id}] is no valid IMD table")
            continue
        }
        val imd = opt.get()
        imds.add(imd)
    }
    return imds
}

/**
 * Create an Information Module Definition from a DICOM XML table.
 *
 * An empty `Optional` is returned in the following cases:
 * - the table `Element` is not a table
 * - the XML element attribute `xml:id` is empty
 * - the table has no rows
 * - the table has a row entry that is not valid an Information Module Definition entry.
 *
 * @param table XML table element
 * @return When the creation of the Information Module Definition was successful, it's returned.
 * Otherwise, an empty `Optional` is returned.
 */
internal fun buildImd(table: Element): Optional<Imd> {
    val imd = Imd()
    if (table.nodeName != "table") {
        return Optional.empty()
    }
    val id = table.getAttribute("xml:id")
    if (id.isBlank()) {
        log.error("XML table has no id attribute which is required for the implementation.")
        return Optional.empty()
    }
    imd.id = id
    imd.parentIds = getParentXmlId(table)
    val caption = getCaption(table)
    if (caption.isNullOrBlank()) {
        log.error("XML table has no caption which is required for the implementation.")
        return Optional.empty()
    }
    imd.caption = caption
    if (!hasImdTableHeader(table)) {
        log.error("XML table [${imd.id}] has no matching IMD table header")
        return Optional.empty()
    }
    val optTrs = findElements(table, "tbody/tr")
    if (optTrs.isEmpty) {
        log.error("XML table [${imd.id}] has no rows")
        return Optional.empty()
    }
    val trs = optTrs.get()
    for (i in trs.indices) {
        val optEntry = buildImdEntry(trs[i])
        if (optEntry.isEmpty) {
            log.error("XML table [${imd.id}] rows [$i] does not contain a valid IMD entry")
            return Optional.empty()
        }
        imd.items.add(optEntry.get())
    }
    return Optional.of(imd)
}

/**
 * Iterates the Information Module Definitions in the DicomStandard items.
 *
 * Foreach `IncludeEntry` find the matching table identifier in the
 * Composite Information Object Definition list or in the Information
 * Module Definition list. If the identifier is not found, look in the XML document.
 *
 * The missing identifier is found in the `Document`, it's added to the `DicomStandard`.
 * If the missing identifier is not found in the `Document`,
 * the function returns false to indicate not all identifiers have been found.
 *
 * The function returns true when all internal links have been found.
 *
 * @param document Part 03 of the DICOM standard
 * @param dicomStandard model for the DICOM standard
 * @return True when all include links have been found and added to the `DicomStandard`.
 * False if not all include links have been found.
 */
internal fun findDependents(document: Document, dicomStandard: DicomStandard): Boolean {
    val ciodIds = dicomStandard.ciodIds()
    var imdIds = dicomStandard.imdIds()

    val notImd = mutableListOf<String>()
    val notFound = mutableListOf<String>()

    var i = 0
    var size = imdIds.size
    var isOk = true
    while (i < size) {
        val elementId = imdIds.elementAt(i)
        val imd = dicomStandard.imd(elementId) ?: continue
        for (item in imd.items) {
            if (!item.isInclude()) {
                continue
            }
            val linkId = (item as IncludeEntry).xref.linkend
            if (linkId.isBlank()) {
                continue
            }
            if (ciodIds.contains(linkId) || imdIds.contains(linkId) ||
                notImd.contains(linkId) || notFound.contains(linkId)
            ) {
                continue
            }
            val expression = "//table[@id=\'$linkId\']"
            val optElement = findElement(document.documentElement, expression)
            if (optElement.isEmpty) {
                notFound.add(linkId)
                log.error("Unable to find XML id: $linkId")
                isOk = false
                continue
            }
            val element = optElement.get()
            val optImd = buildImd(element)
            if (optImd.isEmpty) {
                log.warn("Unable to build Information Module Definition or attribute table from: $linkId")
                notImd.add(linkId)
            } else {
                val newImd = optImd.get()
                dicomStandard.add(newImd)
                imdIds = dicomStandard.imdIds()
                size = imdIds.size
            }
        }

        i += 1
    }
    return isOk
}


/**
 * Determine the depth of a sequence item in an XML table entry.
 *
 * The function counts the larger than characters at the beginning of the string. Newlines, return characters and whitespaces are discarded while counting.
 *
 * # Examples
 *
 * * `entry` => 0
 * * `>entry` => 1
 * * `>>entry` => 2
 *
 * @param s input string
 * @return Depth of an item in a (nested) sequence
 */
internal fun sequenceItemDepth(s: String): UShort {
    var indent: UShort = 0U
    for (i in s.indices) {
        if (s[i] == '>') {
            ++indent
        } else if (s[i] == ' ' || s[i] == '\n' || s[i] == '\r') {
            // ignore whitespaces, newline characters and return carriages at the beginning of the string
            continue
        } else {
            break
        }
    }
    return indent
}

/**
 * Test if a string (like an Attribute Name) has a substring with `Include`.
 *
 * The characters before `Include` can only be:
 * * newline
 * * carriage return
 * * whitespace
 * * larger than character '>'
 *
 * @param s input string
 * @return True if the input string complies with the stated requirements, otherwise false is returned.
 */
internal fun attributeHasInclude(s: String): Boolean {
    val index = s.indexOf("Include")
    if (index == -1) {
        return false
    }
    for (i in 0 until index) {
        if (s[i] != '>' && s[i] != ' ' && s[i] != '\n' && s[i] != '\r') {
            return false
        }
    }
    return true
}

/**
 * Get the name of an attribute.
 *
 * These characters at the start of the String are excluded from the name:
 * * newline
 * * carriage return
 * * whitespace
 * * larger than character '>'
 *
 * @param s input string
 * @return Name of the attribute
 */
internal fun attributeName(s: String): String {
    var index = -1
    for (i in s.indices) {
        val c = s[i]
        if (c != '>' && c != ' ' && c != '\n' && c != '\r') {
            index = i
            break
        }
    }
    if (index == -1) {
        return ""
    }
    var t = s.substring(index)
    t = t.replace("[ \n\r\t.]+".toRegex(), " ")
    return t
}

/**
 * Build an Information Module Definition entry from an XML table row.
 *
 * @param tr XML table row
 * @return Entry can be either a [DataEntry] or an [IncludeEntry] depending on the data in the table row.
 */
internal fun buildImdEntry(tr: Element): Optional<tv.dicom.std.core.model.imd.Entry> {
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
    if (tds.isEmpty() || tds.size > 4) {
        log.error("Table row contains an unsupported number of columns [${tds.size}]")
        return Optional.empty()
    }

    if (tds.size == 1 || tds.size == 2) {
        val entry = IncludeEntry()
        entry.seqIndent = sequenceItemDepth(tds[0].textContent)
        if (attributeHasInclude(tds[0].textContent)) {
            entry.xref = xref(tds[0])
        }
        if (tds.size == 2) {
            entry.description = trimWsNl(tds[1].textContent)
        }
        return Optional.of(entry as tv.dicom.std.core.model.imd.Entry)
    } else { // if (tds.size == 3 || tds.size == 4) {
        val entry = DataEntry()
        entry.seqIndent = sequenceItemDepth(tds[0].textContent)
        entry.name = attributeName(trimWsNl(tds[0].textContent))
        val tag = Tag.of(tds[1].textContent)
        if (tag == null) {
            log.error("Table row contains an unsupported entry in the Tag column [${tds[1]}]")
            return Optional.empty()
        }
        entry.tag = tag
        entry.description = trimWsNl(tds[2].textContent)
        if (tds.size == 4) {
            val type = attributeTypeFromString(trimWsNl(tds[2].textContent))
            if (type == null) {
                log.error("Table row contains an unsupported entry in the Type column [${tds[2]}]")
                return Optional.empty()
            }
            entry.type = type
            entry.description = trimWsNl(tds[3].textContent)
        }
        return Optional.of(entry)
    }
}

/**
 * Get the caption of an XML table.
 *
 * @param table XML DOM element
 * @return Caption of an XML table.
 * Null is returned the XML DOM element is not a table or if a nested caption element is not found.
 */
internal fun getCaption(table: Element): String? {
    if (table.nodeName != "table") {
        log.error("Invalid function argument: expected a table element but got a ${table.nodeName}")
        return null
    }
    val opt = findElement(table, "caption")
    if (opt.isEmpty) {
        log.error("Table doesn't have a caption.")
        return null
    }
    val caption = opt.get()
    return trimWsNl(caption.textContent)
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
 * Find an XML [Element] based on an XPath expression.
 *
 * @param node XML DOM Node
 * @param expression XPath expression
 * @return If a matching [Element] is found an [Optional] of the [Element] is returned. Otherwise, an [Optional.empty] is returned.
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
 * @return If a matching [NodeList] is found an [Optional] of the [NodeList] is returned. Otherwise, an [Optional.empty] is returned.
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
 * @return If a matching [List] is found an [Optional] of the [List] is returned. Otherwise, an [Optional.empty] is returned.
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