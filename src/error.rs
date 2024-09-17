use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to read bytes to end `{0}`")]
    FailedToRead(String),

    #[error("provided content is not utf8")]
    Utf8Error,

    #[error("Failed to parse {0}")]
    Nom(nom::error::Error<String>),

    #[error("Failed with incomplete parse")]
    Incomplete,
}

impl<'a> From<nom::Err<nom::error::Error<&'a [u8]>>> for ParseError {
    fn from(err: nom::Err<nom::error::Error<&'a [u8]>>) -> Self {
        match err.map_input(|i| String::from_utf8_lossy(i).to_string()) {
            nom::Err::Incomplete(needed) => {
                dbg!(needed);
                ParseError::Incomplete
            }
            nom::Err::Error(err) => ParseError::Nom(err),
            nom::Err::Failure(err) => ParseError::Nom(err),
        }
    }
}
