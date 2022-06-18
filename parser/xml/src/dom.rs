use std::collections::BTreeMap;
use std::io::BufRead;
use std::path::Path;
use std::str::FromStr;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader as EventReader;

use crate::ParserError;

fn build_element<R>(
    e: &BytesStart,
    reader: &mut EventReader<R>,
) -> Result<model::Element, ParserError>
where
    R: BufRead,
{
    let name = std::str::from_utf8(e.name())?;
    let qn = model::QualifiedName::from_str(name)?;

    let attrs = e
        .attributes()
        .map(|a| {
            let a = a?;
            let key =
                std::str::from_utf8(a.key).map_err(|er| ParserError::Utf8Error { source: er })?;
            let qkey = model::QualifiedName::from_str(key)?;
            let value = a.unescape_and_decode_value(reader)?;
            Ok((qkey, value))
        })
        .collect::<Result<BTreeMap<model::QualifiedName, String>, ParserError>>()?;

    Ok(model::Element {
        name: qn,
        attrs,
        children: vec![],
    })
}

/// Create a Document Object Model from an XML file and return the root element if successful.
///
/// # Arguments
///
/// * `path`: file path to an XML file
///
/// returns: Result<Element, ParserError>
pub fn read_file<P>(path: P) -> Result<model::Element, ParserError>
where
    P: AsRef<Path>,
{
    let mut rdr = quick_xml::Reader::from_file(path)?;
    from_reader(&mut rdr)
}

/// Create a Document Object Model from an XML string and return the root element if successful.
///
/// # Arguments
///
/// * `s`: XML string
///
/// returns: Result<Element, ParserError>
pub fn read_str(s: &str) -> Result<model::Element, ParserError> {
    let mut rdr = quick_xml::Reader::from_str(s);
    from_reader(&mut rdr)
}

/// Create a Document Object Model from an EventReader and return the root element if successful.
///
/// # Arguments
///
/// * `reader`: XML event reader
///
/// returns: Result<Element, ParserError>
pub fn from_reader<R>(
    reader: &mut EventReader<R>,
) -> Result<model::Element, ParserError>
where
    R: BufRead,
{
    let mut buf = Vec::new();
    let mut stack = vec![];

    loop {
        match reader.read_event(&mut buf) {
            Ok(event) => match event {
                Event::Start(ref e) => {
                    let elem = build_element(e, reader)?;
                    stack.push(elem);
                }
                Event::End(ref e) => {
                    let n = stack.len();
                    if n <= 1 {
                        break;
                    }
                    let name = std::str::from_utf8(e.name())?;
                    let qn = model::QualifiedName::from_str(name)?;
                    let elem = stack.pop().unwrap();
                    if elem.name != qn {
                        return Err(ParserError::ParserEndMismatch(qn, elem.name));
                    }
                    let m = stack.len();
                    assert!(
                        m > 0,
                        "stack must have at least one Element on it to append a child Node."
                    );
                    let last = stack.get_mut(m - 1).unwrap();
                    last.children.push(model::Node::Element(elem));
                }
                Event::Empty(ref e) => {
                    let n = stack.len();
                    if n < 1 {
                        return Err(ParserError::ParseError(
                            "attempting to add an empty element to an empty stack.".to_string(),
                        ));
                    }
                    let elem = build_element(e, reader)?;
                    let last = stack.get_mut(n - 1).unwrap();
                    last.children.push(model::Node::Element(elem));
                }
                Event::Text(s) => {
                    let n = stack.len();
                    let text = s.unescape_and_decode(reader)?;
                    if n == 0 {
                        continue;
                    }
                    let last = stack.get_mut(n - 1).unwrap();
                    last.children.push(model::Node::Text(text));
                }
                Event::Comment(_) => {}
                Event::CData(_) => {}
                Event::Decl(_) => {}
                Event::PI(_) => {}
                Event::DocType(_) => {}
                Event::Eof => {
                    let n = stack.len();
                    if n == 0 {
                        return Err(ParserError::ParseError(
                            "EOF reached, no element on stack".to_string(),
                        ));
                    }
                    break;
                }
            },
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    let n = stack.len();
    if n != 1 {
        return Err(ParserError::ParseError(format!(
            "expected the stack to only contain the root element but {} were found",
            n
        )));
    }
    let root = stack.pop().unwrap();
    Ok(root)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    const XML_SAMPLE_01: &str = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
<book xmlns="http://docbook.org/ns/docbook" xmlns:xl="http://www.w3.org/1999/xlink" label="PS3.3" version="5.0" xml:id="PS3.3">
  <title>PS3.3</title>
  <subtitle>DICOM PS3.3 2021c - Information Object Definitions</subtitle>
  <info>
    <author>
      <orgname>DICOM Standards Committee</orgname>
    </author>
    <copyright>
      <year>2021</year>
      <!--<biblioid class="other" otherclass="patentnum">6,272,235</biblioid>-->
      <holder>NEMA</holder>
    </copyright>
      <legalnotice>
          <para xml:id="para_80687a13-9d6e-433a-b7a4-36746d82979e">A DICOM® publication</para>
      </legalnotice>
  </info>
</book>
 "#;

    #[test]
    fn read_xml01() {
        let r = super::model::Element::from_str(XML_SAMPLE_01);
        assert!(r.is_ok(), "{}", r.err().unwrap());
    }
}

pub mod model {
    use std::cmp::Ordering;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use crate::dom::from_reader;
    use crate::{is_char_whitespace_or_return, query, ElementError, QualifiedNameError};

    /// Represents a Qualified XML name.
    ///
    /// An XML qualified name contains a namespace and a local name in the format of `namespace:localname`.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use dicom_std_xml_parser::dom::model::QualifiedName;
    /// let qn = QualifiedName::from_str("ns:nns:name").unwrap();
    /// # assert_eq!(qn.ns.as_str(), "ns:nns");
    /// # assert_eq!(qn.local.as_str(), "name");
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct QualifiedName {
        /// XML namespace
        pub ns: String,
        /// local XML name
        pub local: String,
    }

    impl Default for QualifiedName {
        fn default() -> Self {
            Self {
                ns: "".to_string(),
                local: "".to_string(),
            }
        }
    }

    impl FromStr for QualifiedName {
        type Err = QualifiedNameError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let pos = s.rfind(':');
            let n = s.len();
            if n == 0 {
                return Err(Self::Err::AttributeNoName);
            }
            match pos {
                None => Ok(Self {
                    ns: "".to_string(),
                    local: s.to_string(),
                }),
                Some(p) => {
                    let n = s.len();
                    if p == n - 1 {
                        Err(Self::Err::AttributeNoName)
                    } else {
                        Ok(Self {
                            ns: (&s[0..p]).to_string(),
                            local: (&s[p + 1..]).to_string(),
                        })
                    }
                }
            }
        }
    }

    impl std::fmt::Display for QualifiedName {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            if self.ns.is_empty() {
                write!(f, "{}", &self.local)
            } else {
                write!(f, "{}:{}", &self.ns, &self.local)
            }
        }
    }

    impl PartialOrd<Self> for QualifiedName {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            match self.ns.partial_cmp(&other.ns) {
                None => self.local.partial_cmp(&other.local),
                Some(ns_ord) => match ns_ord {
                    Ordering::Equal => self.local.partial_cmp(&other.local),
                    _ => Some(ns_ord),
                },
            }
        }
    }

    impl Ord for QualifiedName {
        fn cmp(&self, other: &Self) -> Ordering {
            match self.partial_cmp(other) {
                None => Ordering::Equal,
                Some(ord) => ord,
            }
        }
    }

    impl<S> PartialEq<S> for QualifiedName
    where
        S: AsRef<str>,
    {
        fn eq(&self, other: &S) -> bool {
            let o = other.as_ref();
            let rhs = QualifiedName::from_str(o);
            if rhs.is_err() {
                return false;
            }
            let rhs = rhs.unwrap();
            self == &rhs
        }
    }

    impl QualifiedName {
        pub fn has_namespace(&self) -> bool {
            !self.ns.is_empty()
        }
    }

    /// XML nodes contain either XML elements or plain text.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Node {
        /// XML element
        Element(Element),
        /// XML text
        Text(String),
    }

    impl Node {
        /// Get a reference to Element from the Node if it exists.
        pub fn as_element(&self) -> Option<&Element> {
            match self {
                Node::Element(e) => Some(e),
                _ => None,
            }
        }

        /// Get a mutable reference to Element from the Node if it exists.
        pub fn as_element_mut(&mut self) -> Option<&mut Element> {
            match self {
                Node::Element(e) => Some(e),
                _ => None,
            }
        }

        /// Get a reference to the text stored in the Node.
        ///
        /// This only returns the text if the instance is Node::Text, not Node::Element.
        pub fn as_text(&self) -> Option<&str> {
            match self {
                Node::Text(s) => Some(s.as_str()),
                _ => None,
            }
        }

        /// Get a mutable reference to the text stored in the Node.
        ///
        /// This only returns the text if the instance is Node::Text, not Node::Element.
        pub fn as_text_mut(&mut self) -> Option<&mut String> {
            match self {
                Node::Text(s) => Some(s),
                _ => None,
            }
        }

        /// Turns the Node into an Element.
        pub fn into_element(self) -> Option<Element> {
            match self {
                Node::Element(e) => Some(e),
                _ => None,
            }
        }

        /// Turns the Node into a String.
        pub fn into_text(self) -> Option<String> {
            match self {
                Node::Text(s) => Some(s),
                _ => None,
            }
        }

        /// Check if the Node is an Element.
        pub fn is_element(&self) -> bool {
            match self {
                Node::Element(_) => true,
                Node::Text(_) => false,
            }
        }

        /// Check if the Node is Text.
        pub fn is_text(&self) -> bool {
            match self {
                Node::Element(_) => false,
                Node::Text(_) => true,
            }
        }
    }

    impl From<Element> for Node {
        fn from(e: Element) -> Self {
            Self::Element(e)
        }
    }

    /// Data model for an XML element.
    ///
    /// An XML element has a name, zero or more attributes and children.
    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct Element {
        /// Name of the XML element
        pub name: QualifiedName,
        /// Map of XML attributes with the name as a unique key with a matching the attribute value.
        pub attrs: BTreeMap<QualifiedName, String>,
        /// XML element children
        pub children: Vec<Node>,
    }

    impl TryFrom<Node> for Element {
        type Error = ElementError;

        fn try_from(value: Node) -> Result<Self, Self::Error> {
            match value {
                Node::Element(e) => Ok(e),
                Node::Text(_) => Err(Self::Error::FromNodeToElement),
            }
        }
    }

    impl FromStr for Element {
        type Err = ElementError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut rdr = quick_xml::Reader::from_str(s);
            match from_reader(&mut rdr) {
                Ok(elem) => Ok(elem),
                Err(e) => Err(e.into()),
            }
        }
    }

    impl Element {
        /// Get an XML attribute value by the attribute name.
        ///
        /// # Arguments
        /// * `name` - XML attribute name
        pub fn get_attr(&self, name: &QualifiedName) -> Option<&str> {
            for (k, v) in &self.attrs {
                if k.ns.as_str() == name.ns && k.local == name.local {
                    return Some(v.as_str());
                }
            }
            None
        }

        /// Find the first child element that matches with a given predicate.
        ///
        /// # Arguments
        /// * `predicate` - a function or lambda that takes an Element as an argument and returns a boolean
        ///
        /// # Example
        ///
        /// ```
        /// # use std::str::FromStr;
        /// # use dicom_std_xml_parser::dom::model::Element;
        ///
        /// let xml = r#"
        /// <?xml version="1.0" encoding="utf-8" standalone="no"?> <book
        /// xmlns="http://docbook.org/ns/docbook"
        /// xmlns:xl="http://www.w3.org/1999/xlink" label="PS3.3"
        /// version="5.0" xml:id="PS3.3">
        ///   <header>
        ///     <title>Title</title>
        ///     <subtitle>SubTitle1</subtitle>
        ///     <subtitle>SubTitle2</subtitle>
        ///   </header>
        /// </book>
        /// "#;
        ///
        /// let root = Element::from_str(xml).unwrap();
        /// let child = root.find_child(|e| e.name.local.as_str() == "subtitle").unwrap();
        /// assert_eq!(child.name.local.as_str(), "subtitle");
        /// let text = child.text().unwrap();
        /// assert_eq!(text, "SubTitle1");
        /// ```
        pub fn find_child<P>(&self, predicate: P) -> Option<&Element>
        where
            P: for<'r> Fn(&'r &Element) -> bool,
        {
            query::child(self, predicate)
        }

        /// Find all child elements that matches with a given predicate.
        ///
        /// # Arguments
        /// * `predicate` - a function or lambda that takes an Element as an argument and returns a boolean
        ///
        /// # Example
        ///
        /// ```
        /// # use std::str::FromStr;
        /// # use dicom_std_xml_parser::dom::model::Element;
        ///
        /// let xml = r#"
        /// <?xml version="1.0" encoding="utf-8" standalone="no"?> <book
        /// xmlns="http://docbook.org/ns/docbook"
        /// xmlns:xl="http://www.w3.org/1999/xlink" label="PS3.3"
        /// version="5.0" xml:id="PS3.3">
        ///   <header>
        ///     <title>Title</title>
        ///     <subtitle>SubTitle1</subtitle>
        ///     <subtitle>SubTitle2</subtitle>
        ///   </header>
        /// </book>
        /// "#;
        ///
        /// let root = Element::from_str(xml).unwrap();
        /// let children = root.find_children(|e| e.name.local.as_str() == "subtitle");
        /// # assert_eq!(children.len(), 2);
        /// let texts = ["SubTitle1", "SubTitle2"];
        /// for (e, s) in children.iter().zip(texts.iter()) {
        ///   assert_eq!(e.name.local.as_str(), "subtitle");
        ///   let t = e.text().unwrap();
        ///   assert_eq!(t.unwrap().as_str(), *s);
        /// }
        /// ```
        pub fn find_children<P>(&self, predicate: P) -> Vec<&Element>
        where
            P: for<'r> Fn(&'r &Element) -> bool,
        {
            query::children(self, predicate)
        }

        /// Get the text from the current element and it's subelements.
        pub fn text(&self) -> Option<String> {
            let mut s = "".to_string();
            let mut b = false;
            for child in &self.children {
                match child {
                    Node::Element(e) => {
                        if let Some(t) = e.text() {
                            b = true;
                            s.push_str(&t);
                        }
                    }
                    Node::Text(text) => {
                        b = true;
                        s.push_str(text);
                    }
                }
            }
            if b {
                Some(s)
            } else {
                None
            }
        }

        /// Get the trimmed text from the current element
        ///
        /// The text is trimmed at the beginning and end by removing whitespace
        /// and newline characters.
        pub fn text_trim(&self) -> Option<String> {
            let s = self.text()?;
            let mut t = "".to_string();
            for i in s.split(|c| c == '\n') {
                t.push_str(i.trim_start());
                t.push('\n');
            }
            let _ = t.remove(t.len() - 1);
            Some(t.trim_matches(is_char_whitespace_or_return).to_string())
        }
    }

    #[cfg(test)]
    mod tests {
        use std::collections::btree_map::BTreeMap;
        use std::str::FromStr;

        use crate::tests::{init_logger, test_resource_sample_dir};

        fn xml_sample_02() -> String {
            let file = test_resource_sample_dir().join("xml_sample_02.xml");
            let r = std::fs::read_to_string(file);
            assert!(r.is_ok(), "{}", r.unwrap_err());
            r.unwrap()
        }

        #[test]
        fn qualified_name_from_str() {
            assert!(super::QualifiedName::from_str("").is_err());
            let qn = super::QualifiedName::from_str("name");
            assert!(qn.is_ok());
            let qn = qn.unwrap();
            assert!(qn.ns.is_empty());
            assert_eq!(qn.local.as_str(), "name");
        }

        #[test]
        fn text_1() {
            init_logger();
            let xml = xml_sample_02();
            let root = super::Element::from_str(xml.as_str());
            assert!(root.is_ok(), "{}", root.err().unwrap());
            let root = root.unwrap();
            let author = root.find_child(|e| e.name.local.as_str() == "author");
            assert!(author.is_some());
            let author = author.unwrap();
            let text = author.text();
            assert!(text.is_some());
            let text = text.unwrap();
            let mut e_text = r#"
            DICOM Standards Committee
        "#;
            assert_eq!(text.as_str(), e_text);
            let copyright = root.find_child(|e| e.name.local.as_str() == "copyright");
            assert!(copyright.is_some());
            let copyright = copyright.unwrap();
            let text = copyright.text();
            assert!(text.is_some());
            let text = text.unwrap();
            e_text = r#"
            2021
            NEMA
        "#;
            assert_eq!(text.as_str(), e_text);
        }

        #[test]
        fn text_trim_1() {
            init_logger();
            let xml = xml_sample_02();
            let root = super::Element::from_str(xml.as_str());
            assert!(root.is_ok(), "{}", root.err().unwrap());
            let root = root.unwrap();
            let author = root.find_child(|e| e.name.local.as_str() == "author");
            assert!(author.is_some());
            let author = author.unwrap();
            let text = author.text_trim();
            assert!(text.is_some());
            let text = text.unwrap();
            let mut e_text = r#"DICOM Standards Committee"#;
            assert_eq!(text.as_str(), e_text);
            let copyright = root.find_child(|e| e.name.local.as_str() == "copyright");
            assert!(copyright.is_some());
            let copyright = copyright.unwrap();
            let text = copyright.text_trim();
            assert!(text.is_some());
            let text = text.unwrap();
            e_text = r#"2021
NEMA"#;
            assert_eq!(text.as_str(), e_text);
        }

        #[test]
        fn text_2() {
            let para = super::Element {
                name: super::QualifiedName {
                    ns: "".to_string(),
                    local: "para".to_string(),
                },
                attrs: BTreeMap::new(),
                children: vec![super::Node::Text("\nTest data\n".to_string())],
            };
            assert!(para.text().is_some());
            assert_eq!(para.text().unwrap(), "\nTest data\n");
        }

        #[test]
        fn text_trim_2() {
            let para = super::Element {
                name: super::QualifiedName {
                    ns: "".to_string(),
                    local: "para".to_string(),
                },
                attrs: BTreeMap::new(),
                children: vec![super::Node::Text("\nTest data\n".to_string())],
            };
            assert!(para.text_trim().is_some());
            assert_eq!(para.text_trim().unwrap(), "Test data");
        }
    }
}
