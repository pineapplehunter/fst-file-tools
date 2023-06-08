use std::cell::OnceCell;

use nom::{
    bytes::complete::take,
    combinator::{eof, map, map_res},
    multi::many_m_n,
    Finish,
};
use serde::Serialize;

use crate::{
    data_types::VarInt,
    error::{BlockParseError, FstFileParseError, FstFileResult},
    FstParsable,
};

use super::Block;

type BlackoutContentResult<'a> = Result<BlackoutContent, FstFileParseError<&'a [u8]>>;

/// Blackout Block
#[derive(Debug, Clone)]
pub struct BlackoutBlock<'a> {
    block: &'a Block<'a>,
    content: OnceCell<BlackoutContentResult<'a>>,
}

impl<'a> BlackoutBlock<'a> {
    pub fn from_block(block: &'a Block) -> Self {
        Self {
            block,
            content: OnceCell::new(),
        }
    }

    fn get_content_cache(&self) -> &BlackoutContentResult {
        self.content.get_or_init(|| {
            BlackoutContent::parse(self.block.data)
                .finish()
                .map(|(_, v)| v)
        })
    }

    pub fn get_content(&self) -> &BlackoutContentResult {
        self.get_content_cache()
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
    fn parse(input: &[u8]) -> FstFileResult<'_, BlackoutRecord> {
        let (input, active) = map(take(1u8), |b: &[u8]| b[0] == 1)(input)?;
        let (input, time_delta) = VarInt::parse(input)?;
        Ok((input, BlackoutRecord { active, time_delta }))
    }
}

impl FstParsable for BlackoutContent {
    fn parse(input: &[u8]) -> FstFileResult<'_, BlackoutContent> {
        let (input, count) = map_res(VarInt::parse, |v| {
            usize::try_from(v).map_err(|_e| (input, BlockParseError::LengthTooLargeForMachine))
        })(input)?;
        let (input, records) = many_m_n(count, count, BlackoutRecord::parse)(input)?;

        let data = BlackoutContent { records };

        let (_input, _) = eof(input)?;
        Ok((input, data))
    }
}
