use std::{cell::OnceCell, ffi::CStr};

use nom::{
    bytes::complete::take,
    combinator::eof,
    number::complete::{be_i64, be_u64, le_f64},
    sequence::tuple,
    Finish,
};
use serde::Serialize;
use tracing::debug_span;

use crate::{
    data_types::{FileType, TimeScale},
    error::{FstFileParseError, FstFileResult},
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
            HeaderBlockContent::parse(self.block.data)
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

impl FstParsable for HeaderBlockContent {
    fn parse(input: &[u8]) -> FstFileResult<'_, HeaderBlockContent> {
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
            be_u64,
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

fn c_str_with_size<'a>(size: usize) -> impl Fn(&'a [u8]) -> FstFileResult<'a, String> {
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
