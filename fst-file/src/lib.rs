use std::sync::Arc;

use block_parsers::{
    blackout::BlackoutBlock, hierarchy::HierarchyBlock, value_change_data::ValueChangeDataBlock,
    Block,
};
use data_types::{BlockInfo, BlockType};
use error::{ParseResult, PositionError};
use nom::{
    combinator::{complete, eof, map, map_res},
    error::{context, ErrorKind, ParseError, VerboseError, VerboseErrorKind},
    multi::many_till,
    Finish, IResult, Offset,
};
use tracing::{debug, debug_span};

use crate::block_parsers::{geometry::GeometryBlock, header::HeaderBlock};

/// Block data and their parsers
pub mod block_parsers;
pub mod data_types;
pub mod error;

/// Parses blocks
fn parse_blocks<'a>(input: &'a [u8]) -> IResult<&[u8], Vec<BlockInfo>, VerboseError<&[u8]>> {
    complete(|input: &'a [u8]| {
        let input_original = input;
        let (input, (blocks, _)) = many_till(
            context(
                "parse block",
                map(Block::parse_block_with_position, |((s, e), b)| {
                    BlockInfo::from_offset_and_block(input_original.offset(s), s.offset(e) - 1, b)
                }),
            ),
            eof,
        )(input)?;
        Ok((input, blocks))
    })(input)
}

#[derive(Debug)]
pub struct FstFileContent {
    pub header: Option<HeaderBlock>,
    pub hierarchy: Option<HierarchyBlock>,
    pub blackout: Option<BlackoutBlock>,
    pub geometry: Option<GeometryBlock>,
    pub value_change_data: Arc<[ValueChangeDataBlock]>,
}

/// Parse the whole content of the fst file
pub fn parse_raw_block_information(
    input: &[u8],
) -> Result<Vec<BlockInfo>, PositionError<VerboseErrorKind>> {
    parse_blocks(input)
        .finish()
        .map(|(_, blocks)| blocks)
        .map_err(|e| PositionError::from_verbose_parse_error(e, input))
}

/// Parse the whole content of the fst file
pub fn parse(input: &[u8]) -> Result<FstFileContent, PositionError<VerboseErrorKind>> {
    let _span = debug_span!("parse content");
    parse_blocks(input)
        .finish()
        .map_err(|e| PositionError::from_verbose_parse_error(e, input))
        .map(|(_, blocks)| {
            // let mut header_block = None;
            let mut hierarchy = None;
            let mut blackout = None;
            let mut header = None;
            let mut geometry = None;
            let mut value_change_data = Vec::new();

            for (i, block) in blocks.into_iter().enumerate() {
                let block = block.take_block();
                match block.block_type {
                    BlockType::HierarchyGz
                    | BlockType::HierarchyLz4
                    | BlockType::HierarchyLz4Duo => {
                        debug!("using hierarchy block from #{}", i);
                        hierarchy = Some(HierarchyBlock::from_block(block))
                    }
                    BlockType::Blackout => {
                        debug!("using blackout block from #{}", i);
                        blackout = Some(BlackoutBlock::from_block(block))
                    }
                    BlockType::Skip => {}
                    BlockType::Header => {
                        debug!("using header block from #{}", i);
                        header = Some(HeaderBlock::from_block(block))
                    }
                    BlockType::Geometry => {
                        debug!("using geometry block from #{}", i);
                        geometry = Some(GeometryBlock::from_block(block))
                    }
                    BlockType::ValueChangeData
                    | BlockType::ValueChangeDataAlias
                    | BlockType::ValueChangeDataAlias2 => {
                        debug!("using value change data block from #{}", i);
                        value_change_data.push(ValueChangeDataBlock::from_block(block));
                    }
                    _ => {}
                }
            }

            FstFileContent {
                hierarchy,
                blackout,
                header,
                geometry,
                value_change_data: value_change_data.into(),
            }
        })
}

/// Parsable types
pub(crate) trait FstParsable: Sized {
    /// parse data from &[[u8]] and give [Self]
    fn parse(input: &[u8]) -> ParseResult<'_, Self>;
}

pub(crate) fn as_usize<'a, V, F>(f: F) -> impl Fn(&'a [u8]) -> ParseResult<'a, usize>
where
    V: TryInto<usize>,
    F: Fn(&'a [u8]) -> ParseResult<'a, V>,
{
    move |input| {
        context(
            "as usize",
            map_res(&f, |v| {
                v.try_into()
                    .map_err(|_| VerboseError::from_error_kind(input, ErrorKind::Digit))
            }),
        )(input)
    }
}

pub(crate) fn convert_type<'a, V, F, U>(f: F) -> impl Fn(&'a [u8]) -> ParseResult<'a, U>
where
    V: TryInto<U>,
    F: Fn(&'a [u8]) -> ParseResult<'a, V>,
{
    move |input| {
        context(
            "convert_type",
            map_res(&f, |v| {
                v.try_into()
                    .map_err(|_| VerboseError::from_error_kind(input, ErrorKind::Digit))
            }),
        )(input)
    }
}
