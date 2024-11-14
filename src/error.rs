use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to read bytes to end `{0}`")]
    FailedToRead(String),

    #[error("provided content is not utf8")]
    Utf8Error,

    #[error("{0}")]
    Nom(String),

    #[error("Failed with incomplete parse")]
    Incomplete,
}
