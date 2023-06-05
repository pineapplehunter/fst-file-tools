use blocks::{parse_block, Block};
use error::FstFileResult;
use nom::{combinator::eof, error::context, multi::many_till};

pub mod blocks;
pub mod data_types;
pub mod error;

pub fn parse_file(input: &[u8]) -> FstFileResult<'_, Vec<Block>> {
    let (input, (data, _)) = many_till(context("parse block", parse_block), eof)(input)?;
    Ok((input, data))
}
