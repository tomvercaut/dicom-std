pub use error::*;
pub use helper::*;
pub mod dom;

pub(crate) mod query;

mod error;
mod helper;

pub mod data_dictionary;
mod defs;
pub mod iod;

pub use defs::*;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use log::{debug, LevelFilter};

    pub(crate) fn test_resource_dir() -> PathBuf {
        let s = dicom_std_test_data::path_dicom_standard();
        debug!("test path to the DICOM standard: {}", s);
        PathBuf::from(s)
    }

    pub(crate) fn test_resource_sample_dir() -> PathBuf {
        let d = env!("CARGO_MANIFEST_DIR");
        let mut p = PathBuf::from(d);
        p = p.join("resources");
        p.join("samples")
    }

    pub(crate) fn init_logger() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();
    }

    pub(crate) fn path_dicom_part(part_id: i32) -> PathBuf {
        let part: String;
        if part_id < 10 {
            part = format!("part0{}", part_id.to_string());
        } else {
            part = format!("part{}", part_id.to_string());
        }
        let mut p = test_resource_dir();
        p = p.join(part.as_str());
        p.join(format!("{}.xml", part.as_str()))
    }

    #[test]
    fn check_dicom_parts_exist() {
        let v = vec![3, 5, 6];
        for i in v {
            let p = path_dicom_part(i);
            assert!(p.exists());
        }
    }
}
