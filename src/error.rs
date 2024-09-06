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
    #[error("{0}")]
    StyleError(#[from] StyleParserError),
    #[error("`{0}`")]
    Failed(String),
}

#[derive(Error, Debug)]
pub enum StyleParserError {
    #[error("Failed to parse `{0}` to value")]
    FailedToParseVal(String),
    #[error("`{0}` does not represent a ui rect")]
    FailedToParseRect(String),
    #[error("UnkownToken `{0}`")]
    UnkownToken(String),
    #[error("Failed to parse `{0}`")]
    FailedToParse(String),
    #[error("Failed to parse as color `{0}`")]
    FailedToParseColor(String),
}
