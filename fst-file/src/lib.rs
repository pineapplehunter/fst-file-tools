use std::fmt;

use blocks::{parse_block, Block};
use error::{FstFileParseError, FstFileResult};
use nom::{combinator::eof, error::context, multi::many_till, Finish, Offset};

pub mod blocks;
pub mod data_types;
pub mod error;

#[derive(Clone)]
pub struct Blocks<'a> {
    start_of_input: &'a [u8],
    blocks: Vec<Block<'a>>,
}

impl Blocks<'_> {
    pub fn get(&self, index: usize) -> Option<&Block<'_>> {
        self.blocks.get(index)
    }
}

impl fmt::Display for Blocks<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, block) in self.blocks.iter().enumerate() {
            let block_len = block.len();
            let data_len = block_len - 9;
            let start_of_data = self.start_of_input.offset(block.get_data_raw());
            let start_of_block = start_of_data - 9;
            let end_of_block = start_of_data + data_len;
            writeln!(f, "Block#{idx} {}", block.block_type)?;
            writeln!(f, "    block_len:      {block_len}")?;
            writeln!(f, "    start of block: {start_of_block}",)?;
            writeln!(f, "    data_len:       {data_len}")?;
            writeln!(f, "    start of data:  {start_of_data}",)?;
            writeln!(f, "    end:            {end_of_block}")?;
        }
        Ok(())
    }
}

pub fn parse_blocks(input: &[u8]) -> FstFileResult<'_, Blocks> {
    let input_original = input;
    let (input, (blocks, _)) = many_till(context("parse block", parse_block), eof)(input)?;
    let blocks = Blocks {
        start_of_input: input_original,
        blocks,
    };
    Ok((input, blocks))
}

pub fn parse_file(input: &[u8]) -> Result<Blocks, FstFileParseError<&[u8]>> {
    parse_blocks(input).finish().map(|(_, blocks)| blocks)
}
