use crate::dom::model::QualifiedName;

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("XML parser: last element name {0} on stack doesn't match with the end element {1}.")]
    ParserEndMismatch(QualifiedName, QualifiedName),
    #[error("No element on stack, can add text: {0}")]
    ParserStackEmptyText(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Expected XML element: {0}, but received: {1}")]
    XmlExpectedElement(QualifiedName, QualifiedName),
    #[error("XML element not found: {0}")]
    XmlElementNotFound(QualifiedName),
    #[error("{1} XML [{0}] elements found but expected one.")]
    XmlElementMultipleFound(String, usize),
    #[error("XML tables misses a tbody element.")]
    XmlTableHasNoTbody,
    #[error("XML table has not the expected [{0}] number of columns [{1}]")]
    XmlTableInvalidNumberOfColumns(String, usize),
    #[error("XML table column failed to parse column: {0}")]
    XmlTableColumnParse(String),
    #[error("XML table header doesn't match with an IOD module table.")]
    NotIodModuleTable,
    #[error("No existing IOD module to append item to")]
    NoIodModuleToAppendItem,
    #[error("XML table header doesn't match with a data elements registery table.")]
    NotDataElementRegistery,
    #[error(transparent)]
    QuickXml {
        #[from]
        source: quick_xml::Error,
    },
    #[error(transparent)]
    QualifiedName {
        #[from]
        source: QualifiedNameError,
    },
    #[error(transparent)]
    Utf8Error {
        #[from]
        source: std::str::Utf8Error,
    },
    #[error(transparent)]
    FromUtf8Error {
        #[from]
        source: std::string::FromUtf8Error,
    },
    #[error(transparent)]
    TagError {
        #[from]
        source: dicom_std_core::TagError,
    },
    #[error(transparent)]
    TagRangeError {
        #[from]
        source: dicom_std_core::TagRangeError,
    },
    #[error(transparent)]
    IodModuleTypeError {
        #[from]
        source: dicom_std_core::IodModuleTypeError,
    },
    #[error(transparent)]
    VRError {
        #[from]
        source: dicom_std_core::VRError,
    },
    #[error(transparent)]
    VMError {
        #[from]
        source: dicom_std_core::VMError,
    },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    DownloadError {
        #[from]
        source: dicom_std_fetch::DownloadError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum IodModuleItemError {
    #[error("No existing IOD module item to append item to")]
    NoItemToAppend,
    #[error("Overflow detected, trying to add an item to an IodModuleItem while the expected number of items has been reached [{0}].")]
    Overflow(usize),
}

#[derive(thiserror::Error, Debug)]
pub enum QualifiedNameError {
    #[error("AttributeHasNoName")]
    AttributeNoName,
}

#[derive(thiserror::Error, Debug)]
pub enum ElementError {
    #[error("Element can't be created from a Node")]
    FromNodeToElement,
    #[error(transparent)]
    QuickXml {
        #[from]
        source: quick_xml::Error,
    },
    #[error(transparent)]
    ParserError {
        #[from]
        source: ParserError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Link not found error: {0}")]
    LinkNotFound(String),
    #[error(transparent)]
    QualifiedName {
        #[from]
        source: QualifiedNameError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum IODLibraryError {
    #[error(transparent)]
    QualifiedName {
        #[from]
        source: QualifiedNameError,
    },
    #[error(transparent)]
    Parser {
        #[from]
        source: ParserError,
    },
    #[error(transparent)]
    Query {
        #[from]
        source: QueryError,
    },
}
