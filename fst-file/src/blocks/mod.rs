pub mod blackout;
pub mod header;
pub mod hierarchy;

use header::HeaderBlock;
use nom::{
    bytes::complete::take,
    combinator::{map, map_res},
    error::context,
    number::complete::be_u64,
};

use crate::{
    data_types::{parse_block_type, BlockType},
    error::{BlockParseError, FstFileResult},
};

use self::{
    blackout::{parse_blackout_block, BlackoutBlock}, header::parse_header_block,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    HeaderBlock(HeaderBlock),
    BlackoutBlock(BlackoutBlock),
    AnonBlock(BlockType, usize),
}

pub fn parse_block<'a>(input: &'a [u8]) -> FstFileResult<'a, Block> {
    let (input, block_type) = parse_block_type(input)?;
    let (input, length) = map_res(be_u64, |s| {
        usize::try_from(s).map_err(|_e| BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let (input, data) = take(length - 8)(input)?;
    let (_input, block) = match block_type {
        BlockType::Header => context(
            "in header block",
            map(parse_header_block, Block::HeaderBlock),
        )(data)?,
        BlockType::Blackout => context(
            "in blackout block",
            map(parse_blackout_block, Block::BlackoutBlock),
        )(data)?,
        // BlockType::ValueChangeData => todo!(),
        // BlockType::Geometry => todo!(),
        // BlockType::Hierarchy => todo!(),
        // BlockType::ValueChangeDataAlias => todo!(),
        // BlockType::HierarchyLz4 => todo!(),
        // BlockType::HierarchyLz4Duo => todo!(),
        // BlockType::ValueChangeDataAlias2 => todo!(),
        // BlockType::GZippedWrapper => todo!(),
        // BlockType::Skip => todo!(),
        _ => (data, Block::AnonBlock(block_type, length)),
    };
    Ok((input, block))
}
