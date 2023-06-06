use std::{cell::OnceCell, io::Read};

use nom::{
    bytes::complete::take, combinator::map_res, error::context, multi::many_m_n,
    number::complete::be_u64, Finish,
};
use tracing::debug;

use crate::{
    data_types::VarInt,
    error::{BlockParseError, FstFileParseError, FstFileResult},
    FstParsable,
};

use super::Block;

type GeometryResult<'a> = Result<Geometry, FstFileParseError<&'a [u8]>>;

pub struct GeometryBlock<'a> {
    block: &'a Block<'a>,
    geometry: OnceCell<GeometryResult<'a>>,
}

#[derive(Debug)]
pub struct Geometry(Vec<VarInt>);

impl<'a> GeometryBlock<'a> {
    pub fn from_block(block: &'a Block) -> Self {
        Self {
            block,
            geometry: OnceCell::new(),
        }
    }

    fn get_geometry_cache(&'a self) -> &GeometryResult {
        self.geometry
            .get_or_init(|| Geometry::parse(self.block.data).finish().map(|(_, v)| v))
    }

    pub fn get_geometry(&'a self) -> &GeometryResult {
        self.get_geometry_cache()
    }
}

impl FstParsable for Geometry {
    fn parse(input: &[u8]) -> FstFileResult<'_, Self> {
        let original_input = input;
        let (input, uncompressed_length) = map_res(be_u64, |v: u64| {
            usize::try_from(v).map_err(|_e| (input, BlockParseError::LengthTooLargeForMachine))
        })(input)?;
        let (input, count) = map_res(be_u64, |v| {
            usize::try_from(v).map_err(|_e| (input, BlockParseError::LengthTooLargeForMachine))
        })(input)?;
        let (input, data_raw) = take(original_input.len() - 16)(input)?;

        let mut data;
        if original_input.len() - 16 == uncompressed_length {
            debug!("geometry is not compressed");
            data = data_raw.to_vec();
        } else {
            debug!("geometry is compressed");
            let mut decompressor = flate2::read::ZlibDecoder::new(data_raw);
            data = Vec::new();
            decompressor.read_to_end(&mut data).unwrap();
        }

        let (_, g) = context("inner data", |input| {
            many_m_n(count, count, VarInt::parse)(input)
        })(&data[..])
        .expect("something went wrong while parsing geometry data");

        let geometry = Geometry(g);
        Ok((input, geometry))
    }
}
