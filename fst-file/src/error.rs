use nom::{
    error::{ContextError, ErrorKind, FromExternalError, ParseError},
    IResult,
};
use thiserror::Error;

use crate::data_types::{VarIntParseError, VarIntParseErrorKind};

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
    #[error("var int parse error > {0}")]
    VarIntParseError(#[from] VarIntParseErrorKind),
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

impl<I, E> From<(I, E)> for FstFileParseError<I>
where
    E: Into<FstFileParseErrorInner>,
{
    fn from(value: (I, E)) -> Self {
        Self {
            errors: vec![(value.0, value.1.into())],
        }
    }
}

impl<I, E> FromExternalError<I, E> for FstFileParseError<I>
where
    E: Into<FstFileParseError<I>>,
{
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        let mut e = e.into();
        e.errors.push((input, kind.into()));
        e
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
