use std::{cell::OnceCell, ffi::CStr};

use nom::{
    bytes::complete::take,
    combinator::{eof, map, map_res},
    number::complete::{be_i64, be_i8, be_u64, be_u8, le_f64},
    Finish,
};
use num_traits::FromPrimitive;
use serde::Serialize;
use tracing::debug_span;

use crate::{
    data_types::{FileType, TimeScale},
    error::{BlockParseError, FstFileParseError, FstFileResult},
};

use super::Block;

#[derive(Debug, Clone, Serialize)]
pub struct HeaderBlockContent {
    pub start_time: u64,
    pub end_time: u64,
    pub real_endianness: f64,
    pub writer_memory_use: u64,
    pub num_scopes: u64,
    pub num_hiearchy_vars: u64,
    pub num_vars: u64,
    pub num_vc_blocks: u64,
    pub timescale: TimeScale,
    pub writer: String,
    pub date: String,
    pub filetype: FileType,
    pub timezero: i64,
}

type ContentResult<'a> = Result<HeaderBlockContent, FstFileParseError<&'a [u8]>>;

#[derive(Debug, Clone)]
pub struct HeaderBlock<'a> {
    block: &'a Block<'a>,
    content: OnceCell<ContentResult<'a>>,
}

impl<'a> HeaderBlock<'a> {
    fn get_content_cached(&'a self) -> &'a ContentResult<'a> {
        self.content.get_or_init(|| {
            let _span = debug_span!("caching header content").entered();
            parse_header_content(self.block.data)
                .finish()
                .map(|(_, content)| content)
        })
    }

    pub fn get_content(&'a self) -> &'a ContentResult<'a> {
        self.get_content_cached()
    }

    pub fn from_block(block: &'a Block<'a>) -> Self {
        Self {
            block,
            content: OnceCell::new(),
        }
    }
}

pub fn parse_header_content(input: &[u8]) -> FstFileResult<'_, HeaderBlockContent> {
    let (input, start_time) = be_u64(input)?;
    let (input, end_time) = be_u64(input)?;
    let (input, real_endianness) = le_f64(input)?;
    assert!((real_endianness - std::f64::consts::E).abs() < std::f64::EPSILON);
    let (input, writer_memory_use) = be_u64(input)?;
    let (input, num_scopes) = be_u64(input)?;
    let (input, num_hiearchy_vars) = be_u64(input)?;
    let (input, num_vars) = be_u64(input)?;
    let (input, num_vc_blocks) = be_u64(input)?;
    let (input, timescale) = map(be_i8, TimeScale)(input)?;
    let (input, writer) = map_res(take(128u32), |b: &[u8]| {
        CStr::from_bytes_until_nul(b)
            .map(|s| s.to_string_lossy().to_string())
            .map_err(|_e| (input, BlockParseError::CStringParseError(b.to_vec())))
    })(input)?;
    let (input, date) = map_res(take(26u32), |b: &[u8]| {
        CStr::from_bytes_until_nul(b)
            .map(|s| s.to_string_lossy().to_string())
            .map_err(|_e| (input, BlockParseError::CStringParseError(b.to_vec())))
    })(input)?;
    let (input, _reserved) = take(93u32)(input)?;
    let (input, filetype) = map_res(be_u8, |i| {
        FileType::from_u8(i).ok_or((input, BlockParseError::WrongFileType))
    })(input)?;
    let (input, timezero) = be_i64(input)?;
    let data = HeaderBlockContent {
        start_time,
        end_time,
        real_endianness,
        writer_memory_use,
        num_scopes,
        num_hiearchy_vars,
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
