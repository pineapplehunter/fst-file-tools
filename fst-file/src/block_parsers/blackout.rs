use nom::{
    bytes::complete::take,
    combinator::{eof, map, map_res},
    error::VerboseErrorKind,
    multi::many_m_n,
    Finish,
};
use serde::Serialize;
use thiserror::Error;

use crate::{
    data_types::VarInt,
    error::{BlockParseError, ParseResult, PositionError},
    FstParsable,
};

use super::Block;

#[derive(Debug, Error)]
pub enum BlackoutParseError {
    #[error("parse error: {0}")]
    ParseError(#[from] PositionError<VerboseErrorKind>),
}

/// Blackout Block
#[derive(Debug, Clone)]
pub struct BlackoutBlock(Block);

impl BlackoutBlock {
    pub fn from_block(block: Block) -> Self {
        Self(block)
    }

    pub fn get_content(&self) -> Result<BlackoutContent, BlackoutParseError> {
        let data = self.0.get_data_raw();
        Ok(BlackoutContent::parse(data)
            .finish()
            .map(|(_, v)| v)
            .map_err(|e| PositionError::from_verbose_parse_error(e, data))?)
    }
}

/// Record of blackout
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlackoutRecord {
    active: bool,
    time_delta: VarInt,
}

/// Content of blackout block
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlackoutContent {
    records: Vec<BlackoutRecord>,
}

impl FstParsable for BlackoutRecord {
    fn parse(input: &[u8]) -> ParseResult<BlackoutRecord> {
        let (input, active) = map(take(1u8), |b: &[u8]| b[0] == 1)(input)?;
        let (input, time_delta) = VarInt::parse(input)?;
        Ok((input, BlackoutRecord { active, time_delta }))
    }
}

impl FstParsable for BlackoutContent {
    fn parse(input: &[u8]) -> ParseResult<BlackoutContent> {
        let (input, count) = map_res(VarInt::parse, |v| {
            usize::try_from(v).map_err(|_e| (input, BlockParseError::LengthTooLargeForMachine))
        })(input)?;
        let (input, records) = many_m_n(count, count, BlackoutRecord::parse)(input)?;

        let data = BlackoutContent { records };

        let (_input, _) = eof(input)?;
        Ok((input, data))
    }
}
