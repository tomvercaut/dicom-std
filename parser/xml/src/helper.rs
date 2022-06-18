use std::str::FromStr;

use log::debug;

use dicom_std_core::{Link, Tag, Usage};

use crate::dom::model::{Element, QualifiedName};
use crate::{query, ParserError, UNICODE_ZERO_WIDTH_SPACE};

/// Remove internal duplicates.
pub trait Dedup<T: PartialEq + Clone> {
    fn clear_duplicates(&mut self);
}

impl<T: PartialEq + Clone> Dedup<T> for Vec<T> {
    fn clear_duplicates(&mut self) {
        let mut already_seen = vec![];
        self.retain(|item| match already_seen.contains(item) {
            true => false,
            _ => {
                already_seen.push(item.clone());
                true
            }
        })
    }
}

/// Test if a character is a whitespace, a carriage return or a newline.
///
/// # Arguments
///
/// * `c` - character
#[inline]
pub(crate) fn is_char_whitespace_or_return(c: char) -> bool {
    c.is_whitespace() || c == '\r' || c == '\n'
}

/// Test if a character is zero width space.
///
/// # Arguments
/// * `c` - character
#[allow(dead_code)]
pub(crate) fn is_zero_width_space(c: char) -> bool {
    c == UNICODE_ZERO_WIDTH_SPACE
}

/// Get the trimmed text from a th element. If the th element has a nested para element,
/// than get the trimmed text from the para element.
///
/// # Arguments
///
/// * `e` - XML element (expecting a td element)
pub(crate) fn th_or_nested_para_text_trim(e: &Element) -> Option<String> {
    if e.name != QualifiedName::from_str("th").unwrap() {
        return None;
    }
    return match e.find_child(|t| t.name == QualifiedName::from_str("para").unwrap()) {
        None => e.text_trim(),
        Some(para) => para.text_trim(),
    };
}

/// Get the column and rowspan attribute from an XML element.
///
/// If an attribute is not found or the parsing of the value failed,
/// the respective value is set to 0.
///
/// # Arguments
///
/// * `e` - XML element
pub(crate) fn col_rowspan(e: &Element) -> (usize, usize) {
    let colspan = match e.get_attr(&QualifiedName::from_str("colspan").unwrap()) {
        None => 0,
        Some(s) => s.parse::<usize>().unwrap_or(0),
    };
    let rowspan = match e.get_attr(&QualifiedName::from_str("rowspan").unwrap()) {
        None => 0,
        Some(s) => s.parse::<usize>().unwrap_or(0),
    };
    (colspan, rowspan)
}

/// Get the trimmed text from a td element. If the td element has a nested para element,
/// than get the trimmed text from the para element.
///
/// # Arguments
///
/// * `e` - XML element (expecting a td element)
pub(crate) fn td_or_nested_para_text(e: &Element) -> Option<String> {
    if e.name != QualifiedName::from_str("td").unwrap() {
        return None;
    }
    return match e.find_child(|t| t.name == QualifiedName::from_str("para").unwrap()) {
        None => e.text_trim(),
        Some(para) => para.text_trim(),
    };
}

/// Get the trimmed text from a td element. If the td element has a nested para element,
/// the function checks if an emphasis element exists.
/// If it does, the trimmed text from the emphasis elemented is returned.
/// Otherwise the trimmed text from the para element is returned.
///
/// # Arguments
/// * `e` - XML element (expecting a td element)
pub(crate) fn td_or_nested_para_emphasis_text(e: &Element) -> Option<String> {
    if e.name != QualifiedName::from_str("td").unwrap() {
        return None;
    }
    return match e.find_child(|t| t.name == QualifiedName::from_str("para").unwrap()) {
        None => e.text_trim(),
        Some(para) => {
            match para.find_child(|t| t.name == QualifiedName::from_str("emphasis").unwrap()) {
                None => para.text_trim(),
                Some(emphasis) => emphasis.text_trim(),
            }
        }
    };
}

/// Get a nested emphasis element.
pub(crate) fn find_emphasis_element(e: &Element) -> Option<&Element> {
    let qn = QualifiedName::from_str("emphasis").unwrap();
    if e.name == qn{
        return Some(e);
    }
    e.find_child(|t|t.name == qn)
}

/// Filter all the "table" elements from a list of Elements.
///
/// # Arguments
/// * `ve` - input vector of Elements
#[allow(dead_code)]
pub(crate) fn filter_tables<'a>(ve: &Vec<&'a Element>) -> Vec<&'a Element> {
    let mut vt = vec![];
    for e in ve {
        let t = query::children_name(e, &QualifiedName::from_str("table").unwrap());
        vt.extend(t);
    }
    vt.clear_duplicates();
    vt
}

/// Test if the names in the table header, match with a predefined list of names.
///
/// # Arguments
/// * `e` - element with a thead element [e.g. table]
/// * `hdr_names` - slice with header names
pub(crate) fn has_table_column_header_names(e: &Element, hdr_names: &[&str]) -> bool {
    let vthead = e.find_children(|t| t.name == QualifiedName::from_str("thead").unwrap());
    let nthead = vthead.len();
    if nthead != 1 {
        debug!("XML table has zero or multiple thead elements [{}]", nthead);
        return false;
    }
    let thead = *vthead.get(0).unwrap();
    let vtr = thead.find_children(|t| t.name == QualifiedName::from_str("tr").unwrap());
    let ntr = vtr.len();
    if ntr != 1 {
        debug!("XML table has zero or multiple thead rows [{}]", ntr);
        return false;
    }
    let tr = *vtr.get(0).unwrap();
    let vth = tr.find_children(|t| t.name == QualifiedName::from_str("th").unwrap());
    let ntd = vth.len();
    if ntd != hdr_names.len() {
        debug!(
            "XML table doesn't have {} columns but {}",
            hdr_names.len(),
            ntd
        );
        return false;
    }
    // check title columns
    let vcs = vth
        .iter()
        .map(|t| th_or_nested_para_text_trim(*t))
        .collect::<Vec<Option<String>>>();
    for (a, b) in vcs.iter().zip(hdr_names.iter()) {
        match a {
            None => {
                if (*b).trim() != "" {
                    return false;
                }
            }
            Some(a) => {
                let la = a.to_lowercase();
                let lb = b.to_lowercase();
                if la.as_str() != lb.as_str() {
                    debug!("\na=\"{}\"\nb=\"{}\"", la.as_str(), lb.as_str());
                    return false;
                }
            }
        }
    }
    true
}

/// Parse an XML table column with a xref link.
///
/// # Arguments
/// * `e` - XML element (expecting a td element)
pub(crate) fn parse_td_reference(td: &Element) -> Result<Link, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    let vxref = td.find_children(|e| e.name == QualifiedName::from_str("xref").unwrap());
    let nvxref = vxref.len();
    if nvxref == 0 {
        return Err(ParserError::XmlElementNotFound("xref".to_string().parse()?));
    }
    if nvxref > 1 {
        return Err(ParserError::XmlElementMultipleFound(
            "xref".to_string(),
            nvxref,
        ));
    }
    let xref = *vxref.get(0).unwrap();
    let err = ParserError::XmlTableColumnParse(
        "failed to parse xref element".to_string(),
    );
    parse_xref(xref).ok_or(err)
}

/// Parse an XML table column with a Usage description (M, U, C).
///
/// # Arguments
/// * `e` - XML element (expecting a td element)
pub(crate) fn parse_td_usage(td: &Element) -> Result<Usage, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    let opt = td.find_child(|t| t.name == QualifiedName::from_str("para").unwrap());
    if opt.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected para element found".to_string(),
        ));
    }
    let para = opt.unwrap();
    let s = para.text_trim();
    let usage = if let Some(s) = s {
        Usage::from(s.as_str())
    } else {
        Usage::None
    };
    if usage == Usage::None {
        return Err(ParserError::XmlTableColumnParse(
            "failed to parse Usage from para element".to_string(),
        ));
    }
    Ok(usage)
}

/// Parse an XML table column with a Attribute name.
///
/// # Arguments
///
/// * `e` -XML element (expecting a td element)
pub(crate) fn parse_td_attribute_name(td: &Element) -> Result<String, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    let opt = td.find_child(|t| t.name == "para");
    if opt.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected para element found".to_string(),
        ));
    }
    let para = opt.unwrap();
    let err = ParserError::XmlTableColumnParse(
        "failed to parse Attribute Name from para element".to_string(),
    );
    para.text_trim().ok_or(err)
}

/// Parse an XML td element and extract a DICOM tag.
///
/// # Arguments
///
/// * `td` - XML element (expecting a td element)
pub(crate) fn parse_td_tag(td: &Element) -> Result<Tag, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    let opt = td.find_child(|t| t.name == "para");
    if opt.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected para element found".to_string(),
        ));
    }
    let para = opt.unwrap();
    let err = ParserError::XmlTableColumnParse(
        "failed to parse Attribute Name from para element".to_string(),
    );
    let txt = para.text_trim().ok_or(err)?;
    return match Tag::from_str(txt.as_str()) {
        Ok(tag) => Ok(tag),
        Err(e) => Err(e.into()),
    };
}

/// Parse an xref element into a link.
///
/// # Arguments
///
/// * `xref` - XML element (expecting an xref element)
fn parse_xref(xref: &Element) -> Option<Link> {
    if xref.name != QualifiedName::from_str("xref").unwrap() {
        return None;
    }
    Some(Link {
        target: xref
            .get_attr(&QualifiedName::from_str("linkend").unwrap())
            .unwrap_or_default()
            .to_string(),
        style: xref
            .get_attr(&QualifiedName::from_str("xrefstyle").unwrap())
            .unwrap_or_default()
            .to_string()
    })
}

/// Parse a td XML element with a nested xref XML element and extract the include text.
///
/// # Arguments
///
/// * `td` - XML element (expecting a td element)
pub(crate) fn parse_td_include_text(td: &Element) -> Result<String, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    let opt = td.find_child(|t| t.name == "para");
    if opt.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected para element found".to_string(),
        ));
    }
    let para = opt.unwrap();
    let opt = para.find_child(|t| t.name == QualifiedName::from_str("emphasis").unwrap());
    if opt.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected emphasis element found".to_string(),
        ));
    }
    let emphasis = opt.unwrap();
    let txt = emphasis.text_trim().unwrap_or_default();
    let pos = txt.find("Include");
    if pos.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "invalid format of an include text in module attribute table column".to_string(),
        ));
    }
    let pos = 7 + pos.unwrap();
    let sub = &txt[0..pos];
    Ok(sub.to_string())
}

/// Parse a td XML element with a nested xref XML element and extract a Link.
///
/// # Arguments
/// * `td` - XML element (expecting a td element)
pub(crate) fn parse_td_include_link(td: &Element) -> Result<Link, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    let opt = td.find_child(|t| t.name == QualifiedName::from_str("para").unwrap());
    if opt.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected para element found".to_string(),
        ));
    }
    let para = opt.unwrap();
    let xref = para.find_child(|t| t.name == QualifiedName::from_str("xref").unwrap());
    if xref.is_none() {
        return Err(ParserError::XmlTableColumnParse(
            "expected xref element found".to_string(),
        ));
    }
    let xref = xref.unwrap();
    let err = ParserError::XmlTableColumnParse(
        "failed to parse Link from xref element".to_string(),
    );
    parse_xref(xref).ok_or(err)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use dicom_std_core::Link;

    use crate::dom::model::Element;
    use crate::tests::init_logger;

    #[test]
    fn parse_td_include_text_link() {
        init_logger();
        let xml = r#"
        <td align="left" colspan="3" rowspan="1">
            <para xml:id="para_470d7553-b1a8-4c41-bad7-fa0913c930c1">
                <emphasis role="italic">
                    &gt;Include <xref linkend="table_10-3b" xrefstyle="select: label quotedtitle"/>
                </emphasis>
            </para>
        </td>
        "#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let include_text = super::parse_td_include_text(&root);
        assert!(include_text.is_ok(), "{}", include_text.err().unwrap());
        let include_text = include_text.unwrap();
        assert_eq!(include_text.as_str(), ">Include");
        let link = super::parse_td_include_link(&root);
        assert!(link.is_ok(), "{}", link.err().unwrap());
        let link = link.unwrap();
        let e_link = Link {
            target: "table_10-3b".to_string(),
            style: "select: label quotedtitle".to_string(),
        };
        assert_eq!(link, e_link);
    }
}
