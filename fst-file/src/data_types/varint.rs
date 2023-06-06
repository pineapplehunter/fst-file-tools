use std::fmt;

use nom::{
    bytes::complete::{take, take_while},
    error::ErrorKind,
};
use num_traits::WrappingSub;
use serde::Serialize;
use thiserror::Error;

use crate::{error::FstFileResult, FstParsable};

/// Variable sized unsigned int
///
/// The dos said that 64 bits is enough, so 128 should be safe.
/// See docs for more information. <https://blog.timhutt.co.uk/fst_spec/#_varints>
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VarInt(pub u128);

impl fmt::Debug for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u128(self.0)
    }
}

impl FstParsable for VarInt {
    /// Parse a [VarInt] from &[[u8]]
    fn parse(input: &[u8]) -> FstFileResult<'_, VarInt> {
        let input_original = input;
        let (input, data) = take_while(|b| b & 0b1000_0000 != 0)(input)?;
        let (input, last) = take(1u8)(input)?;
        let mut val = 0;
        val |= last[0] as u128;
        for s in data.iter().rev() {
            if let Some(v) = val.checked_shl(7) {
                val = v;
            } else {
                return Err(nom::Err::Error(
                    (input_original, VarIntParseErrorKind::TooLarge).into(),
                ));
            }
            val |= (s & 0b0111_1111) as u128;
        }
        Ok((input, VarInt(val)))
    }
}

/// Variable sized signed int
///
/// Signed variant of [VarInt]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SVarInt(pub i128);

impl fmt::Debug for SVarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for SVarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i128(self.0)
    }
}

impl TryFrom<VarInt> for usize {
    type Error = <usize as TryFrom<u128>>::Error;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        usize::try_from(value.0)
    }
}

impl FstParsable for SVarInt {
    /// Parse a [SVarInt] from &[[u8]]
    fn parse(input: &[u8]) -> FstFileResult<'_, SVarInt> {
        let input_original = input;
        let (input, data) = take_while(|b| b & 0b1000_0000 != 0)(input)?;
        let (input, last) = take(1u8)(input)?;
        let mut val = 0;
        if last[0] & 0b0100_0000 != 0 {
            val = val.wrapping_sub(&1);
        }
        val |= last[0] as i128;
        for s in data.iter().rev() {
            if let Some(v) = val.checked_shl(7) {
                val = v;
            } else {
                return Err(nom::Err::Error(
                    (input_original, VarIntParseErrorKind::TooLarge).into(),
                ));
            }
            val |= (s & 0b0111_1111) as i128;
        }
        Ok((input, SVarInt(val)))
    }
}

/// Errors that could occour while parsing VarInt
#[derive(Debug, Error, Clone, PartialEq)]
pub enum VarIntParseErrorKind {
    #[error("value did not fit in u64")]
    TooLarge,
    #[error("nom error {0:?}")]
    NomError(ErrorKind),
}

#[cfg(test)]
mod test {
    use crate::{
        data_types::{SVarInt, VarInt},
        FstParsable,
    };

    #[test]
    fn varint() {
        let input = [0xC5, 0x18];
        let (_i, a) = VarInt::parse(&input).unwrap();
        assert_eq!(a, VarInt(3141));

        let input = [0x01];
        let (_i, a) = VarInt::parse(&input).unwrap();
        assert_eq!(a, VarInt(1));

        let input = [0x58];
        let (_i, a) = VarInt::parse(&input).unwrap();
        assert_eq!(a, VarInt(0x58));
    }

    #[test]
    fn svarint() {
        let input = [0xC5, 0x18];
        let (_i, a) = SVarInt::parse(&input).unwrap();
        assert_eq!(a, SVarInt(3141));

        let input = [0xC5, 0x58];
        let (_i, a) = SVarInt::parse(&input).unwrap();
        assert_eq!(a, SVarInt(-59));

        let input = [0xBB, 0x87, 0x7F];
        let (_i, a) = SVarInt::parse(&input).unwrap();
        assert_eq!(a, SVarInt(-15429));
    }
}
