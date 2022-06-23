use dicom_std_core::VR;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{replace_char_at, LanguageTranslateError, LanguageTranslator};

#[derive(Debug, Copy, Clone)]
pub struct RustLanguageTranslator {}

impl std::default::Default for RustLanguageTranslator {
    fn default() -> Self {
        Self {}
    }
}

impl LanguageTranslator for RustLanguageTranslator {
    fn get_object_name(s: &str) -> std::result::Result<String, LanguageTranslateError> {
        let mut t = s.trim().to_string();
        let n = t.len();
        if n == 0 {
            return Err(LanguageTranslateError::EmptyObjectName);
        }
        let mut next_captical = false;
        for i in 0..n {
            let c: &mut str = &mut t[i..i + 1];
            let mut c: char = c.chars().nth(0).unwrap();
            if next_captical && c.is_ascii_alphanumeric() {
                c.make_ascii_uppercase();
                t = replace_char_at(t.as_str(), i, c);
                next_captical = false;
            }
            if c.is_whitespace() || c == '_' {
                next_captical = true;
            }
        }
        let t = t.replace(
            |c: char| !c.is_alphanumeric() && !c.is_whitespace() && c != '_',
            "",
        );
        let t = t.replace(|c: char| c.is_whitespace(), "_");
        lazy_static! {
            static ref RE: Regex = Regex::new("_+").unwrap();
        }
        let t = RE.replace_all(t.as_ref(), "");
        // strip characters at the front that are not [a-zA-Z]
        let u: &str = t.trim_start_matches(|c: char| !c.is_ascii_alphabetic());
        // strip characters at the end that are not [a-zA-Z0-9]
        let v: &str = u.trim_end_matches(|c: char| !c.is_ascii_alphanumeric());
        let mut v = v.to_string();
        v.as_mut_str()
            .get_mut(0..1)
            .map(|c: &mut str| c.make_ascii_uppercase());
        Ok(v.to_string())
    }

    fn get_function_name(s: &str) -> std::result::Result<String, LanguageTranslateError> {
        let mut t = s.trim().to_string();
        {
            let mut u = "".to_string();
            let n = t.len();
            for i in 0..n {
                let curr = t.chars().nth(i).unwrap();
                if i > 0 {
                    let prev = t.chars().nth(i - 1).unwrap();
                    if prev.is_lowercase() && curr.is_uppercase() {
                        u.push('_');
                    }
                }
                u.push(curr);
            }
            t = u;
        }
        let n = t.len();
        if n == 0 {
            return Err(LanguageTranslateError::EmptyFunctionName);
        }
        let t = t.replace(
            |c: char| !c.is_alphanumeric() && !c.is_whitespace() && c != '_',
            "",
        );
        let t = t.replace(|c: char| c.is_whitespace(), "_");
        lazy_static! {
            static ref RE: Regex = Regex::new("_+").unwrap();
        }
        let t = RE.replace_all(t.as_ref(), "_");
        // strip characters at the front that are not [a-zA-Z]
        let u: &str = t.trim_start_matches(|c: char| !c.is_ascii_alphabetic());
        // strip characters at the end that are not [a-zA-Z0-9]
        let v: &str = u.trim_end_matches(|c: char| !c.is_ascii_alphanumeric());
        Ok(v.to_ascii_lowercase())
    }

    fn get_variable_name(s: &str) -> std::result::Result<String, LanguageTranslateError> {
        match Self::get_function_name(s) {
            Ok(s) => Ok(s),
            Err(e) => match e {
                LanguageTranslateError::EmptyFunctionName => {
                    Err(LanguageTranslateError::EmptyVariableName)
                }
                _ => Err(e),
            },
        }
    }

    fn get_object_types_by_vr(vr: VR) -> Vec<String> {
        match vr {
            VR::AE
            | VR::AS
            | VR::CS
            | VR::LO
            | VR::LT
            | VR::PN
            | VR::SH
            | VR::ST
            | VR::UC
            | VR::UI
            | VR::UR
            | VR::UT => {
                vec!["String".to_string()]
            }
            VR::AT => {
                vec!["dicom_std_core::model::Tag".to_string()]
            }
            VR::DA => {
                vec!["String".to_string(), "chono::naive::NaiveData".to_string()]
            }
            VR::DS => {
                vec!["String".to_string(), "f64".to_string()]
            }
            VR::DT => {
                vec![
                    "String".to_string(),
                    "chrono::naive::NaiveDateTime".to_string(),
                ]
            }
            VR::FL => {
                vec!["String".to_string(), "f32".to_string()]
            }
            VR::FD => {
                vec!["String".to_string(), "f64".to_string()]
            }
            VR::IS => {
                vec!["String".to_string(), "i32".to_string()]
            }
            VR::OB => {
                vec!["Vec<u8>".to_string()]
            }
            VR::OD => {
                vec!["Vec<u8>".to_string()]
            }
            VR::OF => {
                vec!["Vec<u8>".to_string()]
            }
            VR::OL => {
                vec!["Vec<u8>".to_string()]
            }
            VR::OV => {
                vec!["Vec<u8>".to_string()]
            }
            VR::OW => {
                vec!["Vec<u8>".to_string()]
            }
            VR::SL => {
                vec!["i32".to_string()]
            }
            VR::SQ => {
                vec!["Vec".to_string()]
            }
            VR::SS => {
                vec!["i16".to_string()]
            }
            VR::SV => {
                vec!["i64".to_string()]
            }
            VR::TM => {
                vec!["String".to_string(), "chrono::naive::NaiveTime".to_string()]
            }
            VR::UL => {
                vec!["u32".to_string()]
            }
            VR::UN => {
                vec!["Vec<u8>".to_string()]
            }
            VR::US => {
                vec!["u16".to_string()]
            }
            VR::UV => {
                vec!["u64".to_string()]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use crate::rust::RustLanguageTranslator;
    use crate::LanguageTranslator;

    #[allow(dead_code)]
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn to_object_name() {
        let arr = [
            "name___abc",
            "name   abc",
            "name _ abc",
            "name's",
            ";:\"{}[]'+=-_~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=- _~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=-A _~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=- A_~`$^!@#$&()?/><.,",
            ";:\"{}[]1'+=- A_~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=-a_~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=-a_b~`$^!@#$&()?/><.,",
        ];
        let exp = [
            "NameAbc", "NameAbc", "NameAbc", "Names", "", "", "A", "A", "A", "A", "AB",
        ];
        assert_eq!(arr.len(), exp.len());
        for (a, e) in arr.iter().zip(exp.iter()) {
            let t = RustLanguageTranslator::get_object_name(*a);
            let t = t.unwrap_or("".to_string());
            assert_eq!(t.as_str(), *e);
        }
    }

    #[test]
    fn to_function_name() {
        let arr = [
            "name___abc",
            "name   abc",
            "name _ abc",
            "name's",
            ";:\"{}[]'+=-_~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=- _~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=-A _~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=- A_~`$^!@#$&()?/><.,",
            ";:\"{}[]1'+=- A_~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=-a_~`$^!@#$&()?/><.,",
            ";:\"{}[]'+=-a_b~`$^!@#$&()?/><.,",
            "PatientName",
        ];
        let exp = [
            "name_abc",
            "name_abc",
            "name_abc",
            "names",
            "",
            "",
            "a",
            "a",
            "a",
            "a",
            "a_b",
            "patient_name",
        ];
        assert_eq!(arr.len(), exp.len());
        for (a, e) in arr.iter().zip(exp.iter()) {
            let t = RustLanguageTranslator::get_function_name(*a);
            let t = t.unwrap_or("".to_string());
            assert_eq!(t.as_str(), *e);
        }
    }
}
