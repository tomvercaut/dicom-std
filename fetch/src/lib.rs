use markup5ever::{local_name, LocalName, Namespace, QualName};
use scraper::{Html, Selector};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::time::Duration;

use log::debug;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use url::Url;

/// Errors that can occur while downloading the DICOM standard.
#[derive(thiserror::Error, Debug)]
pub enum DownloadError {
    #[error(transparent)]
    UrlParseError {
        #[from]
        source: url::ParseError,
    },
    #[error(transparent)]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },
    #[error("URL request failed with status code: {0}")]
    RequestStatusCode(StatusCode),
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Didn't download DICOM standard part(s). Another thread or process already locked the download.")]
    WasLocked,
    #[error("Failed to remove lockfile [{0:?}].")]
    FailedToRemoveLock(PathBuf),
    #[error(transparent)]
    ParseIntError {
        #[from]
        source: std::num::ParseIntError,
    },
    #[error("Unable to get the DICOM part size in bytes from the web")]
    UnknownDicomPartSizeOnline,
    #[error("Unknown error detected while downloading a DICOM XML part")]
    UnknownDownloadError,
}

/// Timestamp and file size info.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStamp {
    // timestamp of the file
    pub date_time: chrono::NaiveDateTime,
    // byte size of a file
    pub size: usize,
}

/// Download part of the DICOM standard.
///
/// The part and version that are specified will be downloaded.
///
/// # Arguments
/// * `odir` - output directory path
/// * `version` - DICOM version
/// * `url` - URL to the DICOM standard docbook
/// * `part` - DICOM part
/// * `timeout` - timeout in seconds
///
fn dicom_standard_part(
    odir: PathBuf,
    version: &str,
    url: Url,
    part: u32,
    timeout: u64,
) -> Result<(), DownloadError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()?;

    let spart = if part < 10 {
        format!("part0{}", part)
    } else {
        format!("part{}", part)
    };
    let filename = format!("{}.xml", spart.as_str());
    let url_ext = format!("{}/{}", spart.as_str(), filename.as_str());

    let part_path = odir;
    let part_path = part_path.join(spart.as_str());
    if !part_path.exists() {
        debug!("creating directory: {:?}", &part_path);
        std::fs::create_dir_all(&part_path)?;
    }
    let part_path = part_path.join(filename.as_str());
    let mut attempts = 0;
    let max_attempts = 3;
    let file_stamp = dicom_part_byte_size_online(version, part, timeout)?;
    let part_url = url.join(url_ext.as_str())?;
    while attempts < max_attempts {
        debug!("Checking if the {} was downloaded previously", url.as_str());
        if (&part_path).exists() {
            let meta = std::fs::metadata(&part_path)?;
            if meta.is_file() {
                if meta.len() == file_stamp.size as u64 {
                    debug!("{} was already downloaded.", url.as_str());
                    return Ok(());
                } else {
                    std::fs::remove_file(&part_path)?;
                }
            }
        }

        // only download the file if it doesn't already exist
        debug!(
            "Downloading {:?} from {} [attempt: {}]",
            &part_path, &part_url, attempts
        );
        let resp = client.get(part_url.clone()).send()?;
        let code = resp.status();
        if code != StatusCode::OK {
            if attempts < max_attempts {
                attempts += 1;
                continue;
            }
            return Err(DownloadError::RequestStatusCode(code));
        }
        let body = resp.text()?;
        let mut file = std::fs::File::create(&part_path)?;
        file.write_all(body.as_bytes())?;
        debug!("Download of {:?} was succesful", &part_path);
        return Ok(());
    }
    Err(DownloadError::UnknownDownloadError)
}

/// Get the URL to the source of a specified DICOM version.
///
/// # Arguments
///
/// * `version`: DICOM version (current, 2021d, ...)
///
/// returns: String
fn dicom_docbook_url(version: &str) -> String {
    format!(
        "https://dicom.nema.org/medical/dicom/{}/source/docbook/",
        version
    )
}

/// Get the URL to the DICOM part for a specified DICOM version.
///
/// On this page the filestamp and URL to the XML file are found.
///
/// # Arguments
///
/// * `version`: DICOM version (current, 2021d, ...)
/// * `part`: DICOM part
///
/// returns: String
fn dicom_docbook_url_part(version: &str, part: u32) -> String {
    let p = if part < 10 {
        format!("0{}", part)
    } else {
        format!("{}", part)
    };
    format!("{}part{}/", dicom_docbook_url(version), p)
}

/// Get the URL to a DICOM XML part for a specified DICOM version.
///
/// # Arguments
///
/// * `version`: DICOM version (current, 2021d, ...)
/// * `part`: DICOM part
///
/// returns: String
fn dicom_docbook_url_part_xml(version: &str, part: u32) -> String {
    let p = if part < 10 {
        format!("0{}", part)
    } else {
        format!("{}", part)
    };
    format!("{}part{}/part{}.xml", dicom_docbook_url(version), p, p)
}

/// Download one or more DICOM XML parts and store them in an output directory.
///
/// # Arguments
///
/// * `odir`: output directory
/// * `version`: DICOM version
/// * `parts`: a list of DICOM part numbers
/// * `timeout`: time out in seconds
pub fn dicom_standard_parts<P: AsRef<Path> + Clone>(
    odir: P,
    version: String,
    parts: Vec<u32>,
    timeout: u64,
) -> Result<(), DownloadError> {
    let url = Url::from_str(dicom_docbook_url(version.as_str()).as_str())?;

    let odir = odir.as_ref();
    if !odir.is_dir() {
        std::fs::create_dir_all(odir)?;
    }
    let odir_path_buf = odir.to_path_buf();
    let (sender, receiver) = channel();
    for part in parts.clone() {
        let tarc_odir = odir_path_buf.clone();
        let turl = url.clone();
        let tpart = part;
        let ttimeout = timeout;
        let sender = sender.clone();
        let tversion = version.clone();
        debug!(
            "{:?} <- {} part {}, timeout: {}",
            &tarc_odir, &turl, tpart, ttimeout
        );
        std::thread::spawn(move || {
            sender
                .send(dicom_standard_part(
                    tarc_odir,
                    tversion.as_str(),
                    turl,
                    tpart,
                    ttimeout,
                ))
                .unwrap();
        });
    }

    debug!("waiting for the downloads to finish ...");
    for _ in parts {
        let r = receiver.recv().unwrap();
        match r {
            Ok(_) => {
                debug!("one download finished");
            }
            Err(e) => {
                debug!("one download failed");
                return Err(e);
            }
        }
    }
    debug!("finished downloading the DICOM parts");
    Ok(())
}

/// Retrieve the date, time and size stats from a DICOM xml part from the DICOM standard website.
///
/// # Arguments
///
/// * `version`: DICOM version
/// * `part`: part number
/// * `timeout`: timeout in seconds for the GET request
pub fn dicom_part_byte_size_online(
    version: &str,
    part: u32,
    timeout: u64,
) -> Result<FileStamp, DownloadError> {
    // build the url
    let url = dicom_docbook_url_part(version, part);
    let timeout = Duration::from_secs(timeout);
    let client = Client::builder().timeout(timeout).build()?;
    let resp = client.get(Url::from_str(url.as_str())?).send()?;
    let code = resp.status();
    if code != StatusCode::OK {
        return Err(DownloadError::RequestStatusCode(code));
    }
    // interpret the returned body
    let body = resp.text()?;
    let doc = Html::parse_document(body.as_str());
    let selector_pre = Selector::parse("pre").unwrap();
    let qn_href = QualName::new(None, Namespace::from(""), LocalName::from("href"));
    let url_prefix = Url::parse("https://dicom.nema.org")?;
    let url_xml_part = dicom_docbook_url_part_xml(version, part);
    for pre in doc.select(&selector_pre) {
        let mut e = pre.last_child();
        while e.is_some() {
            if let Some(last_child_ref) = e {
                let node = last_child_ref.value();
                if !node.is_element() {
                    e = last_child_ref.prev_sibling();
                    continue;
                }
                let element = node.as_element().unwrap();
                if element.name.local != local_name!("a") {
                    e = last_child_ref.prev_sibling();
                    continue;
                }
                if let Some(href) = element.attrs.get(&qn_href) {
                    let shref = url_prefix.join(&*href.to_string())?.to_string();
                    if shref == url_xml_part {
                        if let Some(prev_sibling) = last_child_ref.prev_sibling() {
                            let prev_node = prev_sibling.value();
                            if prev_node.is_text() {
                                let text = prev_node.as_text().unwrap().to_string();
                                let mut v = vec![];
                                for ts in text.split(' ') {
                                    let t = ts.trim();
                                    if !t.is_empty() {
                                        v.push(t);
                                    }
                                }
                                if v.len() == 4 {
                                    let sdate = *v.get(0).unwrap();
                                    let stime = *v.get(1).unwrap();
                                    let colon = stime.find(':');
                                    if colon.is_none() {
                                        continue;
                                    }
                                    let vsdate = sdate.split('/').collect::<Vec<&str>>();
                                    let month = vsdate.get(0).unwrap().parse::<u32>()?;
                                    let day = vsdate.get(1).unwrap().parse::<u32>()?;
                                    let year = vsdate.get(2).unwrap().parse::<i32>()?;
                                    let colon = colon.unwrap();
                                    let hour = (&stime[0..colon]).parse::<u32>()?;
                                    let minute = (&stime[colon + 1..]).parse::<u32>()?;
                                    let sam_pm = *v.get(2).unwrap();
                                    let date_time = if sam_pm == "AM" {
                                        chrono::NaiveDate::from_ymd(year, month, day)
                                            .and_hms(hour, minute, 0)
                                    } else if sam_pm == "PM" {
                                        chrono::NaiveDate::from_ymd(year, month, day).and_hms(
                                            hour + 12,
                                            minute,
                                            0,
                                        )
                                    } else {
                                        panic!("invalid time format: {}", stime);
                                    };
                                    let ss = *v.get(3).unwrap();
                                    if ss.trim() == "<dir>" {
                                        continue;
                                    }
                                    let size = ss.parse::<usize>()?;
                                    return Ok(FileStamp { date_time, size });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Err(DownloadError::UnknownDicomPartSizeOnline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_dicom_part_byte_size_online() {
        let mut data = HashMap::new();
        let mut parts_2021a = HashMap::new();
        parts_2021a.insert(
            1,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 1, 30).and_hms(15, 53, 0),
                size: 102866,
            },
        );
        parts_2021a.insert(
            2,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 1, 30).and_hms(15, 55, 0),
                size: 3306023,
            },
        );
        parts_2021a.insert(
            3,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 1, 30).and_hms(16, 12, 0),
                size: 22276054,
            },
        );
        parts_2021a.insert(
            4,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 1, 30).and_hms(16, 14, 0),
                size: 4141511,
            },
        );
        data.insert("2021d", parts_2021a);

        let mut parts_2021d = HashMap::new();
        parts_2021d.insert(
            1,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 9, 10).and_hms(15, 1, 0),
                size: 103652_usize,
            },
        );
        parts_2021d.insert(
            2,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 9, 10).and_hms(15, 2, 0),
                size: 3308291,
            },
        );
        parts_2021d.insert(
            3,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 9, 10).and_hms(15, 18, 0),
                size: 22569606,
            },
        );
        parts_2021d.insert(
            4,
            FileStamp {
                date_time: chrono::NaiveDate::from_ymd(2021, 9, 10).and_hms(15, 22, 0),
                size: 4146401,
            },
        );
        data.insert("2021d", parts_2021d);
        let timeout = 100;
        for (version, parts) in &data {
            for (part, exp_file_stamp) in parts {
                let size = dicom_part_byte_size_online(version, *part, timeout);
                assert!(size.is_ok(), "{}", size.err().unwrap());
                let file_stamp = size.unwrap();
                assert_eq!(file_stamp, *exp_file_stamp);
            }
        }
    }
}
