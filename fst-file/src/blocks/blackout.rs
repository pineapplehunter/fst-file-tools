use nom::{
    bytes::complete::take,
    combinator::{eof, map, map_res},
    multi::many_m_n,
};

use crate::{
    data_types::{parse_varint, VarInt},
    error::{BlockParseError, FstFileResult},
};

#[derive(Debug, Clone, PartialEq)]
pub struct BlackoutRecord {
    active: bool,
    time_delta: VarInt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlackoutBlock {
    records: Vec<BlackoutRecord>,
}

fn parse_record<'a>(input: &'a [u8]) -> FstFileResult<'a, BlackoutRecord> {
    let (input, active) = map(take(1u8), |b: &[u8]| b[0] == 1)(input)?;
    let (input, time_delta) = parse_varint(input)?;
    Ok((input, BlackoutRecord { active, time_delta }))
}

pub fn parse_blackout_block<'a>(input: &'a [u8]) -> FstFileResult<'a, BlackoutBlock> {
    let (input, count) = map_res(parse_varint, |v| {
        usize::try_from(v).map_err(|_e| BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let (input, records) = many_m_n(count, count, parse_record)(input)?;

    let data = BlackoutBlock { records };

    let (_input, _) = eof(input)?;
    Ok((input, data))
}
