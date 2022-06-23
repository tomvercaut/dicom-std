use crate::rust::RustAstError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Lifetime has invalid string format: {}", {0})]
    LifetimeParseError(String),
    #[error("Error while creating a new lifetime: {}", {0})]
    LifetimeNew(String),
    #[error("Tranlator error: {0}")]
    Translator(String),
    #[error("Table ID [{0}] not found.")]
    TableNotFound(String),
    #[error(transparent)]
    RustAstError {
        #[from]
        source: RustAstError,
    },
    #[error(transparent)]
    LanguageTranslateError {
        #[from]
        source: LanguageTranslateError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum LanguageTranslateError {
    #[error("Object name is empty.")]
    EmptyObjectName,
    #[error("Function name is empty.")]
    EmptyFunctionName,
    #[error("Function name is empty.")]
    EmptyVariableName,
}

pub type Result<T> = std::result::Result<T, crate::Error>;
