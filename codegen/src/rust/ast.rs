use log::{debug, error};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::format;

use dicom_std_core::{
    DataDictionary, DataDictionaryEntry, DicomStandard, IODLibrary, ModuleAttribute,
    ModuleAttributeInclude, ModuleDefinition, ModuleDefinitionItem, Tag, VR,
};
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
pub(crate) fn sequence_item_type_from_keyword(
    keyword: &str,
) -> Result<String, LanguageTranslateError> {
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
pub(crate) fn sequence_type_from_keyword(keyword: &str) -> Result<String, LanguageTranslateError> {
    let t = sequence_item_type_from_keyword(keyword)?;
    let v = RustLanguageTranslator::get_object_types_by_vr(VR::SQ)
        .get(0)
        .unwrap()
        .to_string();
    Ok(format!("{}<{}>", v, t))
}

#[derive(Debug, Clone, Default)]
pub struct RustAstBuilder {
    // composite_iod_traits: Vec<Trait>,
    ie_traits: Vec<Trait>,
    normative: BTreeMap<String, Trait>,
}

/// Data struct storing the indices and depth at which a sequence starts and ends.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) struct SeqIndex {
    /// Depth of the sequence
    depth: i16,
    /// Index at which the sequence starts
    start: usize,
    /// Index at which the sequence ends (one past the last sequence member)
    end: usize,
}

impl PartialOrd for SeqIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let order = self.start.partial_cmp(&other.start)?;
        return if order != Ordering::Equal {
            Some(order)
        } else {
            self.end.partial_cmp(&other.end)
        };
    }
}

impl Ord for SeqIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Find the indices in the field where (nested) sequences start and finish.
///
/// The algorithm is based on changes detecting the depth changes between StructFields.
///
/// Expected increases in depth are not expected to exceed 1. If it does, the function will panic.
///
/// # Arguments
/// * `fields` - slice of [StructField]
pub(crate) fn find_seq_indices_begin_end(fields: &[StructField]) -> Vec<SeqIndex> {
    let n = fields.len();
    let mut rv = vec![];
    if n == 0 {
        return rv;
    }
    let mut queue: Vec<SeqIndex> = vec![];
    // To avoid an if inside the loop, check if the first field is a sequence.
    if fields[0].is_sequence {
        queue.push(SeqIndex {
            depth: fields[0].depth,
            start: 0,
            end: 0,
        });
    }
    for i in 1..n {
        let prev = &fields[i - 1];
        let curr = &fields[i];
        let dd = curr.depth - prev.depth;
        if dd < 0 {
            // We must have jumped out of one or more sequences.
            let m = dd.abs();
            for _ in 0..m {
                rv.push(SeqIndex {
                    depth: queue.last().unwrap().depth,
                    start: queue.last().unwrap().start,
                    end: i,
                });
                queue.pop();
            }
        } else if dd > 0 {
            // We must have jumped in a sequence.
            if dd != 1 {
                panic!("Expected depth increases between to StructFields, not to exceed 1.");
            }
        }
        if curr.is_sequence {
            // Add the new sequence to the queue.
            queue.push(SeqIndex {
                depth: curr.depth,
                start: i,
                end: i,
            });
        }
    }
    // Any remaining indices on the queue should be moved into the return value.
    for qf in queue.iter().rev() {
        rv.push(SeqIndex {
            depth: qf.depth,
            start: qf.start,
            end: n,
        });
    }
    rv
}

/// Find the nested sequence indices of `index` in `indices`.
///
/// The function doesn't assume that indices are sorted.
///
/// # Arguments
/// * `indices` - slice of sequence indices
/// * `index` - sequence index
///
/// # Returns
/// Vector of nested sequences indices within the `index` sequence.
pub(crate) fn get_nested_sequence_indices<'a, 'b>(
    indices: &'a [SeqIndex],
    index: &'b SeqIndex,
) -> Vec<&'a SeqIndex> {
    let mut v = vec![];
    for i in indices {
        if i == index {
            continue;
        }
        if i.start > index.start && i.end <= index.end {
            v.push(i);
        }
    }
    v
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
        Ok(StructField {
            visibility: Visibility::Public,
            name: format!("{}_item", var_name),
            lifetime: None,
            reference: false,
            type_: sequence_type_from_keyword(var_name.as_str())?,
            depth,
            is_sequence: true,
        })
    } else {
        Ok(StructField {
            visibility: Visibility::Public,
            name: var_name,
            lifetime: None,
            reference: false,
            type_: obj_name.get(0).unwrap().to_string(),
            depth,
            is_sequence: false,
        })
    };
}

#[cfg(test)]
mod tests {
    use dicom_std_core::{
        DataDictionary, DataDictionaryEntry, IODLibrary, IodModuleType, ModuleAttribute, Tag,
        TagRange, VM, VR,
    };
    use log::debug;
    use std::path::PathBuf;

    use crate::rust::syntax::StructField;
    use crate::rust::{module_attribute_to_field, RustAstBuilder, SeqIndex, Visibility};
    use crate::{init_test_logger, read_data_dictionary, read_iod_library, test_resource_dir};

    #[test]
    fn sequence_item_type_from_keyword() {
        assert_eq!(
            super::sequence_item_type_from_keyword("AnatomicRegionModifierSequence")
                .unwrap()
                .as_str(),
            "AnatomicRegionModifierSequenceItem"
        );
        assert_eq!(
            super::sequence_item_type_from_keyword("Anatomic Region Modifier Sequence")
                .unwrap()
                .as_str(),
            "AnatomicRegionModifierSequenceItem"
        );
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
        dict.add(DataDictionaryEntry {
            tag: TagRange::from(Tag::new(0x0008, 0x2220)),
            name: ">Anatomic Region Modifier Sequence".to_string(),
            keyword: "AnatomicRegionModifierSequence".to_string(),
            vr: vec![VR::SQ],
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
        let field_seq = module_attribute_to_field(module_attributes.get(2).unwrap(), &dict);
        assert!(field_seq.is_ok());
        let field_seq = field_seq.unwrap();
        assert_eq!(field_seq.visibility, Visibility::Public);
        assert_eq!(
            field_seq.name.as_str(),
            "anatomic_region_modifier_sequence_item"
        );
        assert_eq!(
            field_seq.type_.as_str(),
            "Vec<AnatomicRegionModifierSequenceItem>"
        );
    }

    #[test]
    fn module_definition_to_struct() {
        let iod_library = read_iod_library();
        let data_dictionary = read_data_dictionary();
        print!("normative n={}", iod_library.normative.len());
    }

    #[test]
    fn get_nested_sequence_indices() {
        let index = SeqIndex {
            depth: 0,
            start: 1,
            end: 12,
        };
        let expected = vec![
            SeqIndex {
                depth: 1,
                start: 3,
                end: 6,
            },
            SeqIndex {
                depth: 1,
                start: 8,
                end: 12,
            },
        ];
        let mut indices = expected.clone();
        indices.insert(1, index.clone());
        let result = super::get_nested_sequence_indices(&indices, &index);
        assert_eq!(expected.len(), result.len());
        for index in &result {
            assert!(expected.contains(*index));
        }
        // assert_eq!(&expected, &result);
    }

    #[test]
    fn find_seq_indices_begin_end() {
        init_test_logger();
        let fields = vec![
            // 0
            StructField {
                visibility: Visibility::Public,
                name: "item".to_string(),
                lifetime: None,
                reference: false,
                type_: "".to_string(),
                depth: 0,
                is_sequence: false,
            },
            // 1
            StructField {
                visibility: Default::default(),
                name: "sequence start".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: true,
            },
            // 2
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 1,
                is_sequence: false,
            },
            // 3
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 1,
                is_sequence: false,
            },
            // 4
            StructField {
                visibility: Visibility::Public,
                name: "sequence start".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 1,
                is_sequence: true,
            },
            // 5
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 2,
                is_sequence: false,
            },
            // 6
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 2,
                is_sequence: false,
            },
            // 7
            StructField {
                visibility: Visibility::PublicCrate,
                name: "other item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: false,
            },
        ];
        let expected = vec![
            SeqIndex {
                depth: 0,
                start: 1,
                end: 7,
            },
            SeqIndex {
                depth: 1,
                start: 4,
                end: 7,
            },
        ];
        let v = super::find_seq_indices_begin_end(&fields);
        debug!("expected: {:?}\n", &expected);
        debug!("result: {:?}\n", &v);
        for item in &v {
            let nc = v.iter().filter(|itr| *itr == item).count();
            assert_eq!(nc, 1);
        }
        for item in &v {
            assert!(expected.contains(item));
        }
    }

    #[test]
    fn find_seq_indices_begin_end2() {
        init_test_logger();
        let fields = vec![
            StructField {
                visibility: Visibility::Public,
                name: "item".to_string(),
                lifetime: None,
                reference: false,
                type_: "".to_string(),
                depth: 0,
                is_sequence: false,
            },
            StructField {
                visibility: Default::default(),
                name: "sequence start".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: true,
            },
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 1,
                is_sequence: false,
            },
            StructField {
                visibility: Visibility::PublicCrate,
                name: "other item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: false,
            },
            StructField {
                visibility: Default::default(),
                name: "sequence start".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: true,
            },
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 1,
                is_sequence: false,
            },
            StructField {
                visibility: Visibility::PublicCrate,
                name: "other item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: false,
            },
        ];
        let expected = vec![
            SeqIndex {
                depth: 0,
                start: 1,
                end: 3,
            },
            SeqIndex {
                depth: 0,
                start: 4,
                end: 6,
            },
        ];
        let v = super::find_seq_indices_begin_end(&fields);
        for item in &v {
            let nc = v.iter().filter(|itr| *itr == item).count();
            assert_eq!(nc, 1);
        }
        for item in &v {
            assert!(expected.contains(item));
        }
    }

    #[test]
    fn find_seq_indices_begin_end3() {
        init_test_logger();
        let fields = vec![
            StructField {
                visibility: Default::default(),
                name: "sequence start".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
                is_sequence: true,
            },
            StructField {
                visibility: Visibility::Public,
                name: "sequence item".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 1,
                is_sequence: false,
            },
        ];
        let expected = vec![SeqIndex {
            depth: 0,
            start: 0,
            end: 2,
        }];
        let v = super::find_seq_indices_begin_end(&fields);
        for item in &v {
            let nc = v.iter().filter(|itr| *itr == item).count();
            assert_eq!(nc, 1);
        }
        for item in &v {
            assert!(expected.contains(item));
        }
    }
}
