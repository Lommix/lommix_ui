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

pub struct VerboseHtmlError<'a> {
    input: &'a [u8],
}

impl<'a> VerboseHtmlError<'a> {
    pub fn Format(&'a self) -> String {
        "".into()
    }
}

impl<'a> nom::error::ParseError<&'a [u8]> for VerboseHtmlError<'a> {
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        todo!()
    }

    fn append(input: &'a [u8], kind: nom::error::ErrorKind, other: Self) -> Self {
        todo!()
    }
}

impl<'a> nom::error::ContextError<&'a [u8]> for VerboseHtmlError<'a> {
    fn add_context(_input: &'a [u8], _ctx: &'static str, other: Self) -> Self {
        other
    }
}

pub trait HtmlError<'a>:
    nom::error::ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>
{
}
