use log::error;
use std::collections::BTreeMap;
use std::fmt::format;

use dicom_std_core::{DataDictionary, DataDictionaryEntry, DicomStandard, IODLibrary, ModuleAttribute, ModuleAttributeInclude, ModuleDefinition, ModuleDefinitionItem, Tag, VR};
use dicom_std_utils::is_char_whitespace_or_return;

use crate::rust::{RustAstError, RustLanguageTranslator, Struct, StructField, Trait, Visibility};
use crate::{Error, LanguageTranslateError, LanguageTranslator};

/// Count the number of `>` at the start of a string.
///
/// # Arguments
/// * `s` - string
fn depth_by_name(s: &str) -> i16 {
    let t = s.trim_matches(is_char_whitespace_or_return);
    let mut n = 0;
    for c in t.chars() {
        if c != '>' {
            break;
        }
        n += 1;
    }
    n
}
/// Test if a DICOM tag is a sequence.
///
/// # Arguments
/// * `tag` - DICOM tag
/// * `data_dictionary` - data dictionary containing info about DICOM tags
fn is_sequence(tag: &Tag, data_dictionary: &DataDictionary) -> bool {
    match data_dictionary.by_tag(*tag) {
        None => false,
        Some(entry) => entry.is_seq(),
    }
}

/// Create a sequence item type from a keyword from the data dictionary.
///
/// Output format: <keyword>Item
/// * `AnatomicRegionModifierSequence` -> `AnatomicRegionModifierSequenceItem`
///
/// # Arguments
/// * `keyword` - data dictionary entry keyword
pub(crate) fn sequence_item_type_from_keyword(keyword: &str) -> Result<String, LanguageTranslateError> {
    let t = RustLanguageTranslator::get_object_name(keyword)?;
    Ok(format!("{}Item", t))
}

/// Create a sequence type from a keyword from the data dictionary.
///
/// Output format:
/// * `AnatomicRegionModifierSequence` -> `Vec<AnatomicRegionModifierSequenceItem>`
///
/// # Arguments
/// * `keyword` - data dictionary entry keyword
pub(crate) fn sequence_type_from_keyword(keyword:&str) -> Result<String, LanguageTranslateError> {
    let t = sequence_item_type_from_keyword(keyword)?;
    let v = RustLanguageTranslator::get_object_types_by_vr(VR::SQ).get(0).unwrap().to_string();
    Ok(format!("{}<{}>", v, t))
}

#[derive(Debug, Clone, Default)]
pub struct RustAstBuilder {
    // composite_iod_traits: Vec<Trait>,
    ie_traits: Vec<Trait>,
    normative: BTreeMap<String, Trait>,
}

/// Create a field in a struct form a module attribute.
///
/// # Arguments
///
/// * `module_attribute`: module attribute describing a dicom tag, keyword, description ...
/// * `dict`: dicom data dictionary with a list of known elements
///
/// returns: Result<StructField, RustAstError>
/// ```
pub(crate) fn module_attribute_to_field(
    module_attribute: &ModuleAttribute,
    dict: &DataDictionary,
) -> Result<StructField, RustAstError> {
    if module_attribute.tag.g == 0 && module_attribute.tag.e == 0 {
        return Err(RustAstError::ModuleDefinitionTagEmpty);
    }
    let entry = dict
        .by_tag(module_attribute.tag)
        .ok_or(RustAstError::TagNotFoundDataDict(module_attribute.tag))?;
    let depth = depth_by_name(&entry.name);
    let var_name = RustLanguageTranslator::get_variable_name(&entry.keyword)?;
    if entry.vr.is_empty() {
        return Err(RustAstError::DictionaryEntryHasNoVR(entry.tag));
    }
    let first_vr = entry.vr.get(0).unwrap();
    let mut obj_name = RustLanguageTranslator::get_object_types_by_vr(*first_vr);
    return if entry.is_seq() {
        let mut t = sequence_item_type_from_keyword(var_name.as_str())?;
        Ok(StructField {
            visibility: Visibility::Public,
            name: format!("{}_item", var_name),
            lifetime: None,
            reference: false,
            type_: sequence_type_from_keyword(var_name.as_str())?,
            depth,
        })
    } else {
        Ok(StructField {
            visibility: Visibility::Public,
            name: var_name,
            lifetime: None,
            reference: false,
            type_: obj_name.get(0).unwrap().to_string(),
            depth,
        })
    };
}

#[cfg(test)]
mod tests {
    use dicom_std_core::{
        DataDictionary, DataDictionaryEntry, IODLibrary, IodModuleType, ModuleAttribute, Tag,
        TagRange, VM, VR,
    };
    use std::path::PathBuf;

    use crate::rust::{module_attribute_to_field, RustAstBuilder, Visibility};
    use crate::{read_data_dictionary, read_iod_library, test_resource_dir};

    #[test]
    fn sequence_item_type_from_keyword() {
        assert_eq!(super::sequence_item_type_from_keyword(
            "AnatomicRegionModifierSequence").unwrap().as_str(),
                   "AnatomicRegionModifierSequenceItem");
        assert_eq!(super::sequence_item_type_from_keyword(
            "Anatomic Region Modifier Sequence").unwrap().as_str(),
                   "AnatomicRegionModifierSequenceItem");
    }

    #[test]
    fn module_attribute_to_struct_field() {
        let module_attributes = vec![
            ModuleAttribute {
                name: "Patient's Name".to_string(),
                tag: Tag::new(0x0010, 0x0010),
                type_: IodModuleType::Two,
                description: "Patient's full name.".to_string(),
            },
            ModuleAttribute {
                name: "Patient's Name".to_string(),
                tag: Tag::new(0x0010, 0x0010),
                type_: IodModuleType::Two,
                description: "Patient's full name.".to_string(),
            },
            ModuleAttribute {
                name: "Anatomic Region Modifier Sequence".to_string(),
                tag: Tag::new(0x0008, 0x2220),
                type_: IodModuleType::Three,
                description: "Patient's full name.".to_string(),
            },
        ];
        let mut dict = DataDictionary::default();
        dict.add(DataDictionaryEntry {
            tag: TagRange::from(Tag::new(0x0010, 0x0010)),
            name: "Patient's Name".to_string(),
            keyword: "PatientName".to_string(),
            vr: vec![VR::PN],
            vm: VM::from(1),
            description: "".to_string(),
            retired: false,
        });
        dict.add(DataDictionaryEntry {
            tag: TagRange::from(Tag::new(0x0010, 0x0020)),
            name: "Patient ID".to_string(),
            keyword: "PatientID".to_string(),
            vr: vec![VR::LO],
            vm: VM::from(1),
            description: "".to_string(),
            retired: false,
        });
        dict.add(DataDictionaryEntry{
            tag: TagRange::from(Tag::new(0x0008, 0x2220)),
            name: ">Anatomic Region Modifier Sequence".to_string(),
            keyword: "AnatomicRegionModifierSequence".to_string(),
            vr: vec![VR::SQ],
            vm: VM::from(1),
            description: "".to_string(),
            retired: false
        });
        let ast_builder = RustAstBuilder::default();
        let field = module_attribute_to_field(module_attributes.get(0).unwrap(), &dict);
        assert!(field.is_ok(), "{}", field.err().unwrap());
        let field = field.unwrap();
        assert_eq!(field.visibility, Visibility::Public);
        assert_eq!(field.name.as_str(), "patient_name");
        // assert_eq!(field.type_, "patient_name");
        let field_seq = module_attribute_to_field(module_attributes.get(2).unwrap(), &dict);
        assert!(field_seq.is_ok());
        let field_seq = field_seq.unwrap();
        assert_eq!(field_seq.visibility, Visibility::Public);
        assert_eq!(field_seq.name.as_str(), "anatomic_region_modifier_sequence_item");
        assert_eq!(field_seq.type_.as_str(), "Vec<AnatomicRegionModifierSequenceItem>");
    }

    #[test]
    fn module_definition_to_struct() {
        let iod_library = read_iod_library();
        let data_dictionary = read_data_dictionary();
        print!("normative n={}", iod_library.normative.len());
    }
}
