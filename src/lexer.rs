// The parser and lexer are currently
// merged. This makes certain things really
// hard and costly like finding comments.
//
// lex -> syntaxtree -> lint -> compile
// use thiserror::Error;
//
// pub(crate) struct Lexer<'a> {
//     source: &'a [u8],
//     col: u16,
//     line: u16,
// }
//
// impl<'a> Iterator for Lexer<'a> {
//     type Item = Result<Token<'a>, TokenError<'a>>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }
//
// #[derive(Debug)]
// pub struct Span {
//     start: usize,
//     end: usize,
//     line: u16,
//     col: u16,
// }
//
// impl std::fmt::Display for Span {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[{}]:[{}]", self.line, self.col)
//     }
// }
//
// #[derive(Error, Debug)]
// pub enum TokenError<'a> {
//     #[error("unkown token {0} at {1}")]
//     Unkown(&'a str, Span),
// }
//
// pub(crate) enum Token<'a> {
//     Ident(Ident),
//     Literal(&'a str),
//     StartTag,
//     EndTag,
//     Equals,
//     StartComment,
//     EndComment,
//     StartPunc,
//     EndPunc,
// }
//
// pub(crate) enum Ident {
//     OnSpawn,
//     OnEnter,
//     OnPress,
//     OnExit,
//     Source,
//     Target,
//     Id,
// }
