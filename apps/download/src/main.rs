use std::path::PathBuf;
use std::str::FromStr;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use log::{trace, LevelFilter};

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
    let matches = App::new(env!("CARGO_BIN_NAME"))
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("output-dir")
                .help("Output directory where the standard is saved.")
                .required(false)
                .short("o")
                .long("output-dir")
                .takes_value(true)
                .empty_values(false)
                .number_of_values(1)
                .value_name("directory")
                .default_value("dicom_standard"),
        )
        .arg(
            Arg::with_name("version")
                .help("Specify the version of the standard you want to download.")
                .required(false)
                .short("v")
                .long("version")
                .takes_value(true)
                .empty_values(false)
                .number_of_values(1)
                .default_value("current"),
        )
        .arg(
            Arg::with_name("part")
                .help("DICOM part number")
                .long("part")
                .short("p")
                .takes_value(true)
                .multiple(true)
                .empty_values(false)
                .required_unless("all"),
        )
        .arg(
            Arg::with_name("timeout")
                .help("timeout in seconds to download one DICOM part")
                .long("timeout")
                .short("t")
                .takes_value(true)
                .multiple(false)
                .empty_values(false)
                .number_of_values(1)
                .default_value("500"),
        )
        .arg(
            Arg::with_name("all")
                .help(
                    "Download all the DICOM parts [1-22], except for part 9 and 13 [do not exist].",
                )
                .long("all")
                .short("a")
                .takes_value(false)
                .multiple(false)
                .required_unless("part"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .help("Print more info while the application runs.")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .help("Print more debugging info while the application runs.")
                .required(false)
                .takes_value(false),
        )
        .get_matches();
    let verbose = matches.is_present("verbose");
    let debug = matches.is_present("debug");
    init_logger(verbose, debug);
    let timeout = matches
        .value_of("timeout")
        .unwrap()
        .parse::<u64>()
        .expect(&*format!(
            "failed to convert timeout [{}] to an u64",
            matches.value_of("timeout").unwrap_or_default()
        ));

    // Determine which DICOM parts to download
    let mut parts = vec![];
    if matches.is_present("all") {
        for i in 1..23u32 {
            if i != 9 && i != 13 {
                parts.push(i);
            }
        }
    } else if matches.is_present("part") {
        let values = matches.values_of("part").unwrap();
        for value in values {
            let part = u32::from_str(value);
            if let Err(e) = part {
                panic!("Failed to parse DICOM part number [{}]: {}", value, e);
            }
            parts.push(part.unwrap());
        }
    }

    let odir = matches.value_of("output-dir").unwrap().to_string();
    let version = matches.value_of("version").unwrap().to_string();
    let path = PathBuf::from(odir.as_str());
    let path = path.join(version.as_str());

    trace!("output path: {}", odir.as_str());
    trace!("version: {}", version.as_str());

    let result = dicom_std_fetch::dicom_standard_parts(path, version, parts, timeout);
    let _ = result.expect("Failed to download all the required parts from the DICOM standard.");
    Ok(())
}
