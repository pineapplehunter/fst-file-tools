use nom::{
    bytes::complete::{take, take_while},
    error::ErrorKind,
};
use num_traits::WrappingSub;
use thiserror::Error;

use crate::error::FstFileResult;

pub type VarInt = u128;
pub type SVarInt = i128;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum VarIntParseErrorKind {
    #[error("value did not fit in u64")]
    TooLarge,
    #[error("nom error {0:?}")]
    NomError(ErrorKind),
}

pub fn parse_varint(input: &[u8]) -> FstFileResult<'_, VarInt> {
    let input_original = input;
    let (input, data) = take_while(|b| b & 0b1000_0000 != 0)(input)?;
    let (input, last) = take(1u8)(input)?;
    let mut val = 0;
    val |= last[0] as VarInt;
    for s in data.iter().rev() {
        if let Some(v) = val.checked_shl(7) {
            val = v;
        } else {
            return Err(nom::Err::Error(
                (input_original, VarIntParseErrorKind::TooLarge).into(),
            ));
        }
        val |= (s & 0b0111_1111) as VarInt;
    }
    Ok((input, val))
}

pub fn parse_svarint(input: &[u8]) -> FstFileResult<'_, SVarInt> {
    let input_original = input;
    let (input, data) = take_while(|b| b & 0b1000_0000 != 0)(input)?;
    let (input, last) = take(1u8)(input)?;
    let mut val = 0;
    if last[0] & 0b0100_0000 != 0 {
        val = val.wrapping_sub(&1);
    }
    val |= last[0] as SVarInt;
    for s in data.iter().rev() {
        if let Some(v) = val.checked_shl(7) {
            val = v;
        } else {
            return Err(nom::Err::Error(
                (input_original, VarIntParseErrorKind::TooLarge).into(),
            ));
        }
        val |= (s & 0b0111_1111) as SVarInt;
    }
    Ok((input, val))
}

#[cfg(test)]
mod test {
    use crate::data_types::parse_svarint;

    use super::parse_varint;

    #[test]
    fn varint() {
        let input = [0xC5, 0x18];
        let (_i, a) = parse_varint(&input).unwrap();
        assert_eq!(a, 3141);

        let input = [0x01];
        let (_i, a) = parse_varint(&input).unwrap();
        assert_eq!(a, 1);

        let input = [0x58];
        let (_i, a) = parse_varint(&input).unwrap();
        assert_eq!(a, 0x58);
    }

    #[test]
    fn svarint() {
        let input = [0xC5, 0x18];
        let (_i, a) = parse_svarint(&input).unwrap();
        assert_eq!(a, 3141);

        let input = [0xC5, 0x58];
        let (_i, a) = parse_svarint(&input).unwrap();
        assert_eq!(a, -59);

        let input = [0xBB, 0x87, 0x7F];
        let (_i, a) = parse_svarint(&input).unwrap();
        assert_eq!(a, -15429);
    }
}
