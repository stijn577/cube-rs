use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum CubeParseError {
    #[error("The filed {0} could not be found!")]
    FileNotFound(PathBuf),
    #[error("Failed to execute cargo command: {0}")]
    CargoFailed(String),
    #[error("Failed to find entry '{0}' in section '{1}' in .mxproject file.")]
    EntryNotFound(String, String),
    #[error("Failed to parse entry '{0}' correctly")]
    EntryParse(String),
    #[error("Failed to create build.rs file")]
    BuildRsCreate,
    #[error("Failed to create wrapper.h file")]
    WrapperHCreate,
}
