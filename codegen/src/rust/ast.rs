use log::error;
use std::collections::BTreeMap;

use dicom_std_core::{
    DataDictionary, DataDictionaryEntry, DicomStandard, IODLibrary, ModuleAttribute,
    ModuleAttributeInclude, ModuleDefinition, ModuleDefinitionItem, Tag,
};
use dicom_std_utils::is_char_whitespace_or_return;

use crate::rust::{RustAstError, RustLanguageTranslator, Struct, StructField, Trait, Visibility};
use crate::{Error, LanguageTranslateError, LanguageTranslator};

/// Count the number of `>` at the start of a string.
///
/// # Arguments
/// * `s` - string
fn depth_by_name(s: &str) -> i32 {
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
    let var_name = RustLanguageTranslator::get_variable_name(&entry.keyword)?;
    if entry.vr.is_empty() {
        return Err(RustAstError::DictionaryEntryHasNoVR(entry.tag));
    }
    let first_vr = entry.vr.get(0).unwrap();
    let obj_name = RustLanguageTranslator::get_object_types_by_vr(*first_vr);
    return if entry.is_seq() {
        Ok(StructField {
            visibility: Visibility::Public,
            name: format!("{}_seq", var_name),
            lifetime: None,
            reference: false,
            type_: format!("Vec<{}>", obj_name.get(0).unwrap()),
        })
    } else {
        Ok(StructField {
            visibility: Visibility::Public,
            name: var_name,
            lifetime: None,
            reference: false,
            type_: obj_name.get(0).unwrap().to_string(),
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
        let ast_builder = RustAstBuilder::default();
        let field = module_attribute_to_field(module_attributes.get(0).unwrap(), &dict);
        assert!(field.is_ok(), "{}", field.err().unwrap());
        let field = field.unwrap();
        assert_eq!(field.visibility, Visibility::Public);
        assert_eq!(field.name.as_str(), "patient_name");
        // assert_eq!(field.type_, "patient_name");
    }

    #[test]
    fn module_definition_to_struct() {
        let iod_library = read_iod_library();
        let data_dictionary = read_data_dictionary();
        print!("normative n={}", iod_library.normative.len());
    }
}
