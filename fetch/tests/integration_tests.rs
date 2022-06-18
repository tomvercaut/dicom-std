use dicom_std_utils::temp_dir_fn;
use log::LevelFilter;

fn init_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Trace)
        .try_init();
}

#[test]
fn dowload_dicom_standard() {
    init_logger();
    let mut odir = temp_dir_fn(stdext::function_name!());
    odir = odir.join("dicom_standard");
    let xml_file = odir.join("current").join("part22").join("part22.xml");
    if xml_file.exists() {
        let r = std::fs::remove_file(&xml_file);
        assert!(
            r.is_ok(),
            "Failed to remove existing xml file: {:?}",
            xml_file
        );
    }
    let timeout = 300;
    let parts = vec![22];
    let result = dicom_std_fetch::dicom_standard_parts(odir, "current".to_string(), parts, timeout);
    assert!(result.is_ok(), "{}", result.err().unwrap());
    if xml_file.exists() {
        let r = std::fs::remove_file(&xml_file);
        assert!(
            r.is_ok(),
            "Failed to remove existing xml file: {:?}",
            xml_file
        );
    }
}
