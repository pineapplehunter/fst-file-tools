use std::fmt;

use nom::{
    error::{
        ContextError, ErrorKind, FromExternalError, ParseError, VerboseError, VerboseErrorKind,
    },
    IResult, Offset,
};
use thiserror::Error;

// use crate::data_types::VarIntParseErrorKind;

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
pub enum FstFileParseErrorKind {
    #[error("block parsing error > {0}")]
    BlockParseError(#[from] BlockParseError),
    #[error("nom error of kind {0:?}")]
    NomError(ErrorKind),
    // #[error("var int parse error > {0}")]
    // VarIntParseError(#[from] VarIntParseErrorKind),
    // #[error("hierarchy parse error > {0}")]
    // HierarchyParseError(#[from] HierarchyParseErrorKind),
    #[error("context {0}")]
    Context(&'static str),
}

impl From<ErrorKind> for FstFileParseErrorKind {
    fn from(value: ErrorKind) -> Self {
        Self::NomError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub struct FstFileParseError<I> {
    pub errors: Vec<(I, FstFileParseErrorKind)>,
}

impl<I> ParseError<I> for FstFileParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            errors: vec![(input, FstFileParseErrorKind::from(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other
            .errors
            .push((input, FstFileParseErrorKind::from(kind)));
        other
    }
}

impl<I, E> From<(I, E)> for FstFileParseError<I>
where
    E: Into<FstFileParseErrorKind>,
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
            .push((input, FstFileParseErrorKind::Context(ctx)));
        other
    }
}

pub type FstFileResult<'a, T, I = &'a [u8]> = IResult<I, T, FstFileParseError<I>>;

#[derive(Debug, Clone, Error)]
#[error("error while parsing with position: {errors:?}")]

pub struct PositionError<E: fmt::Debug> {
    errors: Vec<(usize, E)>,
}

impl PositionError<VerboseErrorKind> {
    pub fn from_verbose_parse_error<I: Offset>(error: VerboseError<I>, original_input: I) -> Self {
        PositionError {
            errors: error
                .errors
                .into_iter()
                .map(|(i, e)| (original_input.offset(&i), e))
                .collect(),
        }
    }
}

impl PositionError<FstFileParseErrorKind> {
    pub fn from_fst_parse_error<I: Offset>(error: FstFileParseError<I>, original_input: I) -> Self {
        PositionError {
            errors: error
                .errors
                .into_iter()
                .map(|(i, e)| (original_input.offset(&i), e))
                .collect(),
        }
    }
}

pub type ParseResult<'a, T, I = [u8]> = IResult<&'a I, T, VerboseError<&'a I>>;
