use std::path::PathBuf;

use clap::{ArgGroup, Parser};
use log::{trace, LevelFilter};

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = "")]
#[clap(group(
            ArgGroup::new("pa")
                .required(true)
                .multiple(false)
            )
)]
struct Cli {
    /// Output directory where the standard is saved
    #[clap(
        short,
        long = "output-dir",
        value_parser,
        value_name = "directory",
        default_value = "dicom_standard"
    )]
    output_dir: String,

    /// DICOM standard version (current, 2022a, ...)
    #[clap(
        short = 'V',
        long = "version",
        value_parser,
        value_name = "dicom-standard-version",
        default_value = "current"
    )]
    version: String,

    /// DICOM standard part (3, 5, 6, ...)
    #[clap(
        group = "pa",
        short,
        long = "part",
        value_parser,
        value_name = "dicom-standard-part"
    )]
    parts: Option<Vec<u32>>,

    /// Download all the DICOM standard parts [1-22], except for part 9 and 13 [do not exist].
    #[clap(group = "pa", short, long = "all")]
    all: bool,

    /// Timeout in seconds to download one DICOM part
    #[clap(short, long = "timeout", value_parser, default_value = "500")]
    timeout: u64,

    /// Provide more information while the application runs.
    #[clap(short, long, action)]
    verbose: Option<bool>,

    /// Provide more debugging information while the application runs.
    #[clap(short, long, action)]
    debug: Option<bool>,
}

fn init_logger(verbose: bool, debug: bool) {
    let level;
    if verbose {
        level = LevelFilter::Info;
    } else if debug {
        level = LevelFilter::Debug;
    } else {
        level = LevelFilter::Warn;
    }
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(level)
        .try_init();
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    init_logger(
        cli.verbose.unwrap_or_default(),
        cli.debug.unwrap_or_default(),
    );

    // Determine which DICOM parts to download
    let mut parts = vec![];
    if cli.all {
        for i in 1..23u32 {
            if i != 9 && i != 13 {
                parts.push(i);
            }
        }
    } else if let Some(pts) = cli.parts {
        parts = pts;
    }

    let odir = cli.output_dir;
    let version = cli.version;
    let path = PathBuf::from(odir.as_str());
    let path = path.join(version.as_str());

    trace!("output path: {}", odir.as_str());
    trace!("version: {}", version.as_str());

    let result = dicom_std_fetch::dicom_standard_parts(path, version, parts, cli.timeout);
    let _ = result.expect("Failed to download all the required parts from the DICOM standard.");
    Ok(())
}
