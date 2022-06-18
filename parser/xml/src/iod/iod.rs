use std::str::FromStr;

use log::debug;

use dicom_std_core::{
    CompositeIodModule, CompositeIodModuleItem, CompositeModuleReferenceUsage, IodModuleType,
    ModuleAttribute, ModuleAttributeInclude, ModuleAttributeReferenceTopLevelAttributes,
    ModuleDefinition, ModuleDefinitionItem,
};

use crate::dom::model::{Element, QualifiedName};
use crate::{col_rowspan, helper, query, td_or_nested_para_text, ParserError};

pub(crate) fn filter_composite_iod_module_sections(element: &Element) -> Vec<&Element> {
    let v = query::ids_start_with(element, "sect_A.");
    let mut r = vec![];
    let attr_id_key = QualifiedName::from_str("xml:id").unwrap();
    for i in &v {
        let elem = *i;
        for (k, v) in &elem.attrs {
            if &attr_id_key == k && !v.starts_with("sect_A.1") {
                r.push(*i);
            }
        }
    }
    r
}

/// Parse a composite Information Object Definition (IOD) module table.
///
/// See chapter A in
/// [part 03](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#chapter_A)
/// of the DICOM standard for more information.
///
/// # Arguments
///
/// * `e` - XML table element
fn parse_composite_iod_module_table(e: &Element) -> Result<CompositeIodModule, ParserError> {
    if e.name != "table" {
        return Err(ParserError::XmlExpectedElement(
            QualifiedName::from_str("table").unwrap(),
            e.name.clone(),
        ));
    }
    let mut iod_module = CompositeIodModule::default();
    let caption = e.find_child(|t| t.name == QualifiedName::from_str("caption").unwrap());
    // , |t|&QualifiedName::from_str("caption").unwrap()).ok_or(XmlElementNotFound(QualifiedName::from_str("caption").unwrap()))?;
    // iod_module.caption = caption.text.as_ref().unwrap_or(&"".to_string()).to_string();
    iod_module.caption = caption.unwrap().text().unwrap_or_default();
    // validate table format
    if !has_composite_iod_module_table_header(e) {
        return Err(ParserError::NotIodModuleTable);
    }
    // parse table rows
    match e.find_child(|t| t.name == QualifiedName::from_str("tbody").unwrap()) {
        None => {
            return Err(ParserError::XmlTableHasNoTbody);
        }
        Some(tbody) => {
            let vtr = tbody.find_children(|t| t.name == QualifiedName::from_str("tr").unwrap());
            for tr in vtr {
                let item = parse_composite_iod_module_table_row(tr)?;
                if item.ie.is_empty() {
                    let n = iod_module.items.len();
                    if n == 0 {
                        return Err(ParserError::NoIodModuleToAppendItem);
                    }
                    let iod_item = iod_module.items.get_mut(n - 1).unwrap();
                    iod_item.items.extend(item.items);
                } else {
                    iod_module.items.push(item);
                }
            }
        }
    }
    Ok(iod_module)
}

/// Parse information object definition table.
///
/// See chapter C in [part 03](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#chapter_C)
///
/// # Arguments
///
/// * `e` - XML table element
fn parse_iod_module_table(e: &Element) -> Result<ModuleDefinition, ParserError> {
    if e.name != "table" {
        return Err(ParserError::XmlExpectedElement(
            QualifiedName::from_str("table").unwrap(),
            e.name.clone(),
        ));
    }
    let mut iod_definition = ModuleDefinition::default();
    let caption = e.find_child(|t| t.name == QualifiedName::from_str("caption").unwrap());
    iod_definition.id = e
        .get_attr(&QualifiedName::from_str("xml:id").unwrap())
        .unwrap_or_default()
        .to_string();
    iod_definition.caption = caption.unwrap().text().unwrap_or_default();
    // validate table format
    debug!("parsing table: {}", iod_definition.id.as_str());
    if !has_iod_module_table_header(e) {
        return Err(ParserError::NotIodModuleTable);
    }

    // parse table rows
    match e.find_child(|t| t.name == QualifiedName::from_str("tbody").unwrap()) {
        None => {
            return Err(ParserError::XmlTableHasNoTbody);
        }
        Some(tbody) => {
            let vtr = tbody.find_children(|t| t.name == QualifiedName::from_str("tr").unwrap());
            for tr in vtr {
                if let Some(item) = parse_iod_module_table_row(tr)? {
                    iod_definition.items.push(item);
                }
            }
        }
    }
    Ok(iod_definition)
}

/// Check if an XML table header (thead element) matches with the one from a composite IOD module table.
///
/// To be valid the table header needs to have:
/// - one <thead> element
/// - one <tr> element in thead
/// - 4 columns with the respective names:
///   * IE
///   * Module
///   * Reference
///   * Usage
///
/// # Arguments
/// * `e` - table element
fn has_composite_iod_module_table_header(e: &Element) -> bool {
    let hdr_names = ["IE", "Module", "Reference", "Usage"];
    helper::has_table_column_header_names(e, &hdr_names)
}

/// Check if an XML table header (thead element) matches with the one from a IOD table.
///
/// To be valid the table header needs to have:
/// - one thead element
/// - one tr element in thead
/// - 4 columns with the respective names:
///   * Attribute Name
///   * Tag
///   * Type
///   * Attribute Description
///
/// # Arguments
/// * `e` - table element
fn has_iod_module_table_header(e: &Element) -> bool {
    let hdr_names = [
        ["Attribute Name", "Tag", "Type", "Attribute Description"],
        ["Attribute Name", "Tag", "Type", "Description"],
    ];
    for hdr in hdr_names {
        if helper::has_table_column_header_names(e, &hdr) {
            return true;
        }
    }
    false
}

/// Parse a row in a composite IOD module table.
///
/// # Arguments
///
/// * `tr` - XML element (expecting a tr element)
fn parse_composite_iod_module_table_row(
    tr: &Element,
) -> Result<CompositeIodModuleItem, ParserError> {
    let vtd = tr.find_children(|t| t.name == QualifiedName::from_str("td").unwrap());
    let ntd = vtd.len();

    let mut iod_module_item = CompositeIodModuleItem::default();
    if ntd == 4 {
        let td_ie = *vtd.get(0).unwrap();
        let (_colspan, rowspan) = col_rowspan(td_ie);
        iod_module_item.expected_items = rowspan;
        iod_module_item.ie = parse_td_ie(td_ie)?;
        let mru = CompositeModuleReferenceUsage {
            module: parse_td_module(vtd.get(1).unwrap())?,
            reference: helper::parse_td_reference(vtd.get(2).unwrap())?,
            usage: helper::parse_td_usage(vtd.get(3).unwrap())?,
        };
        iod_module_item.items.push(mru);
    } else if ntd == 3 {
        let mru = CompositeModuleReferenceUsage {
            module: parse_td_module(vtd.get(0).unwrap())?,
            reference: helper::parse_td_reference(vtd.get(1).unwrap())?,
            usage: helper::parse_td_usage(vtd.get(2).unwrap())?,
        };
        iod_module_item.items.push(mru);
    } else {
        return Err(ParserError::XmlTableInvalidNumberOfColumns(
            "3 or 4".to_string(),
            ntd,
        ));
    }
    Ok(iod_module_item)
}

/// Parse an XML td element and extract a DICOM IOD module type.
///
/// # Arguments
///
/// * `td` - XML element (expecting a td element)
fn parse_td_iod_module_type(td: &Element) -> Result<IodModuleType, ParserError> {
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
        "failed to parse Attribute Type from para element".to_string(),
    );
    let txt = para.text_trim().ok_or(err)?;
    return match IodModuleType::from_str(txt.as_str()) {
        Ok(type_) => Ok(type_),
        Err(e) => Err(e.into()),
    };
}

/// Parse a row in a IOD module table.
///
/// # Arguments
///
/// * `tr` - XML element (expecting a tr element)
fn parse_iod_module_table_row(tr: &Element) -> Result<Option<ModuleDefinitionItem>, ParserError> {
    let vtd = tr.find_children(|t| t.name == QualifiedName::from_str("td").unwrap());
    let ntd = vtd.len();

    let mdi;
    if ntd == 4 {
        let mod_attr = ModuleAttribute {
            name: helper::parse_td_attribute_name(vtd.get(0).unwrap())?,
            tag: helper::parse_td_tag(vtd.get(1).unwrap())?,
            type_: parse_td_iod_module_type(vtd.get(2).unwrap())?,
            description: parse_td_description(vtd.get(3).unwrap())?,
        };
        mdi = ModuleDefinitionItem::Module(mod_attr);
    } else if ntd > 0 && ntd <= 2 {
        let mut include = ModuleAttributeInclude::default();
        let text = helper::parse_td_include_text(vtd.get(0).unwrap());
        let link = helper::parse_td_include_link(vtd.get(0).unwrap());
        if text.is_err() || link.is_err() {
            return Ok(None);
        }

        include.text = text.unwrap();
        include.link = link.unwrap();
        if ntd == 2 {
            let descr = parse_td_description(vtd.get(1).unwrap());
            include.description = descr.unwrap();
        } else {
            include.description = "".to_string();
        }
        mdi = ModuleDefinitionItem::Include(include);
    } else if ntd == 3 {
        // normative IOD module tables rarely have 3 columns
        let name = helper::parse_td_attribute_name(vtd.get(0).unwrap())?;
        if !name.contains("Any Attribute from the top level Data Set") {
            return Err(ParserError::XmlTableInvalidNumberOfColumns(
                "1, 2 or 4".to_string(),
                ntd,
            ));
        }
        let top_lvl_attr = ModuleAttributeReferenceTopLevelAttributes {
            text: name,
            type_: parse_td_iod_module_type(vtd.get(1).unwrap())?,
            description: parse_td_description(vtd.get(2).unwrap())?,
        };
        mdi = ModuleDefinitionItem::TopLevelAttributes(top_lvl_attr);
    } else {
        return Err(ParserError::XmlTableInvalidNumberOfColumns(
            "1, 2 or 4".to_string(),
            ntd,
        ));
    }
    Ok(Some(mdi))
}

/// Parse an XML column with a IE description.
///
/// # Arguments
///
/// * `e` - XML element (expecting a td element)
fn parse_td_ie(td: &Element) -> Result<String, ParserError> {
    let err = ParserError::XmlTableColumnParse(
        "unable to extract IE text".to_string(),
    );
    td_or_nested_para_text(td).ok_or(err)
}

/// Parse an XML column with a Module description.
///
/// # Arguments
///
/// * `e` - XML element (expecting a td element)
fn parse_td_module(td: &Element) -> Result<String, ParserError> {
    let err = ParserError::XmlTableColumnParse(
        "unable to extract Module text".to_string(),
    );
    td_or_nested_para_text(td).ok_or(err)
}

/// Parse a td XML element and extract the description.
///
/// Text from all the child nodes from the td Element are recursively extracted.
///
/// # Arguments
///
/// * `td` - XML element (expecting a td element)
fn parse_td_description(td: &Element) -> Result<String, ParserError> {
    if td.name != QualifiedName::from_str("td").unwrap() {
        return Err(ParserError::XmlTableColumnParse(
            "expected td element".to_string(),
        ));
    }
    Ok(td.text_trim().unwrap_or_default())
}

pub mod library {
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::str::FromStr;

    use log::{debug, info};

    use dicom_std_core::{IODLibrary, ModuleDefinitionItem};

    use crate::dom::model::{Element, QualifiedName};
    use crate::iod::filter_composite_iod_module_sections;
    use crate::query::link_to_table_or_nested_tables;
    use crate::{query, IODLibraryError};

    /// Build an IOD library by parsing the root of part 03 of the DICOM standard.
    ///
    /// # Arguments
    ///
    /// * `element`: root of the XML DOM structure referencing part 03
    ///
    /// returns: Result<IODLibrary, IODLibraryError>
    pub fn build(element: &Element) -> Result<IODLibrary, IODLibraryError> {
        info!("Building IOD library");
        let mut library = IODLibrary::default();
        let ciod_sections = filter_composite_iod_module_sections(element);

        let qn_table = QualifiedName::from_str("table")?;

        info!("Parsing composite IOD modules");
        for section in ciod_sections {
            let tables = section.find_children(|t| t.name == qn_table);
            for table in tables {
                let r = super::parse_composite_iod_module_table(table);
                if let Ok(ciom) = r {
                    library.composite.push(ciom);
                }
            }
        }

        let link_exceptions = ["sect_C.19.1.1"];

        let mut module_tables = BTreeMap::new();
        let mut module_references_parsed: BTreeMap<String, Vec<String>> = BTreeMap::new();

        let mut include_map = BTreeMap::new();
        let vcoidm = &library.composite;
        info!("Parsing normative IOD modules");
        for coidm in vcoidm {
            for coidm_item in &coidm.items {
                for cmru in &coidm_item.items {
                    let link = &cmru.reference;
                    if link.target.is_empty() {
                        continue;
                    }
                    // If a module and reference link in was already processed once, no need to do it again.
                    if module_references_parsed.contains_key(&cmru.module) {
                        let references = module_references_parsed.get_mut(&cmru.module).unwrap();
                        if references.contains(&link.target) {
                            continue;
                        } else {
                            references.push(link.target.clone());
                        }
                    } else {
                        module_references_parsed.insert(cmru.module.clone(), vec![]);
                    }

                    if !module_tables.contains_key(&cmru.module) {
                        module_tables.insert(cmru.module.clone(), vec![]);
                    }

                    debug!("Composite IOD module link: {}", link.target.as_str());
                    let vttables = link_to_table_or_nested_tables(element, link)?;
                    debug!(" number of tables found: {}", vttables.len());
                    for table in vttables {
                        if let Ok(md) = super::parse_iod_module_table(table) {
                            {
                                let tables_ids = module_tables.get_mut(&cmru.module).unwrap();
                                let b = tables_ids.contains(&md.id);
                                if !b {
                                    tables_ids.push(md.id.clone());
                                }
                            }
                            debug!("  parsed module ID: {}", md.id.as_str());
                            // lookup all the xref links
                            let mut xrefs_vmd = vec![];
                            let mut i: usize = 0;
                            let mut n: usize = 0;
                            let mut first_iter = true;
                            while i != n || first_iter {
                                if first_iter {
                                    first_iter = false;
                                }
                                for md_item in &md.items {
                                    match md_item {
                                        ModuleDefinitionItem::None => {}
                                        ModuleDefinitionItem::Module(_) => {}
                                        ModuleDefinitionItem::Macro(_) => {}
                                        ModuleDefinitionItem::Include(include) => {
                                            let id = include.link.target.as_str();
                                            if include_map.contains_key(id)
                                                || library.normative.contains_key(id)
                                            {
                                                debug!("   include link already exists: {}", id);
                                                continue;
                                            }
                                            if let Some(table) = query::id(element, id) {
                                                if link_exceptions
                                                    .contains(&include.link.target.as_str())
                                                {
                                                    debug!(
                                                        "   include link exception: {}",
                                                        &include.link
                                                    );
                                                    continue;
                                                } else {
                                                    debug!("   include link: {}", &include.link);
                                                    let tv = link_to_table_or_nested_tables(
                                                        table,
                                                        &include.link,
                                                    )?;
                                                    for t in tv {
                                                        let tmd = super::parse_iod_module_table(t)?;
                                                        debug!(
                                                            "    parsed normative IOD table: {}",
                                                            &tmd.id.as_str()
                                                        );
                                                        assert_eq!(id, tmd.id.as_str());
                                                        xrefs_vmd.push(tmd.clone());
                                                        include_map.insert(tmd.id.clone(), tmd);
                                                    }
                                                }
                                            }
                                        }
                                        ModuleDefinitionItem::TopLevelAttributes(_) => {}
                                    }
                                }
                                i = n;
                                n = xrefs_vmd.len();
                            }
                            // for include_md in xrefs_vmd {
                            //     let id = include_md.id.clone();
                            //     include_map.insert(id, include_md);
                            // }

                            let id = md.id.clone();
                            library.normative.entry(id).or_insert(md);
                        }
                    }

                    // Add link to the module to indicate it was processed
                    let references = module_references_parsed.get_mut(&cmru.module).unwrap();
                    references.push(link.target.clone());
                }
            }
        }
        library.module_tables_ids.extend(module_tables);
        library.normative.extend(include_map);
        Ok(library)
    }

    /// Build an IOD library by reading part 03 of the DICOM standard.
    ///
    /// # Arguments
    ///
    /// * `path`: file path to a DICOM XML file of part 03 of the DICOM standard
    ///
    /// returns: Result<IODLibrary, IODLibraryError>
    pub fn build_from_path<P>(path: P) -> Result<IODLibrary, IODLibraryError>
    where
        P: AsRef<Path>,
    {
        let root = crate::dom::read_file(path)?;
        build(&root)
    }

    #[cfg(test)]
    mod tests {
        use crate::dom;
        use crate::tests::path_dicom_part;

        #[test]
        fn build() {
            // init_logger();
            let root = dom::read_file(path_dicom_part(3));
            assert!(root.is_ok(), "{:?}", root.err().unwrap());
            let root = root.unwrap();
            let library = super::build(&root);
            assert!(library.is_ok(), "{}", library.err().unwrap());
            let library = library.unwrap();
            assert!(!library.composite.is_empty());
            assert!(!library.normative.is_empty());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use log::{debug, trace};

    use dicom_std_core::{
        CompositeIodModuleItem, CompositeModuleReferenceUsage, IodModuleType, Link,
        ModuleAttribute, ModuleAttributeInclude, ModuleDefinitionItem, Tag, Usage,
    };

    use crate::dom::model::{Element, QualifiedName};
    use crate::iod::iod::filter_composite_iod_module_sections;
    use crate::tests::{init_logger, path_dicom_part};
    use crate::{dom, helper, query, ParserError};

    mod test_helper {
        use crate::dom::model::Element;
        use crate::ParserError;
        use dicom_std_core::CompositeIodModule;

        /// Parse a list of XML table elements into a list of composite IOD modules.
        ///
        /// # Arguments
        ///
        /// * `ve` - list of XML table elements
        pub(crate) fn parse_composite_iod_module_tables(
            ve: &Vec<&Element>,
        ) -> Vec<Result<CompositeIodModule, ParserError>> {
            let mut v = vec![];
            for e in ve {
                v.push(super::super::parse_composite_iod_module_table(*e));
            }
            v
        }
    }

    #[test]
    fn parse_file() {
        init_logger();
        let part03 = path_dicom_part(3);
        debug!("path to part03: {:?}", &part03);
        let root = dom::read_file(part03);
        let root = root.unwrap();
        let viod_sections = filter_composite_iod_module_sections(&root);
        assert!(!viod_sections.is_empty());
    }

    #[test]
    fn filter_composite_iod_module_tables() {
        init_logger();
        let part03 = path_dicom_part(3);

        let root = dom::read_file(part03);
        let root = root.unwrap();
        let viod_sections = filter_composite_iod_module_sections(&root);
        let vtables = helper::filter_tables(&viod_sections);
        for table in &vtables {
            let caption =
                table.find_child(|t| t.name == QualifiedName::from_str("caption").unwrap());
            if caption.is_none() {
                continue;
            }
            let caption = caption.unwrap();
            trace!("- {}", caption.text().unwrap_or_default());
        }
        assert!(!vtables.is_empty());
    }

    #[test]
    fn parse_composite_iod_module_table() {
        init_logger();
        let part03 = path_dicom_part(3);
        let root = dom::read_file(part03);
        let root = root.unwrap();
        let table = query::id(&root, "table_A.2-1");
        assert!(table.is_some(), "failed to find table by id");
        let table = table.unwrap();
        let iod_module_table = super::parse_composite_iod_module_table(table);
        assert!(
            iod_module_table.is_ok(),
            "{}",
            iod_module_table.err().unwrap()
        );
        let iod_module_table = iod_module_table.unwrap();
        assert_eq!(iod_module_table.caption.as_str(), "CR Image IOD Modules");
        assert_eq!(iod_module_table.items.len(), 6);
        let ve_imi = vec![
            CompositeIodModuleItem {
                ie: "Patient".to_string(),
                items: vec![
                    CompositeModuleReferenceUsage {
                        module: "Patient".to_string(),
                        reference: Link {
                            target: "sect_C.7.1.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Clinical Trial Subject".to_string(),
                        reference: Link {
                            target: "sect_C.7.1.3".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                ],
                expected_items: 2,
            },
            CompositeIodModuleItem {
                ie: "Study".to_string(),
                items: vec![
                    CompositeModuleReferenceUsage {
                        module: "General Study".to_string(),
                        reference: Link {
                            target: "sect_C.7.2.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Patient Study".to_string(),
                        reference: Link {
                            target: "sect_C.7.2.2".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Clinical Trial Study".to_string(),
                        reference: Link {
                            target: "sect_C.7.2.3".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                ],
                expected_items: 3,
            },
            CompositeIodModuleItem {
                ie: "Series".to_string(),
                items: vec![
                    CompositeModuleReferenceUsage {
                        module: "General Series".to_string(),
                        reference: Link {
                            target: "sect_C.7.3.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "CR Series".to_string(),
                        reference: Link {
                            target: "sect_C.8.1.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Clinical Trial Series".to_string(),
                        reference: Link {
                            target: "sect_C.7.3.2".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                ],
                expected_items: 3,
            },
            CompositeIodModuleItem {
                ie: "Equipment".to_string(),
                items: vec![CompositeModuleReferenceUsage {
                    module: "General Equipment".to_string(),
                    reference: Link {
                        target: "sect_C.7.5.1".to_string(),
                        style: "select: labelnumber".to_string(),
                    },
                    usage: Usage::M,
                }],
                expected_items: 1,
            },
            CompositeIodModuleItem {
                ie: "Acquisition".to_string(),
                items: vec![CompositeModuleReferenceUsage {
                    module: "General Acquisition".to_string(),
                    reference: Link {
                        target: "sect_C.7.10.1".to_string(),
                        style: "select: labelnumber".to_string(),
                    },
                    usage: Usage::M,
                }],
                expected_items: 1,
            },
            CompositeIodModuleItem {
                ie: "Image".to_string(),
                items: vec![
                    CompositeModuleReferenceUsage {
                        module: "General Image".to_string(),
                        reference: Link {
                            target: "sect_C.7.6.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "General Reference".to_string(),
                        reference: Link {
                            target: "sect_C.12.4".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Image Pixel".to_string(),
                        reference: Link {
                            target: "sect_C.7.6.3".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Contrast/Bolus".to_string(),
                        reference: Link {
                            target: "sect_C.7.6.4".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::C,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Display Shutter".to_string(),
                        reference: Link {
                            target: "sect_C.7.6.11".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Device".to_string(),
                        reference: Link {
                            target: "sect_C.7.6.12".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Specimen".to_string(),
                        reference: Link {
                            target: "sect_C.7.6.22".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "CR Image".to_string(),
                        reference: Link {
                            target: "sect_C.8.1.2".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Overlay Plane".to_string(),
                        reference: Link {
                            target: "sect_C.9.2".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Modality LUT".to_string(),
                        reference: Link {
                            target: "sect_C.11.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "VOI LUT".to_string(),
                        reference: Link {
                            target: "sect_C.11.2".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                    CompositeModuleReferenceUsage {
                        module: "SOP Common".to_string(),
                        reference: Link {
                            target: "sect_C.12.1".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::M,
                    },
                    CompositeModuleReferenceUsage {
                        module: "Common Instance Reference".to_string(),
                        reference: Link {
                            target: "sect_C.12.2".to_string(),
                            style: "select: labelnumber".to_string(),
                        },
                        usage: Usage::U,
                    },
                ],
                expected_items: 13,
            },
        ];
        assert_eq!(ve_imi, iod_module_table.items);
    }

    #[test]
    fn parse_composite_iod_module_tables() {
        init_logger();
        let part03 = path_dicom_part(3);
        let root = dom::read_file(part03);
        let root = root.unwrap();
        let chapter_a = query::id(&root, "chapter_A");
        assert!(chapter_a.is_some(), "unable to find chapter_A");
        let chapter_a = chapter_a.unwrap();
        let viod_sections = filter_composite_iod_module_sections(chapter_a);
        let vtables = helper::filter_tables(&viod_sections);
        assert!(!vtables.is_empty());
        let viod_modules = test_helper::parse_composite_iod_module_tables(&vtables);
        let mut filtered_iod_modules = vec![];
        for (table, iod_module) in vtables.iter().zip(viod_modules.iter()) {
            let caption =
                table.find_child(|t| t.name == QualifiedName::from_str("caption").unwrap());
            if caption.is_none() {
                continue;
            }
            let caption = caption.unwrap();
            // trace!("- {}", caption.text.as_ref().unwrap_or(&"".to_string()));
            match iod_module {
                Ok(value) => {
                    filtered_iod_modules.push(value);
                }
                Err(e) => match e {
                    ParserError::NotIodModuleTable => {}
                    _ => {
                        panic!("- {}", caption.text().unwrap_or_default());
                    }
                },
            }
        }
        let n_filtered_iod_modules = filtered_iod_modules.len();
        assert!(n_filtered_iod_modules > 0);
    }

    #[test]
    fn parse_iod_module_table() {
        init_logger();
        let part03 = path_dicom_part(3);
        let root = dom::read_file(part03);
        let root = root.unwrap();
        let table = query::id(&root, "table_C.7-1");
        assert!(table.is_some());
        let table = table.unwrap();
        let iod_table = super::parse_iod_module_table(table);
        assert!(iod_table.is_ok(), "{}", iod_table.err().unwrap());
        let iod_table = iod_table.unwrap();
        assert!(!iod_table.items.is_empty());
        assert_eq!(iod_table.caption.as_str(), "Patient Module Attributes");
        assert_eq!(iod_table.id.as_str(), "table_C.7-1");
        let ve_mdi = vec![
            ModuleDefinitionItem::Module(ModuleAttribute {
                name: "Patient's Name".to_string(),
                tag: Tag {
                    g: 0x0010,
                    e: 0x0010,
                },
                type_: IodModuleType::Two,
                description: "".to_string(),
            }),
            ModuleDefinitionItem::Module(ModuleAttribute {
                name: "Patient ID".to_string(),
                tag: Tag {
                    g: 0x0010,
                    e: 0x0020,
                },
                type_: IodModuleType::Two,
                description: "".to_string(),
            }),
            ModuleDefinitionItem::Include(ModuleAttributeInclude {
                text: "Include".to_string(),
                link: Link {
                    target: "table_10-18".to_string(),
                    style: "select: label quotedtitle".to_string(),
                },
                description: "".to_string(),
            }),
        ];
        let n = ve_mdi.len();
        for i in 0..n {
            let e_mdi: &ModuleDefinitionItem = ve_mdi.get(i).unwrap();
            let mdi: &ModuleDefinitionItem = iod_table.items.get(i).unwrap();
            match mdi {
                ModuleDefinitionItem::None => {
                    panic!()
                }
                ModuleDefinitionItem::Module(module) => match e_mdi {
                    ModuleDefinitionItem::Module(e_module) => {
                        assert_eq!(module.name, e_module.name);
                        assert_eq!(module.tag, e_module.tag);
                        assert_eq!(module.type_, e_module.type_);
                    }
                    _ => {
                        panic!()
                    }
                },
                ModuleDefinitionItem::Macro(_) => {
                    panic!()
                }
                ModuleDefinitionItem::Include(include) => match e_mdi {
                    ModuleDefinitionItem::Include(e_include) => {
                        assert_eq!(include.text, e_include.text);
                        assert_eq!(include.link, e_include.link);
                    }
                    _ => {
                        panic!()
                    }
                },
                ModuleDefinitionItem::TopLevelAttributes(_) => {}
            }
        }
    }

    #[test]
    fn parse_td_ie() {
        init_logger();
        let xml = r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_bc79bc38-fded-4e83-b6cf-358317aeb7a2">Patient</para>
        </td>"#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let ie = super::parse_td_ie(&root);
        assert!(ie.is_ok(), "{}", ie.err().unwrap());
        let ie = ie.unwrap();
        assert_eq!(ie.as_str(), "Patient");
    }

    #[test]
    fn parse_td_module() {
        init_logger();
        let xml = r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_834fae3d-253a-4360-9a6f-94aee097dc49">Clinical Trial Subject</para>
        </td>"#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let module = super::parse_td_module(&root);
        assert!(module.is_ok(), "{}", module.err().unwrap());
        let module = module.unwrap();
        assert_eq!(module.as_str(), "Clinical Trial Subject");
    }

    #[test]
    fn parse_td_reference() {
        init_logger();
        let xml = r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_64574519-b874-4628-8258-5476c306da79">
                <xref linkend="sect_C.7.2.1" xrefstyle="select: labelnumber"/>
            </para>
        </td>
        "#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let reference = helper::parse_td_reference(&root);
        assert!(reference.is_ok(), "{}", reference.err().unwrap());
        let reference = reference.unwrap();
        assert_eq!(reference.target.as_str(), "sect_C.7.2.1");
        assert_eq!(reference.style.as_str(), "select: labelnumber");
    }

    #[test]
    fn parse_td_usage() {
        init_logger();
        let xmls = [
            r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_cf0b8a2b-c59e-407e-a1c0-45924a108d74">M</para>
        </td>
       "#,
            r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_cf0b8a2b-c59e-407e-a1c0-45924a108d74">C</para>
        </td>
       "#,
            r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_cf0b8a2b-c59e-407e-a1c0-45924a108d74">U</para>
        </td>
       "#,
            r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_cf0b8a2b-c59e-407e-a1c0-45924a108d74"></para>
        </td>
       "#,
            r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_cf0b8a2b-c59e-407e-a1c0-45924a108d74">W</para>
        </td>
       "#,
        ];
        let usages = [Usage::M, Usage::C, Usage::U, Usage::None, Usage::None];
        let index = Vec::from_iter(0..xmls.len());
        for (xml, (e_usage, i)) in xmls.iter().zip(usages.iter().zip(index.iter())) {
            let root = Element::from_str(xml);
            assert!(root.is_ok(), "{:?}", root.err().unwrap());
            let root = root.unwrap();
            let usage = helper::parse_td_usage(&root);
            if *i < 3 {
                assert!(usage.is_ok(), "{}", usage.err().unwrap());
                let usage = usage.unwrap();
                assert_eq!(&usage, e_usage);
            } else {
                assert!(usage.is_err(), "{}", usage.ok().unwrap());
            }
        }
    }

    #[test]
    fn parse_td_attribute_name() {
        init_logger();
        let xml = r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_414ce351-5da3-4e6d-865f-600f9df1733a">
                Referenced Patient Photo Sequence
            </para>
        </td>
        "#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let attr_name = helper::parse_td_attribute_name(&root);
        assert!(attr_name.is_ok(), "{}", attr_name.err().unwrap());
        let attr_name = attr_name.unwrap();
        assert_eq!(attr_name.as_str(), "Referenced Patient Photo Sequence");
    }

    #[test]
    fn parse_td_tag() {
        init_logger();
        let xml = r#"
        <td align="center" colspan="1" rowspan="1">
            <para xml:id="para_856d3196-9add-4403-ba33-705a1e67b568">(0010,1100)</para>
        </td>
        "#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let tag = helper::parse_td_tag(&root);
        assert!(tag.is_ok(), "{}", tag.err().unwrap());
        let tag = tag.unwrap();
        assert_eq!(
            tag,
            Tag {
                g: 0x0010,
                e: 0x1100,
            }
        );
    }

    #[test]
    fn parse_td_type() {
        init_logger();
        let vxml = vec![
            r#"
            <td align="center" colspan="1" rowspan="1">
                <para xml:id="para_e9b959e4-8289-4fd7-87fc-6aad978153b4">1</para>
            </td>
            "#,
            r#"
            <td align="center" colspan="1" rowspan="1">
                <para xml:id="para_e9b959e4-8289-4fd7-87fc-6aad978153b4">1C</para>
            </td>
            "#,
            r#"
            <td align="center" colspan="1" rowspan="1">
                <para xml:id="para_e9b959e4-8289-4fd7-87fc-6aad978153b4">2</para>
            </td>
            "#,
            r#"
            <td align="center" colspan="1" rowspan="1">
                <para xml:id="para_e9b959e4-8289-4fd7-87fc-6aad978153b4">2C</para>
            </td>
            "#,
            r#"
            <td align="center" colspan="1" rowspan="1">
                <para xml:id="para_e9b959e4-8289-4fd7-87fc-6aad978153b4">3</para>
            </td>
            "#,
        ];
        let e_vtypes = vec![
            IodModuleType::One,
            IodModuleType::OneC,
            IodModuleType::Two,
            IodModuleType::TwoC,
            IodModuleType::Three,
        ];
        for (xml, e_type) in vxml.iter().zip(e_vtypes.iter()) {
            let root = Element::from_str(xml);
            assert!(root.is_ok(), "{:?}", root.err().unwrap());
            let root = root.unwrap();
            let type_ = super::parse_td_iod_module_type(&root);
            assert!(type_.is_ok(), "{}", type_.err().unwrap());
            let type_ = type_.unwrap();
            assert_eq!(e_type, &type_);
        }
    }

    #[test]
    fn parse_td_description() {
        init_logger();
        let xml = r#"
        <td align="left" colspan="1" rowspan="1">
            <para xml:id="para_073ec5af-fb7b-4a46-a5f9-6176a0417b4a">A photo to confirm the identity of a Patient.</para>
            <para xml:id="para_cba8cef9-df02-4e26-a526-4e3738514a36">Only a single Item is permitted in this Sequence.</para>
            <para xml:id="para_0bb4b3ec-a800-46c3-a0ef-9d5bac390511">See <xref linkend="sect_C.2.2.1.1" xrefstyle="select: labelnumber"/>.</para>
        </td>
        "#;
        let root = Element::from_str(xml);
        assert!(root.is_ok(), "{:?}", root.err().unwrap());
        let root = root.unwrap();
        let desc = super::parse_td_description(&root);
        assert!(desc.is_ok(), "{}", desc.err().unwrap());
        let desc = desc.unwrap();
        let e_desc =
            "A photo to confirm the identity of a Patient.\nOnly a single Item is permitted in this Sequence.\nSee .";
        assert_eq!(desc, e_desc);
    }
}
