#[cfg(test)]
use std::path::PathBuf;

pub use error::*;
use helper::*;
pub use traits::*;

mod error;
mod helper;
mod rust;
mod traits;

#[cfg(test)]
pub fn test_resource_dir() -> PathBuf {
    let d = env!("CARGO_MANIFEST_DIR");
    let p = PathBuf::from(d);
    p.join("resources")
}

#[cfg(test)]
fn read_iod_library() -> IODLibrary {
    let part03 = PathBuf::from(dicom_std_test_data::path_dicom_standard())
        .join("part03")
        .join("part03.xml");
    let iod_library = dicom_std_xml_parser::iod::library::build_from_path(&part03);
    if let Err(e) = iod_library {
        panic!("Failed to build IODLibrary: {:?}", e);
    }
    iod_library.unwrap()
}

#[cfg(test)]
fn read_data_dictionary() -> DataDictionary {
    let part06 = PathBuf::from(dicom_std_test_data::path_dicom_standard())
        .join("part06")
        .join("part06.xml");
    let data_dict = dicom_std_xml_parser::data_dictionary::build(&part06);
    if let Err(e) = data_dict {
        panic!("Failed to build data dictionary: {:?}", e);
    }
    data_dict.unwrap()
}
