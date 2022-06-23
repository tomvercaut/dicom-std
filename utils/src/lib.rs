use std::path::PathBuf;

/// Get a path to a temporary directory.
///
/// # Arguments
/// * `fn_name` name of the function for which the directory needs to be generated
///             (can be auto generated using stdext::function_name!())
pub fn temp_dir_fn(fn_name: &str) -> PathBuf {
    let mut d = std::env::temp_dir();
    d = d.join("dicom_std");
    for t in fn_name.split("::") {
        d = d.join(t);
    }
    d
}

/// Test if a character is a whitespace, a carriage return or a newline.
///
/// # Arguments
///
/// * `c` - character
#[inline]
pub fn is_char_whitespace_or_return(c: char) -> bool {
    c.is_whitespace() || c == '\r' || c == '\n'
}

#[cfg(test)]
mod tests {
    use log::{trace, LevelFilter};

    use super::*;

    pub(crate) fn init_logger() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();
    }

    #[test]
    fn temp_dir() {
        init_logger();
        let d = temp_dir_fn(stdext::function_name!());
        trace!("temp dir: {:?}", d);
        assert!(d.ends_with("temp_dir"));
    }
}
