use serde::{ser::SerializeStruct, Serialize};

use crate::block_parsers::Block;

pub struct BlockInfo {
    block: Block,
    start_position: usize,
    size: usize,
}

impl BlockInfo {
    pub fn get_block_start_offset(&self) -> usize {
        self.start_position
    }
    pub fn get_data_start_offset(&self) -> usize {
        self.start_position + 9
    }
    pub fn get_block_end_offset(&self) -> usize {
        self.start_position + self.size
    }
    pub fn get_block_length(&self) -> usize {
        self.size
    }
    pub fn get_data_length(&self) -> usize {
        self.block.get_data_raw().len()
    }
    pub fn get_block(&self) -> &Block {
        &self.block
    }

    pub fn take_block(self) -> Block {
        self.block
    }

    pub fn from_offset_and_block(start_position: usize, size: usize, block: Block) -> Self {
        Self {
            start_position,
            block,
            size,
        }
    }
}

impl Serialize for BlockInfo {
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
