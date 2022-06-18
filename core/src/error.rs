use crate::Tag;

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Tag format [{0}] is invalid, expected tag format (gggg,eeee)")]
    InvalidFormat(String),
    #[error(transparent)]
    ParseIntError {
        #[from]
        source: std::num::ParseIntError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum TagRangeError {
    #[error("TagRange format [{0}] is invalid, expected tag format (gggg,eeee)")]
    InvalidFormat(String),
    #[error(transparent)]
    ParseIntErrorr {
        #[from]
        source: std::num::ParseIntError,
    },
    #[error(transparent)]
    TagError {
        #[from]
        source: TagError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum VMError {
    #[error("VM format [{0}] is invalid")]
    InvalidFormat(String),
    #[error(transparent)]
    ParseIntError {
        #[from]
        source: std::num::ParseIntError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum VRError {
    #[error("VR format [{0}] is invalid")]
    InvalidFormat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DataDictionaryError {
    #[error("Data dictionary doesn't have any entries.")]
    NoItems,
    #[error("Tag [{0}] was not found in the data dictionary")]
    TagNotFound(Tag),
}

#[derive(Debug, thiserror::Error)]
pub enum IodModuleTypeError {
    #[error("IOD module type format [{0}] is invalid, expected IOD module type format: \"1\",\"1C\",\"2\",\"2C\",\"3\"")]
    InvalidFormat(String),
}
