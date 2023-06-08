use std::{borrow::Cow, cell::OnceCell, io::Read};

use nom::{
    bytes::complete::take,
    combinator::map_res,
    multi::length_data,
    number::complete::{be_u64, be_u8},
};

use crate::{
    as_usize,
    data_types::{VarInt, WriterPackType},
    error::{BlockParseError, FstFileResult},
    FstParsable,
};

use super::Block;

pub struct ValueChangeDataBlock<'a> {
    block: &'a Block<'a>,
    bits_data: OnceCell<Cow<'a, [u8]>>,
}

pub struct ValueChangeData;

impl<'a> ValueChangeDataBlock<'a> {
    pub fn from_block(block: &'a Block<'a>) -> Self {
        Self {
            block,
            bits_data: OnceCell::new(),
        }
    }

    fn parse_value_change_data(input: &[u8]) -> FstFileResult<'_, ValueChangeData> {
        let (input, start_time) = be_u64(input)?;
        let (input, end_time) = be_u64(input)?;
        let (input, memory_required) = be_u64(input)?;
        let (input, bits_uncompressed_len) = as_usize(VarInt::parse)(input)?;
        let (input, bits_compressed_length) = as_usize(VarInt::parse)(input)?;
        let (input, bits_count) = VarInt::parse(input)?;
        let (input, data) = take(bits_compressed_length)(input)?;
        let bits_data;
        if bits_compressed_length == bits_uncompressed_len {
            bits_data = Cow::Borrowed(data);
        } else {
            let mut decoder = flate2::read::ZlibDecoder::new(data);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).unwrap();
            bits_data = Cow::Owned(buf);
        }
        let (input, waves_count) = VarInt::parse(input)?;
        let (input, waves_packtype) = WriterPackType::parse(input)?;

        todo!()
    }
}
