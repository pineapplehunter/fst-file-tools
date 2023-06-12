use std::{
    fmt,
    ops::{Shl, Shr},
};

use nom::{
    bytes::complete::{take, take_while_m_n},
    combinator::{map, verify},
    error::{context, make_error, ErrorKind},
};
use num_traits::WrappingSub;
use serde::Serialize;

use crate::{error::ParseResult, FstParsable};

/// Variable sized unsigned int
///
/// The dos said that 64 bits is enough.
/// See docs for more information. <https://blog.timhutt.co.uk/fst_spec/#_varints>
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VarInt(pub u64);

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
        serializer.serialize_u64(self.0)
    }
}

impl FstParsable for VarInt {
    /// Parse a [VarInt] from &[[u8]]
    fn parse<'a>(input: &'a [u8]) -> ParseResult<'_, VarInt> {
        context("varint", |input: &'a [u8]| {
            let input_original = input;
            let (input, data) = take_while_m_n(0, 20, |b| b & 0b1000_0000 != 0)(input)?;
            let (input, last) = verify(take(1u8), |v: &[u8]| v[0] & 0b1000_0000 == 0)(input)?;
            let mut val = 0;
            val |= last[0] as u64;
            for s in data.iter().rev() {
                let v: u64 = val.shl(7);
                if val != v.shr(7) {
                    return Err(nom::Err::Error(make_error(
                        input_original,
                        ErrorKind::TooLarge,
                    )));
                }
                val = v;
                val |= (s & 0b0111_1111) as u64;
            }
            Ok((input, VarInt(val)))
        })(input)
    }
}

/// Variable sized signed int
///
/// Signed variant of [VarInt]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SVarInt(pub i64);

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
        serializer.serialize_i64(self.0)
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
    fn parse<'a>(input: &'a [u8]) -> ParseResult<'_, SVarInt> {
        context("svarint", |input: &'a [u8]| {
            let input_original = input;
            let (input, data) = take_while_m_n(0, 20, |b| b & 0b1000_0000 != 0)(input)?;
            let (input, last) = map(take(1u8), |v: &[u8]| v[0])(input)?;
            let mut val = 0;
            if last & 0b0100_0000 != 0 {
                val = val.wrapping_sub(&1);
            }
            val |= last as i64;
            for s in data.iter().rev() {
                let v: i64 = val.shl(7);
                if val != v.shr(7) {
                    return Err(nom::Err::Error(make_error(
                        input_original,
                        ErrorKind::TooLarge,
                    )));
                }
                val = v;
                val |= (s & 0b0111_1111) as i64;
            }
            Ok((input, SVarInt(val)))
        })(input)
    }
}

// /// Errors that could occour while parsing VarInt
// #[derive(Debug, Error, Clone, PartialEq)]
// pub enum VarIntParseErrorKind {
//     #[error("value did not fit in u64")]
//     TooLarge,
//     #[error("nom error {0:?}")]
//     NomError(ErrorKind),
// }

#[cfg(test)]
mod test {
    use nom::{
        error::{ErrorKind, VerboseErrorKind},
        Finish,
    };

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
    fn varint_toolarge() {
        let input = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
        ];
        let err = VarInt::parse(&input).finish().unwrap_err();
        assert_eq!(err.errors[0].1, VerboseErrorKind::Nom(ErrorKind::TooLarge));
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

    #[test]
    fn svarint_toolarge() {
        let input = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01,
        ];
        let err = SVarInt::parse(&input).finish().unwrap_err();
        assert_eq!(err.errors[0].1, VerboseErrorKind::Nom(ErrorKind::TooLarge));

        let input = [
            0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x7F,
        ];
        let err = SVarInt::parse(&input).finish().unwrap_err();
        assert_eq!(err.errors[0].1, VerboseErrorKind::Nom(ErrorKind::TooLarge));
    }
}
