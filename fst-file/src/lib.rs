use blocks::{parse_block, Block};
use error::{FstFileParseError, FstFileResult};
use nom::{combinator::eof, error::context, multi::many_till, Finish, Offset};
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize,
};

pub mod blocks;
pub mod data_types;
pub mod error;

#[derive(Clone)]
pub struct Blocks<'a> {
    start_of_input: &'a [u8],
    blocks: Vec<Block<'a>>,
}

impl Serialize for Blocks<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.blocks.len()))?;
        for blockinfo in self.iter() {
            seq.serialize_element(&blockinfo)?;
        }
        seq.end()
    }
}

pub struct BlockInfo<'a> {
    file_offset: usize,
    block: &'a Block<'a>,
}

impl Serialize for BlockInfo<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_struct("Block", 6)?;
        map.serialize_field("type", &self.block.block_type)?;
        map.serialize_field("block_offset", &self.get_block_start_offset())?;
        map.serialize_field("block_length", &self.get_block_length())?;
        map.serialize_field("data_offset", &self.get_data_start_offset())?;
        map.serialize_field("data_length", &self.get_data_length())?;
        map.serialize_field("block_end", &self.get_block_end_offset())?;
        map.end()
    }
}

impl BlockInfo<'_> {
    pub fn get_block_start_offset(&self) -> usize {
        self.file_offset
    }
    pub fn get_data_start_offset(&self) -> usize {
        self.file_offset + 9
    }
    pub fn get_block_end_offset(&self) -> usize {
        self.file_offset + self.block.len() - 1
    }
    pub fn get_block_length(&self) -> usize {
        self.block.len()
    }
    pub fn get_data_length(&self) -> usize {
        self.block.len() - 9
    }
    pub fn get_block(&self) -> &Block {
        self.block
    }
}

impl Blocks<'_> {
    pub fn get(&self, index: usize) -> Option<BlockInfo<'_>> {
        self.blocks.get(index).map(|block| BlockInfo {
            file_offset: self.start_of_input.offset(block.get_data_raw()) - 9,
            block,
        })
    }

    pub fn iter(&self) -> BlocksIter {
        BlocksIter {
            index: 0,
            blocks: self,
        }
    }
}

pub struct BlocksIter<'a> {
    index: usize,
    blocks: &'a Blocks<'a>,
}

impl<'a> Iterator for BlocksIter<'a> {
    type Item = BlockInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.blocks.get(self.index)?;
        self.index += 1;
        Some(out)
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
