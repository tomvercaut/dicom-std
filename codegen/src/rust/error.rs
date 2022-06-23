use dicom_std_core::{Tag, TagRange};

use crate::LanguageTranslateError;

#[derive(thiserror::Error, Debug)]
pub enum RustAstError {
    #[error("Normative table ID [{0}] not found.")]
    NormativeTableNotFound(String),
    #[error(transparent)]
    TranslateError {
        #[from]
        source: LanguageTranslateError,
    },
    #[error("IOD module definition table [id={0}] not found")]
    IODModuleDefinitionTableNotFound(String),
    #[error("module definition tag is undefined [=0x0000,0x0000]")]
    ModuleDefinitionTagEmpty,
    #[error("module definition tag [{0}] was not found in data dictionary")]
    TagNotFoundDataDict(Tag),
    #[error("module attribute include isn't supported: {0}")]
    ModuleAttributeIncludeNotSupported(String),
    #[error("data dictionairy entry [{0}] doesn't have a VR")]
    DictionaryEntryHasNoVR(TagRange),
}
