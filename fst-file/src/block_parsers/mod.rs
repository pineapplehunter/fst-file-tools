use std::{fmt, io::Read};

use nom::{
    combinator::map_res,
    error::{context, ErrorKind, VerboseError},
    multi::length_data,
    number::complete::be_u64,
    IResult,
};
use thiserror::Error;
use tracing::warn;

use crate::{data_types::BlockType, FstParsable};

/// Blackout Block
pub mod blackout;
/// Geometry Block
pub mod geometry;
/// Header Block
pub mod header;
/// Hierarchy Block
pub mod hierarchy;
/// Value Change Data Block
pub mod value_change_data;

/// Abstract block struct that only holds the type of block ([BlockType]) and location of data (&[[u8]])
#[derive(Clone)]
pub struct Block {
    pub block_type: BlockType,
    data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum DecompressError {
    #[error("Gz decompress error: {0}")]
    Gz(#[from] flate2::DecompressError),
    #[error("Lz4 decompress error: {0}")]
    Lz4(#[from] lz4_flex::block::DecompressError),
    #[error("Io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Block {
    fn extract_data_gz(&self) -> Result<Vec<u8>, DecompressError> {
        let uncompressed_size =
            usize::try_from(u64::from_be_bytes(self.data[..8].try_into().unwrap())).unwrap();
        let mut decompressor = flate2::read::GzDecoder::new(&self.data[8..]);
        let mut data = Vec::new();
        decompressor.read_to_end(&mut data)?;
        if data.len() != uncompressed_size {
            warn!(
                data_len = data.len(),
                uncompressed_size = uncompressed_size,
                "extracted data size did not match specified.",
            );
        }
        Ok(data)
    }

    fn extract_data_lz4(&self) -> Result<Vec<u8>, DecompressError> {
        let uncompressed_size =
            usize::try_from(u64::from_be_bytes(self.data[..8].try_into().unwrap())).unwrap();
        let data = lz4_flex::block::decompress(&self.data[8..], uncompressed_size)?;
        if data.len() != uncompressed_size {
            warn!(
                data_len = data.len(),
                uncompressed_size = uncompressed_size,
                "extracted data size did not match specified.",
            );
        }
        Ok(data)
    }

    fn extract_data_lz4_twice(&self) -> Result<Vec<u8>, DecompressError> {
        let uncompressed_size =
            usize::try_from(u64::from_be_bytes(self.data[..8].try_into().unwrap())).unwrap();
        let uncompressed_once_size =
            usize::try_from(u64::from_be_bytes(self.data[8..16].try_into().unwrap())).unwrap();
        let data = lz4_flex::block::decompress(&self.data[8..], uncompressed_once_size)?;
        if data.len() != uncompressed_once_size {
            warn!(
                data_len = data.len(),
                uncompressed_size = uncompressed_once_size,
                "first extracted data size did not match specified.",
            );
        }
        let data2 = lz4_flex::block::decompress(&data, uncompressed_size)?;
        if data2.len() != uncompressed_size {
            warn!(
                data_len = data2.len(),
                uncompressed_size = uncompressed_size,
                "second extracted data size did not match specified.",
            );
        }
        Ok(data2)
    }

    /// Extracts data from block.
    /// If the block content is compressed, it will be uncompressed in this function.
    pub fn extract_data(&self) -> Result<Vec<u8>, DecompressError> {
        Ok(match self.block_type {
            BlockType::Header => self.data.to_vec(),
            BlockType::ValueChangeData => self.data.to_vec(),
            BlockType::Blackout => self.data.to_vec(),
            BlockType::Geometry => self.data.to_vec(),
            BlockType::HierarchyGz => self.extract_data_gz()?,
            BlockType::ValueChangeDataAlias => self.data.to_vec(),
            BlockType::HierarchyLz4 => self.extract_data_lz4()?,
            BlockType::HierarchyLz4Duo => self.extract_data_lz4_twice()?,
            BlockType::ValueChangeDataAlias2 => self.data.to_vec(),
            BlockType::GZippedWrapper => self.extract_data_gz()?,
            BlockType::Skip => self.data.to_vec(),
        })
    }

    /// Get the raw underlying data bytes.
    /// Useful when calculating offsets from another place in the file.
    pub fn get_data_raw(&self) -> &[u8] {
        &self.data
    }

    fn parse_block_length(input: &[u8]) -> IResult<&[u8], usize, VerboseError<&[u8]>> {
        context(
            "block length",
            map_res(be_u64, |v| {
                v.checked_sub(8)
                    .map(|v| v as usize)
                    .ok_or_else(|| (input, ErrorKind::Verify))
            }),
        )(input)
    }

    pub(crate) fn parse_block_with_position(
        input: &[u8],
    ) -> IResult<&[u8], (Span, Self), VerboseError<&[u8]>> {
        let original_input = input;
        let (input, block_type) = context("block type", BlockType::parse)(input)?;
        let (input, data) =
            context("block data length", length_data(Block::parse_block_length))(input)?;
        let data = data.to_vec();
        let block = Block { block_type, data };
        Ok((input, ((original_input, input), block)))
    }
}

pub type Span<'a> = (&'a [u8], &'a [u8]);

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("block_type", &self.block_type)
            .field("data.len()", &self.data.len())
            .finish()
    }
}
