use std::{borrow::Cow, io::Read};

use nom::{
    bytes::complete::take,
    error::{context, VerboseErrorKind},
    multi::many_m_n,
    number::complete::be_u64,
    Finish,
};
use serde::Serialize;
use thiserror::Error;
use tracing::debug;

use crate::{
    as_usize,
    data_types::VarInt,
    error::{ParseResult, PositionError},
    FstParsable,
};

use super::Block;

#[derive(Debug)]
pub struct GeometryBlock(Block);

#[derive(Debug, Serialize)]
pub struct Geometry(Vec<VarInt>);

#[derive(Debug, Error)]
pub enum GeometryParseError {
    #[error("parse error {0}")]
    ParseError(#[from] PositionError<VerboseErrorKind>),
}

impl GeometryBlock {
    pub fn from_block(block: Block) -> Self {
        Self(block)
    }

    pub fn get_content(&self) -> Result<Geometry, GeometryParseError> {
        let data = self.0.get_data_raw();
        Ok(Geometry::parse(data)
            .finish()
            .map(|(_, v)| v)
            .map_err(|e| PositionError::from_verbose_parse_error(e, data))?)
    }
}

impl FstParsable for Geometry {
    fn parse(input: &[u8]) -> ParseResult<Self> {
        let original_input = input;
        let (input, uncompressed_length) = as_usize(be_u64)(input)?;
        let (input, count) = as_usize(be_u64)(input)?;
        let (input, data_raw) = take(original_input.len() - 16)(input)?;

        let data = if original_input.len() - 16 == uncompressed_length {
            debug!("geometry is not compressed");
            Cow::Borrowed(data_raw)
        } else {
            debug!("geometry is compressed");
            let mut decompressor = flate2::read::ZlibDecoder::new(data_raw);
            let mut data_tmp = Vec::new();
            decompressor.read_to_end(&mut data_tmp).unwrap();
            Cow::Owned(data_tmp)
        };

        let (_, g) = context("inner data", |input| {
            many_m_n(count, count, VarInt::parse)(input)
        })(&data)
        .expect("something went wrong while parsing geometry data");

        let geometry = Geometry(g);
        Ok((input, geometry))
    }
}
