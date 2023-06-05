use nom::{
    error::{ContextError, ErrorKind, FromExternalError, ParseError},
    IResult,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum BlockParseError {
    #[error("block type unknown {0}")]
    BlockTypeUnknown(u8),
    #[error("block has wrong length")]
    BlockWrongLength,
    #[error("the block length was too large to fit in usize")]
    LengthTooLargeForMachine,
    #[error("string from c could not be parsed {0:?}")]
    CStringParseError(Vec<u8>),
    #[error("the file type was wrong")]
    WrongFileType,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum FstFileParseErrorInner {
    #[error("block parsing error > {0}")]
    BlockParseError(#[from] BlockParseError),
    #[error("nom error of kind {0:?}")]
    NomError(ErrorKind),
    #[error("context {0}")]
    Context(&'static str),
}

impl From<ErrorKind> for FstFileParseErrorInner {
    fn from(value: ErrorKind) -> Self {
        Self::NomError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub struct FstFileParseError<I> {
    pub errors: Vec<(I, FstFileParseErrorInner)>,
}

impl<I> ParseError<I> for FstFileParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            errors: vec![(input, FstFileParseErrorInner::from(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other
            .errors
            .push((input, FstFileParseErrorInner::from(kind)));
        other
    }
}

impl<I> From<(I, FstFileParseErrorInner)> for FstFileParseError<I> {
    fn from(value: (I, FstFileParseErrorInner)) -> Self {
        Self {
            errors: vec![value],
        }
    }
}

impl<I, E: Into<FstFileParseErrorInner>> FromExternalError<I, E> for FstFileParseError<I> {
    fn from_external_error(input: I, _kind: ErrorKind, e: E) -> Self {
        Self {
            errors: vec![(input, e.into())],
        }
    }
}

impl<I> ContextError<I> for FstFileParseError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other
            .errors
            .push((input, FstFileParseErrorInner::Context(ctx)));
        other
    }
}

pub type FstFileResult<'a, T> = IResult<&'a [u8], T, FstFileParseError<&'a [u8]>>;
