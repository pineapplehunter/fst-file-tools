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
pub struct BlackoutContent {
    records: Vec<BlackoutRecord>,
}

fn parse_record(input: &[u8]) -> FstFileResult<'_, BlackoutRecord> {
    let (input, active) = map(take(1u8), |b: &[u8]| b[0] == 1)(input)?;
    let (input, time_delta) = parse_varint(input)?;
    Ok((input, BlackoutRecord { active, time_delta }))
}

pub fn parse_blackout_content(input: &[u8]) -> FstFileResult<'_, BlackoutContent> {
    let (input, count) = map_res(parse_varint, |v| {
        usize::try_from(v).map_err(|_e| (input,BlockParseError::LengthTooLargeForMachine))
    })(input)?;
    let (input, records) = many_m_n(count, count, parse_record)(input)?;

    let data = BlackoutContent { records };

    let (_input, _) = eof(input)?;
    Ok((input, data))
}
