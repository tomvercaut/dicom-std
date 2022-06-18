use crate::dom::model::{Element, QualifiedName};
use crate::{helper, td_or_nested_para_emphasis_text, ParserError, UNICODE_ZERO_WIDTH_SPACE, find_emphasis_element, is_char_whitespace_or_return};
use dicom_std_core::{DataDictionary, DataDictionaryEntry, TagRange, VM, VR};
use log::{debug, info};
use std::str::FromStr;

/// Build a data dictionary from the table in [part 6, chapter 6](https://dicom.nema.org/medical/dicom/current/output/html/part06.html#chapter_6) of the DICOM standard.
///
/// # Arguments
/// * `element` - (root) XML element of part 6 of the DICOM standard
pub fn build(element: &Element) -> Result<DataDictionary, ParserError> {
    info!("Building data element dictionary.");
    let ids = ["chapter_6"];
    let qn_id = QualifiedName::from_str("xml:id").unwrap();
    let mut chapters = element.find_children(|t| {
        let attr = t.get_attr(&qn_id);
        match attr {
            None => {}
            Some(value) => {
                for id in ids {
                    if id == value {
                        return true;
                    }
                }
            }
        }
        false
    });
    if chapters.is_empty() {
        if let Some(value) = element.get_attr(&qn_id) {
            for id in ids {
                if id == value {
                    chapters.push(element);
                }
            }
        }
    }
    let mut vtables = vec![];
    let qn_table = QualifiedName::from_str("table").unwrap();
    for chapter in chapters {
        let tv = chapter.find_children(|t| t.name == qn_table);
        vtables.extend(tv);
    }
    let mut dict = DataDictionary::default();
    for table in vtables {
        let mut reg = parse_registery_data_elements_table(table)?;
        dict.extend(&mut reg);
    }
    Ok(dict)
}

/// Parse data dictionary of DICOM data elements.
///
/// See chapter 6 in
/// [part 06](https://dicom.nema.org/medical/dicom/current/output/html/part06.html#chapter_6)
/// of the DICOM standard for more information.
///
/// # Arguments
/// * `e` - XML table element
fn parse_registery_data_elements_table(
    e: &Element,
) -> Result<DataDictionary, ParserError> {
    if e.name != "table" {
        return Err(ParserError::XmlExpectedElement(
            QualifiedName::from_str("table").unwrap(),
            e.name.clone(),
        ));
    }
    debug!(
        "Parsing data elements dictionary table: {}",
        e.get_attr(&QualifiedName::from_str("xml:id").unwrap())
            .unwrap()
    );
    let mut reg = DataDictionary::default();
    // validate table format
    if !has_data_element_dictionary_table_header(e) {
        return Err(ParserError::NotDataElementRegistery);
    }
    // parse table rows
    match e.find_child(|t| t.name == QualifiedName::from_str("tbody").unwrap()) {
        None => {
            return Err(ParserError::XmlTableHasNoTbody);
        }
        Some(tbody) => {
            let vtr = tbody.find_children(|t| t.name == QualifiedName::from_str("tr").unwrap());
            for tr in vtr {
                let item = parse_data_element_dictionary_table_row(tr)?;
                reg.add(item);
            }
        }
    }
    Ok(reg)
}

/// Check if an XML table header (thead element) matches with the one from a Registry of DICOM Data Elements.
///
/// To be valid the table header needs to have:
/// * one thead element
/// * one tr element in thead
/// * 6 columns with the respective names:
///     * Tag
///     * Name
///     * Keyword
///     * VR
///     * VM
///     * <empty, no name>
///
/// # Arguments
/// * `e` - table element
fn has_data_element_dictionary_table_header(e: &Element) -> bool {
    let hdr_names = ["Tag", "Name", "Keyword", "VR", "VM", ""];
    if helper::has_table_column_header_names(e, &hdr_names) {
        return true;
    }
    false
}

fn parse_data_element_dictionary_table_row(
    tr: &Element,
) -> Result<DataDictionaryEntry, ParserError> {
    let vtd = tr.find_children(|t| t.name == QualifiedName::from_str("td").unwrap());
    let nvtd = vtd.len();
    let mut entry = DataDictionaryEntry::default();
    if nvtd == 6 {
        entry.tag = parse_td_tag(vtd.get(0).unwrap())?;
        debug!("Parsing entry: {}", &entry.tag);
        let name = parse_td_name(vtd.get(1).unwrap());
        if let Ok(name) = name {
            entry.name = name;
        } else {
            entry.name = "".to_string();
        }
        if !entry.name.is_empty() {
            // See [note](https://dicom.nema.org/medical/dicom/current/output/html/part06.html#note_6_3)
            entry.keyword = parse_td_keyword(vtd.get(2).unwrap())?;
            entry.vr = parse_td_vr(vtd.get(3).unwrap())?;
            entry.vm = parse_td_vm(vtd.get(4).unwrap())?;
        } else {
            if let Ok(keyword) = parse_td_keyword(vtd.get(2).unwrap()) {
                entry.keyword = keyword;
            }
            if let Ok(vr) = parse_td_vr(vtd.get(3).unwrap()) {
                entry.vr = vr;
            }
            if let Ok(vm) = parse_td_vm(vtd.get(4).unwrap()) {
                entry.vm = vm;
            }
        }
        let desc_col = vtd.get(5).unwrap();
        // In [chapter5, Conventions)[https://dicom.nema.org/medical/dicom/current/output/html/part06.html#chapter_5], retired elements are italicized and
        // the description starts with RET.
        let is_italic = match find_emphasis_element(desc_col) {
            None => {false}
            Some(emphasis) => {
                match emphasis.get_attr(&QualifiedName::from_str("role").unwrap()) {
                    None => {false}
                    Some(val) => {val.contains("italic")}
                }
            }
        };
        entry.description = parse_td_description(vtd.get(5).unwrap());
        entry.retired = is_italic || entry.description
            .trim_matches(is_char_whitespace_or_return)
            .starts_with("RET");
        // entry.retired = entry.description.starts_with("RET");
    } else {
        return Err(ParserError::XmlTableInvalidNumberOfColumns(
            "6".to_string(),
            nvtd,
        ));
    }
    Ok(entry)
}

fn parse_td_tag(td: &Element) -> Result<TagRange, ParserError> {
    let e = ParserError::XmlTableColumnParse(
        "unable to extract tag".to_string(),
    );
    let s = td_or_nested_para_emphasis_text(td)
        .ok_or(e)?
        .replace(UNICODE_ZERO_WIDTH_SPACE, "");
    return match TagRange::from_str(s.as_str()) {
        Ok(tag) => Ok(tag),
        Err(e) => Err(ParserError::TagRangeError { source: e }),
    };
}

fn parse_td_name(td: &Element) -> Result<String, ParserError> {
    let e = ParserError::XmlTableColumnParse(
        "unable to extract registery data entry name".to_string(),
    );
    td_or_nested_para_emphasis_text(td)
        .ok_or(e)
        .map(|t| t.replace(UNICODE_ZERO_WIDTH_SPACE, ""))
}

fn parse_td_keyword(td: &Element) -> Result<String, ParserError> {
    let e = ParserError::XmlTableColumnParse(
        "unable to extract registry data element entry keyword".to_string(),
    );
    td_or_nested_para_emphasis_text(td)
        .ok_or(e)
        .map(|t| t.replace(UNICODE_ZERO_WIDTH_SPACE, ""))
}

fn parse_td_vr(td: &Element) -> Result<Vec<VR>, ParserError> {
    let e = ParserError::XmlTableColumnParse(
        "unable to extract data element entry VR".to_string(),
    );
    let s = td_or_nested_para_emphasis_text(td)
        .ok_or(e)
        .map(|t| t.replace(UNICODE_ZERO_WIDTH_SPACE, ""))?;
    if s.starts_with("See Note") {
        return Ok(vec![]);
    }
    return if s.contains(" or ") {
        let vvr = s.split(" or ").collect::<Vec<&str>>();
        let mut v = vec![];
        for svr in vvr {
            match VR::from_str(svr) {
                Ok(vr) => {
                    v.push(vr);
                }
                Err(e) => {
                    return Err(ParserError::VRError { source: e });
                }
            }
        }
        Ok(v)
    } else {
        match VR::from_str(s.as_str()) {
            Ok(vr) => Ok(vec![vr]),
            Err(e) => return Err(ParserError::VRError { source: e }),
        }
    };
}

fn parse_td_vm(td: &Element) -> Result<VM, ParserError> {
    let e=ParserError::XmlTableColumnParse(
        "unable to extract data element entry VM".to_string(),
    );
    let s = td_or_nested_para_emphasis_text(td)
        .ok_or(e)
        .map(|t| t.replace(UNICODE_ZERO_WIDTH_SPACE, ""))?;
    return match VM::from_str(s.as_str()) {
        Ok(tag) => Ok(tag),
        Err(e) => Err(ParserError::VMError { source: e }),
    };
}

fn parse_td_description(td: &Element) -> String {
    td_or_nested_para_emphasis_text(td)
        .unwrap_or_default()
        .replace(UNICODE_ZERO_WIDTH_SPACE, "")
    // td_or_nested_para_text(td).ok_or(ParserError::XmlTableColumnParse(
    //     "unable to extract registry data element entry description".to_string(),
    // ))
}

#[cfg(test)]
mod tests {
    use crate::dom;
    use crate::tests::{init_logger, path_dicom_part};
    use dicom_std_core::{Tag, TagRange, VR};
    use std::str::FromStr;

    #[test]
    fn data_dict_build() {
        init_logger();
        let part06 = path_dicom_part(6);
        let root = dom::read_file(part06).unwrap();
        let dd = super::build(&root);
        assert!(
            dd.is_ok(),
            "Failed to read data dictionary: {}",
            dd.err().unwrap()
        );
        let dd = dd.unwrap();
        let entry = dd.by_tag(Tag {
            g: 0x300A,
            e: 0x011C,
        });
        assert!(entry.is_some(), "Failed to lookup entry by tag.");
        let entry = dd.by_tag(Tag::from_str("6000,1201").unwrap());
        assert!(entry.is_some(), "Failed to lookup entry by tag.");
        let entry = entry.unwrap();
        assert_eq!(entry.tag, TagRange::from_str("60xx,1201").unwrap());
        assert_eq!(entry.name.as_str(), "Overlays - Red");
        assert_eq!(entry.keyword.as_str(), "OverlaysRed");
        assert_eq!(entry.vr.len(), 1);
        assert_eq!(*entry.vr.get(0).unwrap(), VR::US);
        assert_eq!(entry.vm.min, 1);
        assert_eq!(entry.vm.max, 0);
        assert!(!entry.vm.is_min_n);
        assert!(entry.vm.is_max_n);

        let entry = dd.by_tag(Tag::from_str("300A,011C").unwrap());
        assert!(entry.is_some(), "Failed to lookup entry by tag.");
        let entry = entry.unwrap();
        assert_eq!(entry.tag, TagRange::from_str("300A,011C").unwrap());
        assert_eq!(entry.name.as_str(), "Leaf/Jaw Positions");
        assert_eq!(entry.keyword.as_str(), "LeafJawPositions");
        assert_eq!(entry.vr.len(), 1);
        assert_eq!(*entry.vr.get(0).unwrap(), VR::DS);
        assert_eq!(entry.vm.min, 2);
        assert_eq!(entry.vm.max, 2);
        assert!(!entry.vm.is_min_n);
        assert!(entry.vm.is_max_n);
    }
}
