use std::{borrow::Cow, io::Read};

use nom::{
    bytes::complete::take, error::VerboseErrorKind, multi::many_m_n, number::complete::be_u64,
    Finish,
};
use serde::Serialize;
use thiserror::Error;
use tracing::debug_span;

use crate::{
    as_usize,
    data_types::{BlockType, SVarInt, VarInt, WriterPackType},
    error::{ParseResult, PositionError},
    FstParsable, convert_type,
};

use super::{header::HeaderBlockContent, Block};

#[derive(Debug)]
pub struct ValueChangeDataBlock(Block);

#[derive(Debug, Serialize)]
pub struct ValueChangeDataIntermediate {
    start_time: u64,
    end_time: u64,
    memory_required: u64,
    bits_data: Vec<u8>,
    position_data_raw: Vec<u8>,
    time_data: Vec<VarInt>,
    wave_data_raw: Vec<u8>,
    waves_packtype: WriterPackType,
    waves_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ValueChangeData {
    time_data: Vec<u64>,
    chain_table: Vec<i64>,
    chain_table_lengths: Vec<u32>,
}

#[derive(Debug, Error)]
pub enum ValueChangeDataError {
    #[error("parse error {0}")]
    ParseError(#[from] PositionError<VerboseErrorKind>),
    #[error("interger convert to other type error {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}

impl ValueChangeDataBlock {
    pub fn from_block(block: Block) -> Self {
        Self(block)
    }

    pub fn get_intermediate_content(
        &self,
        _header_content: &HeaderBlockContent,
    ) -> Result<ValueChangeDataIntermediate, ValueChangeDataError> {
        let _span = debug_span!("get_intermediate_content").entered();
        let data = self.0.get_data_raw();
        Ok(self
            .parse_value_change_data(data)
            .finish()
            .map(|(_, v)| v)
            .map_err(|e| PositionError::from_verbose_parse_error(e, data))?)
    }

    pub fn get_content(
        &self,
        header_content: &HeaderBlockContent,
    ) -> Result<ValueChangeData, ValueChangeDataError> {
        let _span = debug_span!("get_content").entered();
        let intermediate = self.get_intermediate_content(header_content)?;

        let mut time_data = vec![0; intermediate.time_data.len()];
        let mut previous_time_value = 0;
        for (i, t) in intermediate.time_data.iter().enumerate() {
            previous_time_value += t.0;
            time_data[i] = previous_time_value;
        }

        // get chain_table and chain_table_length
        assert_eq!(self.0.block_type, BlockType::ValueChangeDataAlias2);
        let vc_max_handle = intermediate.waves_count;
        let mut chain_table = vec![0; vc_max_handle + 1];
        let mut chain_table_lengths = vec![0u32; vc_max_handle + 1];
        let mut pval = 0;
        let mut pidx = 0;
        let mut prev_alias = 0;
        let mut position_data_ptr = &intermediate.position_data_raw[..];
        let mut idx = 0;
        loop {
            if position_data_ptr[0] & 1 != 0 {
                let (t, val) = SVarInt::parse(position_data_ptr).finish().map_err(|e|PositionError::from_verbose_parse_error(e, &intermediate.position_data_raw[..]))?;
                position_data_ptr = t;
                let shval = val.0 >> 1;
                match shval {
                    shval if shval > 0 => {
                        pval += shval;
                        chain_table[idx] = pval;

                        if idx != 0 {
                            chain_table_lengths[pidx] = u32::try_from(pval - chain_table[pidx])?;
                        }
                        pidx = idx;
                        idx += 1;
                    }
                    shval if shval < 0 => {
                        chain_table[idx] = 0;
                        prev_alias = u32::try_from(shval)?;
                        chain_table_lengths[idx] = u32::try_from(shval)?;
                        idx += 1;
                    }
                    _ => {
                        chain_table[idx] = 0;
                        chain_table_lengths[idx] = prev_alias;
                        idx += 1;
                    }
                }
            } else {
                let (t, val):(_,u32) = convert_type(VarInt::parse)(position_data_ptr).finish().map_err(|e|PositionError::from_verbose_parse_error(e, &intermediate.position_data_raw[..]))?;
                position_data_ptr = t;
                let loopcnt = val >> 1;
                for _i in 0..loopcnt {
                    chain_table[idx] = 0;
                    idx += 1;
                }
            }
            if position_data_ptr.is_empty() {
                break;
            }
        }
        chain_table[idx] = intermediate.wave_data_raw.len() as i64 + 1;
        chain_table_lengths[pidx] = u32::try_from(chain_table[idx] - chain_table[pidx])?;

        // This check was implemented in gtk wave as a sanity check.
        // since this implementation cannot have negative values as length
        // for i in 0..idx {
        //     let mut v32 = chain_table_lengths[i];
            // if (v32 < 0) && (chain_table[i] != 0) {
            //     v32 = -v32;
            //     v32 -= 1;
            //     let v32: usize = v32.try_into().unwrap();
            //     if v32 < i {
            //         chain_table[i] = chain_table[v32];
            //         chain_table_lengths[i] = chain_table_lengths[v32];
            //     }
            // }
        // }

        // for i in 0..idx {
        //     if(chain_table[i] != 0) {
        //         let process_idx = i/8;
        //         let  process_bit = i&7;

        //     }
        // }

        Ok(ValueChangeData {
            time_data,
            chain_table,
            chain_table_lengths,
        })
    }

    // pub fn get_parsed_data(&self) ->  {
    //     self.value_change_data.get_or_init(move || {
    //         self.parse_value_change_data(self.block.data)
    //             .finish()
    //             .map(|(_, v)| v)
    //             .map_err(|e| PositionError::from_fst_parse_error(e, self.block.data))
    //     })
    // }

    fn parse_value_change_data<'a>(
        &'a self,
        input: &'a [u8],
    ) -> ParseResult<ValueChangeDataIntermediate> {
        let (input, start_time) = be_u64(input)?;
        let (input, end_time) = be_u64(input)?;
        let (input, memory_required) = be_u64(input)?;
        let (input, bits_uncompressed_len) = as_usize(VarInt::parse)(input)?;
        let (input, bits_compressed_length) = as_usize(VarInt::parse)(input)?;
        let (input, _bits_count) = VarInt::parse(input)?;
        let (input, bits_data_raw) = take(bits_compressed_length)(input)?;
        // dbg!(
        //     &start_time,
        //     &end_time,
        //     &memory_required,
        //     &bits_compressed_length,
        //     &bits_compressed_length,
        //     &bits_count
        // );
        let bits_data = if bits_compressed_length == bits_uncompressed_len {
            bits_data_raw.to_vec()
        } else {
            let mut decoder = flate2::read::ZlibDecoder::new(bits_data_raw);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf).unwrap();
            buf
        };
        // dbg!(&bits_data);
        let (input, waves_count) = as_usize(VarInt::parse)(input)?;
        let (input, waves_packtype) = WriterPackType::parse(input)?;
        // dbg!(&waves_count, &waves_packtype);

        let (input, input_end) = input.split_at(input.len() - 24);
        let (input_end, time_uncompressed_length) = as_usize(be_u64)(input_end)?;
        let (input_end, time_compressed_length) = as_usize(be_u64)(input_end)?;
        let (_, time_count) = as_usize(be_u64)(input_end)?;
        // dbg!(
        //     &time_uncompressed_length,
        //     &time_compressed_length,
        //     &time_count
        // );

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
        // dbg!(&time_data);

        let (input, input_end) = input.split_at(input.len() - 8);
        let (_, position_length) = as_usize(be_u64)(input_end)?;
        // dbg!(&position_length);
        let (waves_data_raw, position_data_raw) = input.split_at(input.len() - position_length);

        let vcd = ValueChangeDataIntermediate {
            start_time,
            end_time,
            memory_required,
            position_data_raw: position_data_raw.to_vec(),
            bits_data,
            time_data,
            wave_data_raw: waves_data_raw.to_vec(),
            waves_count,
            waves_packtype,
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
