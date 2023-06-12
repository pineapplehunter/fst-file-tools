use std::ffi::CStr;

use nom::{
    bytes::complete::take,
    combinator::eof,
    error::VerboseErrorKind,
    number::complete::{be_i64, be_u64, le_f64},
    sequence::tuple,
    Finish,
};
use serde::Serialize;
use thiserror::Error;
use tracing::debug_span;

use crate::{
    as_usize,
    data_types::{FileType, TimeScale},
    error::{ParseResult, PositionError},
    FstParsable,
};

use super::Block;

#[derive(Debug, Clone, Serialize)]
pub struct HeaderBlockContent {
    pub start_time: u64,
    pub end_time: u64,
    pub real_endianness: f64,
    pub writer_memory_use: u64,
    pub num_scopes: u64,
    pub num_hierarchy_vars: u64,
    pub num_vars: usize,
    pub num_vc_blocks: u64,
    pub timescale: TimeScale,
    pub writer: String,
    pub date: String,
    pub filetype: FileType,
    pub timezero: i64,
}

#[derive(Debug, Error)]
pub enum HeaderParseError {
    #[error("parse error: {0}")]
    ParseError(#[from] PositionError<VerboseErrorKind>),
}

#[derive(Debug, Clone)]
pub struct HeaderBlock(Block);

impl HeaderBlock {
    pub fn from_block(block: Block) -> Self {
        Self(block)
    }

    pub fn get_content(&self) -> Result<HeaderBlockContent, HeaderParseError> {
        let _span = debug_span!("get header content").entered();
        let data = self.0.get_data_raw();
        Ok(HeaderBlockContent::parse(data)
            .finish()
            .map(|(_, content)| content)
            .map_err(|e| PositionError::from_verbose_parse_error(e, data))?)
    }
}

impl FstParsable for HeaderBlockContent {
    fn parse(input: &[u8]) -> ParseResult<HeaderBlockContent> {
        let (
            input,
            (
                start_time,
                end_time,
                real_endianness,
                writer_memory_use,
                num_scopes,
                num_hierarchy_vars,
                num_vars,
                num_vc_blocks,
                timescale,
                writer,
                date,
                _,
                filetype,
                timezero,
            ),
        ) = tuple((
            be_u64,
            be_u64,
            le_f64,
            be_u64,
            be_u64,
            be_u64,
            as_usize(be_u64),
            be_u64,
            TimeScale::parse,
            c_str_with_size(128),
            c_str_with_size(26),
            take(93u8),
            FileType::parse,
            be_i64,
        ))(input)?;
        assert!((real_endianness - std::f64::consts::E).abs() < std::f64::EPSILON);
        let data = HeaderBlockContent {
            start_time,
            end_time,
            real_endianness,
            writer_memory_use,
            num_scopes,
            num_hierarchy_vars,
            num_vars,
            num_vc_blocks,
            timescale,
            writer,
            date,
            filetype,
            timezero,
        };
        let (input, _) = eof(input)?;
        Ok((input, data))
    }
}

fn c_str_with_size<'a>(size: usize) -> impl Fn(&'a [u8]) -> ParseResult<'a, String> {
    move |input| {
        let (input, data) = take(size)(input)?;
        let mut v = data.to_vec();
        v.push(0);
        let s = CStr::from_bytes_until_nul(&v)
            .unwrap()
            .to_string_lossy()
            .to_string();
        Ok((input, s))
    }
}
