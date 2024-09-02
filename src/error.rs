use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("failed to parse ron file")]
    FailedToParse(String),
    #[error("something broke")]
    Unkown(String),
}

#[derive(Debug, Default)]
pub struct Span {
    pub line: i64,
    pub col: i64,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to parse ui file")]
    FailedToParse { span: Span, msg: String },
    #[error("unknown token")]
    UnknownToken(String),
    #[error("Fucked up")]
    Failed(String),
    #[error("Fucked up")]
    Unclosed(String),
}

#[derive(Error, Debug)]
pub enum StyleParserError {
    #[error("Fucked up")]
    FailedToParseVal(String),
    #[error("Fucked up")]
    UnkownToken(String),
    #[error("It's 1, 2 or 4 attributes to describe a uirect")]
    UnexpectedRectPattern(String),
    #[error("is this a number?")]
    FailedToParseNumber,
}
