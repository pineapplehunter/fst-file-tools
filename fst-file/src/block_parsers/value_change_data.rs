use std::{borrow::Cow, io::Read};

use nom::{
    bytes::complete::take, error::VerboseErrorKind, multi::many_m_n, number::complete::be_u64,
    Finish,
};
use serde::Serialize;
use thiserror::Error;

use crate::{
    as_usize,
    data_types::{SVarInt, VarInt, WriterPackType},
    error::{ParseResult, PositionError},
    FstParsable,
};

use super::Block;

#[derive(Debug)]
pub struct ValueChangeDataBlock {
    block: Block,
    header_num_vars: usize,
}

#[derive(Debug, Serialize)]
pub struct ValueChangeData {
    start_time: u64,
    end_time: u64,
    memory_required: u64,
    bits_data: Vec<u8>,
    position_data: Vec<SVarInt>,
    time_data: Vec<VarInt>,
    wave_data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ValueChangeDataError {
    #[error("parse error {0}")]
    ParseError(#[from] PositionError<VerboseErrorKind>),
}

impl ValueChangeDataBlock {
    pub fn from_block_and_header_data(block: Block, header_num_vars: usize) -> Self {
        Self {
            block,
            header_num_vars,
        }
    }

    pub fn get_content(&self) -> Result<ValueChangeData, ValueChangeDataError> {
        let data = self.block.get_data_raw();
        Ok(self
            .parse_value_change_data(data)
            .finish()
            .map(|(_, v)| v)
            .map_err(|e| PositionError::from_verbose_parse_error(e, data))?)
    }

    // pub fn get_parsed_data(&self) ->  {
    //     self.value_change_data.get_or_init(move || {
    //         self.parse_value_change_data(self.block.data)
    //             .finish()
    //             .map(|(_, v)| v)
    //             .map_err(|e| PositionError::from_fst_parse_error(e, self.block.data))
    //     })
    // }

    fn parse_value_change_data<'a>(&'a self, input: &'a [u8]) -> ParseResult<ValueChangeData> {
        let (input, start_time) = be_u64(input)?;
        let (input, end_time) = be_u64(input)?;
        let (input, memory_required) = be_u64(input)?;
        let (input, bits_uncompressed_len) = as_usize(VarInt::parse)(input)?;
        let (input, bits_compressed_length) = as_usize(VarInt::parse)(input)?;
        let (input, bits_count) = VarInt::parse(input)?;
        let (input, bits_data_raw) = take(bits_compressed_length)(input)?;
        dbg!(
            &start_time,
            &end_time,
            &memory_required,
            &bits_compressed_length,
            &bits_compressed_length,
            &bits_count
        );
        let bits_data = if bits_compressed_length == bits_uncompressed_len {
            bits_data_raw.to_vec()
        } else {
            let mut decoder = flate2::read::ZlibDecoder::new(bits_data_raw);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).unwrap();
            buf
        };
        // dbg!(&bits_data);
        let (input, waves_count) = VarInt::parse(input)?;
        let (input, waves_packtype) = WriterPackType::parse(input)?;
        dbg!(&waves_count, &waves_packtype);

        let (input, input_end) = input.split_at(input.len() - 24);
        let (input_end, time_uncompressed_length) = as_usize(be_u64)(input_end)?;
        let (input_end, time_compressed_length) = as_usize(be_u64)(input_end)?;
        let (_, time_count) = as_usize(be_u64)(input_end)?;
        dbg!(
            &time_uncompressed_length,
            &time_compressed_length,
            &time_count
        );

        let (input, time_data_raw) = input.split_at(input.len() - time_compressed_length);
        let time_data_buf = if time_compressed_length == time_uncompressed_length {
            Cow::Borrowed(time_data_raw)
        } else {
            let mut decoder = flate2::read::ZlibDecoder::new(time_data_raw);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).unwrap();
            Cow::Owned(buf)
        };
        let (_, time_data) =
            many_m_n(time_count, time_count, VarInt::parse)(&time_data_buf).unwrap();
        dbg!(&time_data);

        let (input, input_end) = input.split_at(input.len() - 8);
        let (_, position_length) = as_usize(be_u64)(input_end)?;
        dbg!(&position_length);
        let (waves_data_raw, position_data_raw) = input.split_at(input.len() - position_length);
        let (_, position_data) =
            many_m_n(self.header_num_vars, self.header_num_vars, SVarInt::parse)(
                position_data_raw,
            )?;
        dbg!(&position_data);
        // dbg!(&waves_data_raw);

        let vcd = ValueChangeData {
            start_time,
            end_time,
            memory_required,
            position_data,
            bits_data,
            time_data,
            wave_data: waves_data_raw.to_vec(),
        };
        Ok((input, vcd))
    }
}

// fn a() {
//     let b = ValueChangeDataBlock::from_block(
//         &Block {
//             block_type: BlockType::Skip,
//             data: &[],
//         },
//         0,
//     );
//     b.get_parsed_data().as_ref().unwrap();
// }
