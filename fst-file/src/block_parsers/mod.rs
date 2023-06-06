use std::{fmt, io::Read};

use nom::{combinator::map, multi::length_data, number::complete::be_u64};
use tracing::warn;

use crate::{data_types::BlockType, error::FstFileResult, FstParsable};

/// Blackout Block
pub mod blackout;
/// Geometry Block
pub mod geometry;
/// Header Block
pub mod header;
/// Hierarchy Block
pub mod hierarchy;
// pub mod value_change_data;

/// Abstract block struct that only holds the type of block ([BlockType]) and location of data (&[[u8]])
#[derive(Clone)]
pub struct Block<'a> {
    pub block_type: BlockType,
    data: &'a [u8],
}

impl Block<'_> {
    fn extract_data_gz(&self) -> Vec<u8> {
        let uncompressed_size =
            usize::try_from(u64::from_be_bytes(self.data[..8].try_into().unwrap())).unwrap();
        let mut decompressor = flate2::read::GzDecoder::new(&self.data[8..]);
        let mut data = Vec::new();
        decompressor.read_to_end(&mut data).unwrap();
        if data.len() != uncompressed_size {
            warn!(
                data_len = data.len(),
                uncompressed_size = uncompressed_size,
                "extracted data size did not match specified.",
            );
        }
        data
    }

    fn extract_data_lz4(&self) -> Vec<u8> {
        let uncompressed_size =
            usize::try_from(u64::from_be_bytes(self.data[..8].try_into().unwrap())).unwrap();
        let data = lz4_flex::block::decompress(&self.data[8..], uncompressed_size).unwrap();
        if data.len() != uncompressed_size {
            warn!(
                data_len = data.len(),
                uncompressed_size = uncompressed_size,
                "extracted data size did not match specified.",
            );
        }
        data
    }

    fn extract_data_lz4_twice(&self) -> Vec<u8> {
        let uncompressed_size =
            usize::try_from(u64::from_be_bytes(self.data[..8].try_into().unwrap())).unwrap();
        let uncompressed_once_size =
            usize::try_from(u64::from_be_bytes(self.data[8..16].try_into().unwrap())).unwrap();
        let data = lz4_flex::block::decompress(&self.data[8..], uncompressed_once_size).unwrap();
        if data.len() != uncompressed_once_size {
            warn!(
                data_len = data.len(),
                uncompressed_size = uncompressed_once_size,
                "first extracted data size did not match specified.",
            );
        }
        let data2 = lz4_flex::block::decompress(&data, uncompressed_size).unwrap();
        if data2.len() != uncompressed_size {
            warn!(
                data_len = data2.len(),
                uncompressed_size = uncompressed_size,
                "second extracted data size did not match specified.",
            );
        }
        data2
    }

    /// Extracts data from block.
    /// If the block content is compressed, it will be uncompressed in this function.
    pub fn extract_data(&self) -> Vec<u8> {
        match self.block_type {
            BlockType::Header => self.data.to_vec(),
            BlockType::ValueChangeData => self.data.to_vec(),
            BlockType::Blackout => self.data.to_vec(),
            BlockType::Geometry => self.data.to_vec(),
            BlockType::HierarchyGz => self.extract_data_gz(),
            BlockType::ValueChangeDataAlias => self.data.to_vec(),
            BlockType::HierarchyLz4 => self.extract_data_lz4(),
            BlockType::HierarchyLz4Duo => self.extract_data_lz4_twice(),
            BlockType::ValueChangeDataAlias2 => self.data.to_vec(),
            BlockType::GZippedWrapper => self.extract_data_gz(),
            BlockType::Skip => self.data.to_vec(),
        }
    }

    /// Get the raw underlying data bytes.
    /// Useful when calculating offsets from another place in the file.
    pub fn get_data_raw(&self) -> &[u8] {
        self.data
    }

    /// Size of Block
    pub fn size(&self) -> usize {
        self.data.len() + 9
    }

    fn parse_block_length(input: &[u8]) -> FstFileResult<'_, u64> {
        map(be_u64, |v| v - 8)(input)
    }

    pub(crate) fn parse_block(input: &[u8]) -> FstFileResult<'_, Block> {
        let (input, block_type) = BlockType::parse(input)?;
        let (input, data) = length_data(Block::parse_block_length)(input)?;
        let block = Block { block_type, data };
        Ok((input, block))
    }
}

impl fmt::Debug for Block<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("block_type", &self.block_type)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
