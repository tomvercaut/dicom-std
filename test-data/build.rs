use anyhow::bail;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("resource_path.rs");
    let dcm_std_path = Path::new(&out_dir).join("dicom_standard").join("current");
    let mut fmt = r#"
pub fn path_dicom_standard() -> &'static str {
    "#
    .to_string();
    fmt.push_str(&*format!("{:?}", &dcm_std_path));
    fmt.push_str(
        r#"
    }"#,
    );
    std::fs::write(&out_path, fmt).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={:?}", &out_path);
    if !dcm_std_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&dcm_std_path) {
            let msg = format!(
                "Failed to download DICOM test resources to: {:?} [{}]",
                &dcm_std_path, e
            );
            println!("cargo:warning={}", msg);
            bail!("{}", msg);
        }
    }
    let parts = vec![3, 5, 6];
    // Download speeds from the online repository are slow,
    // so it is better to give it sufficient time to download the parts.
    let timeout = 1500;
    if let Err(e) =
        dicom_std_fetch::dicom_standard_parts(&dcm_std_path, "current".to_string(), parts, timeout)
    {
        let msg = format!(
            "Failed to download DICOM test resources to: {:?} [{:?}]",
            &dcm_std_path, e
        );
        println!("cargo:warning={}", msg);
        bail!("{}", msg);
    }
    Ok(())
}
