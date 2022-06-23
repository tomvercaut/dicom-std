use crate::rust::RustLanguageTranslator;
use crate::{LanguageTranslateError, LanguageTranslator};
use dicom_std_core::DicomStandard;

#[derive(thiserror::Error, Debug)]
pub enum RustGeneratorError {
    #[error(transparent)]
    TranslateError {
        #[from]
        source: LanguageTranslateError,
    },
}

#[derive(Debug, Clone, Default)]
pub struct RustGenerator {
    ie_traits: String,
}

// impl TryFrom<&DicomStandard> for RustGenerator {
//     type Error = RustGeneratorError;
//
//     fn try_from(dicom_std: &DicomStandard) -> Result<Self, Self::Error> {
//         let mut gen: Self = Default::default();
//         for (module_name, table_ids) in &dicom_std.iod_lib.module_tables_ids {
//            let obj_name = RustLanguageTranslator::get_object_name(module_name.as_str())?;
//             let mut struct_blcok = format!("pub trait {} {{\n", obj_name);
//
//             struct_blcok.push_str("}}\n\n");
//             gen.ie_traits.push_str(struct_blcok.as_str());
//         }
//
//         Ok(gen)
//     }
// }

impl RustGenerator {}
