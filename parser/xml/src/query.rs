use std::str::FromStr;

use dicom_std_core::Link;

use crate::dom::model::{Element, Node, QualifiedName};
use crate::QueryError;

fn attr_filter_detail<'a, 'b>(
    ve: &mut Vec<&'a Element>,
    element: &'a Element,
    attr_name: &'b QualifiedName,
    attr_value: &'b str,
    f: fn(&QualifiedName, &str, &QualifiedName, &str) -> bool,
) {
    for (k, v) in &element.attrs {
        if f(k, v.as_str(), attr_name, attr_value) {
            ve.push(element);
        }
    }
    for child in &element.children {
        match child {
            Node::Element(e) => {
                attr_filter_detail(ve, e, attr_name, attr_value, f);
            }
            Node::Text(_) => {}
        }
    }
}

pub fn attr_filter<'a, 'b>(
    element: &'a Element,
    attr_name: &'b QualifiedName,
    attr_value: &'b str,
    f: fn(&QualifiedName, &str, &QualifiedName, &str) -> bool,
) -> Vec<&'a Element> {
    let mut ve = vec![];
    attr_filter_detail(&mut ve, element, attr_name, attr_value, f);
    ve
}

fn starts_with(name1: &QualifiedName, value1: &str, name2: &QualifiedName, value2: &str) -> bool {
    name1 == name2 && value1.starts_with(value2)
}

fn equals_to(name1: &QualifiedName, value1: &str, name2: &QualifiedName, value2: &str) -> bool {
    name1 == name2 && value1 == value2
}

pub fn ids_start_with<'a, 'b>(element: &'a Element, value: &'b str) -> Vec<&'a Element> {
    let qn_id = QualifiedName {
        ns: "xml".to_string(),
        local: "id".to_string(),
    };
    attr_filter(element, &qn_id, value, starts_with)
}

/// Recursive query to find an element by its xml:id attribute.
///
/// # Arguments
///
/// * `element` - XML element
/// * `value` - value of the XML attribute id
pub fn id<'a, 'b>(element: &'a Element, value: &'b str) -> Option<&'a Element> {
    let qn_id = QualifiedName {
        ns: "xml".to_string(),
        local: "id".to_string(),
    };
    let mut v = attr_filter(element, &qn_id, value, equals_to);
    if v.len() == 1 {
        v.pop()
    } else {
        None
    }
}

/// Find sub elements in `element` with a matching element name.
///
/// # Arguments
/// * `ve` - vector of Elements in which matches are stored
/// * `element` - XML element
/// * `name` - name of the element being queried
/// * `limit` - limit the number of result to be found (0: unlimited)
fn names_detail<'a>(
    ve: &mut Vec<&'a Element>,
    element: &'a Element,
    name: &QualifiedName,
    limit: usize,
) {
    if &element.name == name {
        ve.push(element);
    }
    for child in &element.children {
        if limit != 0 && ve.len() >= limit {
            break;
        }
        if let Node::Element(e) = child {
            names_detail(ve, e, name, limit);
        }
    }
}

/// Find all the children in `element` with a matching element `name`.
///
/// # Arguments
/// * `element` - XML element
/// * `name` - name of the element being queried
pub(crate) fn children_name<'a>(element: &'a Element, name: &QualifiedName) -> Vec<&'a Element> {
    let mut ve = vec![];
    for child in &element.children {
        if let Node::Element(e) = child {
            names_detail(&mut ve, e, name, 0);
        }
    }
    ve
}

/// Find a nested child element by predicate.
///
/// This function takes the predicate by reference so it can be called recursively.
/// # Arguments
/// * `element` - XML element
/// * `predicate` - function or lambda that takes an element as an argument and returns a boolean
fn child_detail<'a, 'b, P>(element: &'a Element, predicate: &'b P) -> Option<&'a Element>
where
    P: for<'r> Fn(&'r &Element) -> bool,
{
    if predicate(&element) {
        return Some(element);
    }
    for child in &element.children {
        match child {
            Node::Element(e) => {
                let r = child_detail(e, predicate);
                if r.is_some() {
                    return r;
                }
            }
            Node::Text(_) => {}
        }
    }
    None
}

/// Find a nested child element by predicate.
///
/// # Arguments
/// * `element` - XML element
/// * `predicate` - function or lambda that takes an element as an argument and returns a boolean
pub(crate) fn child<P>(element: &Element, predicate: P) -> Option<&Element>
where
    P: for<'r> Fn(&'r &Element) -> bool,
{
    for child in &element.children {
        match child {
            Node::Element(e) => {
                let r = child_detail(e, &predicate);
                if r.is_some() {
                    return r;
                }
            }
            Node::Text(_) => {}
        }
    }
    None
}

/// Find all nested children elements that match the predicate.
///
/// This function takes the predicate by reference so it can be called recursively.
/// # Arguments
/// * `element` - XML element
/// * `predicate` - function or lambda that takes an element as an argument and returns a boolean
fn children_detail<'a, 'b, 'c, P>(
    v: &'b mut Vec<&'a Element>,
    element: &'a Element,
    predicate: &'c P,
) where
    P: for<'r> Fn(&'r &Element) -> bool,
{
    if predicate(&element) {
        v.push(element);
    }
    for node in &element.children {
        match node {
            Node::Element(e) => {
                children_detail(v, e, predicate);
            }
            Node::Text(_) => {}
        }
    }
}

/// Find all nested children elements that match the predicate.
///
/// # Arguments
/// * `element` - XML element
/// * `predicate` - function or lambda that takes an element as an argument and returns a boolean
pub(crate) fn children<'a, 'b, P>(element: &'a Element, predicate: P) -> Vec<&'a Element>
where
    P: for<'r> Fn(&'r &Element) -> bool,
{
    let mut v = vec![];
    for child in &element.children {
        match child {
            Node::Element(e) => {
                children_detail(&mut v, e, &predicate);
            }
            Node::Text(_) => {}
        }
    }
    v
}

/// Resolve a link to an Element.
///
/// If an element is found by the target in the link:
/// * if the name of the Element is "table", it is returned as a vector
/// * if the name of the Element is something else, find all nested tables within that Element
/// * if no matching element is found, an Error is returned
///
/// # Arguments
/// * `e` - XML element
/// * `link` - a Link pointing to a xml:id
pub(crate) fn link_to_table_or_nested_tables<'a, 'b>(
    e: &'a Element,
    link: &'b Link,
) -> Result<Vec<&'a Element>, QueryError> {
    let qn_table = QualifiedName::from_str("table")?;
    let qn_id = QualifiedName::from_str("xml:id")?;
    let mut v = vec![];
    if e.name == qn_table {
        if let Some(attr) = e.get_attr(&qn_id) {
            if attr == link.target.as_str() {
                return Ok(vec![e]);
            }
        }
    }
    let o = e.find_child(|t| -> bool {
        if let Some(attr) = t.get_attr(&qn_id) {
            if attr == link.target.as_str() {
                return true;
            }
        }
        false
    });
    match o {
        None => {
            return Err(QueryError::LinkNotFound(format!(
                "unable to find include link to xml:id {}",
                &link
            )));
        }
        Some(e) => {
            if e.name == qn_table {
                v.push(e);
            } else {
                v = e.find_children(|t| t.name == qn_table);
            }
        }
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::dom::model::{Element, QualifiedName};
    use crate::tests::{init_logger, test_resource_sample_dir};

    fn xml_sample_01() -> String {
        let file = test_resource_sample_dir().join("xml_sample_01.xml");
        let r = std::fs::read_to_string(file);
        assert!(r.is_ok(), "{}", r.unwrap_err());
        r.unwrap()
    }

    #[test]
    fn query_ids_with() {
        let xml = xml_sample_01();
        let root = Element::from_str(xml.as_str());
        assert!(root.is_ok(), "{}", root.err().unwrap());
        let root = root.unwrap();
        let v = super::ids_start_with(&root, "para");
        let n = v.len();
        assert_eq!(n, 6);
        let vids = vec![
            "para_80687a13-9d6e-433a-b7a4-36746d82979e",
            "para_ca3b4c38-80a2-41ab-8378-33410ef2664a",
            "para_acac4b3b-0236-405c-9a2e-5de428a86acc",
            "para_8370e127-477e-40a5-92c9-1406636d5d41",
            "para_73660222-86aa-4ce9-9cd8-56c8aa07edff",
            "para_1004d97c-6409-463f-bfcf-3a4e1be9cc19",
        ];
        for (e, id) in v.iter().zip(vids.iter()) {
            let attr = e
                .get_attr(&QualifiedName::from_str("xml:id").unwrap())
                .unwrap();
            assert_eq!(attr, *id);
        }
    }

    #[test]
    fn query_id() {
        let xml = xml_sample_01();
        let root = Element::from_str(xml.as_str());
        assert!(root.is_ok(), "{}", root.err().unwrap());
        let root = root.unwrap();
        let elem = super::id(&root, "para_acac4b3b-0236-405c-9a2e-5de428a86acc");
        assert!(elem.is_some(), "unable to find element by id");
        let elem = elem.unwrap();
        assert_eq!(elem.children.len(), 1);
        let text = elem.children.get(0).unwrap();
        assert!(text.is_text());
        let text = text.as_text().unwrap();
        assert!(text.starts_with("NEMA standards and guideline"));
    }

    #[test]
    fn query_children_name() {
        let xml = xml_sample_01();
        let root = Element::from_str(xml.as_str());
        assert!(root.is_ok(), "{}", root.err().unwrap());
        let root = root.unwrap();
        let v = super::children_name(&root, &QualifiedName::from_str("para").unwrap());
        let n = v.len();
        assert_eq!(n, 6);
        let vids = vec![
            "para_80687a13-9d6e-433a-b7a4-36746d82979e",
            "para_ca3b4c38-80a2-41ab-8378-33410ef2664a",
            "para_acac4b3b-0236-405c-9a2e-5de428a86acc",
            "para_8370e127-477e-40a5-92c9-1406636d5d41",
            "para_73660222-86aa-4ce9-9cd8-56c8aa07edff",
            "para_1004d97c-6409-463f-bfcf-3a4e1be9cc19",
        ];
        assert_eq!(n, vids.len());
        for (e, id) in v.iter().zip(vids.iter()) {
            let attr_value = (*e).get_attr(&QualifiedName::from_str("xml:id").unwrap());
            assert!(attr_value.is_some());
            let attr_value = attr_value.unwrap();
            assert_eq!(attr_value, *id);
        }
    }

    #[test]
    fn query_child() {
        let xml = xml_sample_01();
        let root = Element::from_str(xml.as_str());
        assert!(root.is_ok(), "{}", root.err().unwrap());
        let root = root.unwrap();
        let find = super::child(&root, |e| {
            if let Some(attr) = e.get_attr(&QualifiedName::from_str("xml:id").unwrap()) {
                if attr.starts_with("para_") {
                    return true;
                }
            }
            false
        });
        assert!(
            find.is_some(),
            "unable to find child with an attribute starting with para_"
        );
        let find = find.unwrap();
        let attr = find.get_attr(&QualifiedName::from_str("xml:id").unwrap());
        assert!(
            attr.is_some(),
            "attribute xml:id was not found in para element"
        );
        let attr = attr.unwrap();
        assert_eq!(attr, "para_80687a13-9d6e-433a-b7a4-36746d82979e");
    }

    #[test]
    fn query_children() {
        init_logger();
        let xml = xml_sample_01();
        let root = Element::from_str(xml.as_str());
        assert!(root.is_ok(), "{}", root.err().unwrap());
        let root = root.unwrap();
        let v = super::children(&root, |e| {
            e.name == QualifiedName::from_str("para").unwrap()
        });
        let n = v.len();
        assert_eq!(n, 6);
        let vids = vec![
            "para_80687a13-9d6e-433a-b7a4-36746d82979e",
            "para_ca3b4c38-80a2-41ab-8378-33410ef2664a",
            "para_acac4b3b-0236-405c-9a2e-5de428a86acc",
            "para_8370e127-477e-40a5-92c9-1406636d5d41",
            "para_73660222-86aa-4ce9-9cd8-56c8aa07edff",
            "para_1004d97c-6409-463f-bfcf-3a4e1be9cc19",
        ];
        assert_eq!(n, vids.len());
        for (e, id) in v.iter().zip(vids.iter()) {
            let attr_value = (*e).get_attr(&QualifiedName::from_str("xml:id").unwrap());
            assert!(attr_value.is_some());
            let attr_value = attr_value.unwrap();
            assert_eq!(attr_value, *id);
        }
    }
}
