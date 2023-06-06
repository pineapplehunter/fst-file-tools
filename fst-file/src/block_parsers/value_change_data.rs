use nom::{
    bytes::complete::take,
    multi::length_data,
    number::complete::{be_u64, be_u8},
};

use crate::{
    data_types::{parse_varint, WriterPackType},
    error::FstFileResult,
};

use super::Block;

pub struct ValueChangeDataBlock<'a> {
    block: &'a Block<'a>,
}

pub struct ValueChangeData;

impl<'a> ValueChangeDataBlock<'a> {
    pub fn from_block(block: &'a Block<'a>) -> Self {
        Self { block }
    }

    fn parse_value_change_data(input: &[u8]) -> FstFileResult<'_, ValueChangeData> {
        let (input, start_time) = be_u64(input)?;
        let (input, end_time) = be_u64(input)?;
        let (input, memory_required) = be_u64(input)?;
        let (input, bits_uncompressed_len) = parse_varint(input)?;
        let (input, bits_compressed_length) = parse_varint(input)?;
        let (input, bits_count) = parse_varint(input)?;
        let (input, data) = take(bits_compressed_length)(input)?;
        let (input, waves_count) = parse_varint(input)?;
        let (input, waves_packtype) = WriterPackType::parse(input)?;

        todo!()
    }
}
