use serde::{ser::SerializeStruct, Serialize};

use crate::block_parsers::Block;

pub struct BlockInfo<'a> {
    file_offset: usize,
    block: &'a Block<'a>,
}

impl<'a> BlockInfo<'a> {
    pub fn get_block_start_offset(&self) -> usize {
        self.file_offset
    }
    pub fn get_data_start_offset(&self) -> usize {
        self.file_offset + 9
    }
    pub fn get_block_end_offset(&self) -> usize {
        self.file_offset + self.block.size() - 1
    }
    pub fn get_block_length(&self) -> usize {
        self.block.size()
    }
    pub fn get_data_length(&self) -> usize {
        self.block.size() - 9
    }
    pub fn get_block(&'a self) -> &'a Block {
        self.block
    }

    pub fn from_offset_and_block(file_offset: usize, block: &'a Block<'a>) -> Self {
        Self { file_offset, block }
    }
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
