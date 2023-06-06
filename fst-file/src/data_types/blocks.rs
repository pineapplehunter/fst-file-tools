use nom::Offset;
use serde::{ser::SerializeSeq, Serialize};

use crate::{
    block_parsers::{
        blackout::BlackoutBlock, geometry::GeometryBlock, header::HeaderBlock,
        hierarchy::HierarchyBlock, Block,
    },
    data_types::BlockType,
};

use super::BlockInfo;

#[derive(Clone)]
pub struct Blocks<'a> {
    pub(crate) start_of_input: &'a [u8],
    pub(crate) blocks: Vec<Block<'a>>,
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

impl Blocks<'_> {
    pub fn get(&self, index: usize) -> Option<BlockInfo<'_>> {
        self.blocks.get(index).map(|block| {
            BlockInfo::from_offset_and_block(
                self.start_of_input.offset(block.get_data_raw()) - 9,
                block,
            )
        })
    }

    pub fn iter(&self) -> BlocksIter {
        BlocksIter {
            index: 0,
            blocks: self,
        }
    }

    pub fn get_header_block(&self) -> Option<HeaderBlock> {
        self.blocks
            .iter()
            .find(|b| b.block_type == BlockType::Header)
            .map(HeaderBlock::from_block)
    }

    pub fn get_hierarchy_block(&self) -> Option<HierarchyBlock> {
        self.blocks
            .iter()
            .find(|b| {
                matches!(
                    b.block_type,
                    BlockType::HierarchyGz | BlockType::HierarchyLz4 | BlockType::HierarchyLz4Duo
                )
            })
            .map(HierarchyBlock::from_block)
    }

    pub fn get_geometry_block(&self) -> Option<GeometryBlock> {
        self.blocks
            .iter()
            .find(|b| b.block_type == BlockType::Geometry)
            .map(GeometryBlock::from_block)
    }

    pub fn get_blackout_block(&self) -> Option<BlackoutBlock> {
        self.blocks
            .iter()
            .find(|b| b.block_type == BlockType::Blackout)
            .map(|b| BlackoutBlock::from_block(b))
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
