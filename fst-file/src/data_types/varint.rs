use nom::{
    bytes::complete::{take, take_while},
    error::{ErrorKind, ParseError},
    IResult,
};
use thiserror::Error;

pub type VarInt = u64;
pub type SVarInt = i64;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum VarIntParseErrorKind {
    #[error("value did not fit in u64")]
    TooLarge,
    #[error("nom error {0:?}")]
    NomError(ErrorKind),
}

#[derive(Debug, Error)]
pub struct VarIntParseError<I> {
    pub errors: Vec<(I, VarIntParseErrorKind)>,
}

impl<I> ParseError<I> for VarIntParseError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            errors: vec![(input, VarIntParseErrorKind::NomError(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other
            .errors
            .push((input, VarIntParseErrorKind::NomError(kind)));
        other
    }
}

impl<I, E> From<(I, E)> for VarIntParseError<I>
where
    E: Into<VarIntParseErrorKind>,
{
    fn from(value: (I, E)) -> Self {
        VarIntParseError {
            errors: vec![(value.0, value.1.into())],
        }
    }
}

pub fn parse_varint(input_original: &[u8]) -> IResult<&[u8], VarInt, VarIntParseError<&[u8]>> {
    let input = input_original;
    let (input, data) = take_while(|b| b & 0b100_0000 != 0)(input)?;
    let (input, last) = take(1u8)(input)?;
    let mut val = 0;
    val += last[0] as u64;
    for s in data.iter().rev() {
        if let Some(v) = val.checked_shl(7) {
            val = v;
        } else {
            return Err(nom::Err::Error(
                (input_original, VarIntParseErrorKind::TooLarge).into(),
            ));
        }
        val += (s & 0b0111_1111) as u64;
    }
    Ok((input, val))
}

#[cfg(test)]
mod test {
    use super::parse_varint;

    #[test]
    fn varint() {
        let input = [0xC5, 0x18];
        let (_i, a) = parse_varint(&input).unwrap();
        assert_eq!(a, 3141);

        let input = [0x01];
        let (_i, a) = parse_varint(&input).unwrap();
        assert_eq!(a, 1);
    }
}
