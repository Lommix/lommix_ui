use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Default)]
pub struct Span {
    pub line: i64,
    pub col: i64,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}]", self.line, self.col)
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Encountered unkown token `{0}`")]
    UnknownToken(String),
    #[error("Encountered Unclosed Tag `{0}`")]
    Unclosed(String),
    #[error("`{0}`")]
    Failed(String),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for ParseError {
    fn from(value: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        ParseError::UnknownToken(value.to_string())
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum AttributeError {
    #[error("Failed to parse `{0}` to value")]
    #[diagnostic(help("lol you failed"))]
    FailedToParseVal(String),

    #[error("`{0}` does not represent a ui rect")]
    #[diagnostic(help("lol you failed"))]
    FailedToParseRect(String),

    #[error("UnkownToken `{0}`")]
    #[diagnostic(help("lol you failed"))]
    UnkownToken(String),

    #[error("Failed to parse `{0}`")]
    #[diagnostic(help("lol you failed"))]
    FailedToParse(String),

    #[error("Failed to parse as color `{0}`")]
    #[diagnostic(help("lol you failed"))]
    FailedToParseColor(String),
}
