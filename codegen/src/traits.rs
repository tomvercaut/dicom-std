use crate::LanguageTranslateError;
use dicom_std_core::VR;

pub trait LanguageTranslator {
    fn get_object_name(s: &str) -> std::result::Result<String, LanguageTranslateError>;
    fn get_function_name(s: &str) -> std::result::Result<String, LanguageTranslateError>;
    fn get_variable_name(s: &str) -> std::result::Result<String, LanguageTranslateError>;
    fn get_object_types_by_vr(vr: VR) -> Vec<String>;
}
