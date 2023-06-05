use nom::bytes::complete::{take, take_while};

use crate::error::FstFileResult;

pub type VarInt = u64;
pub type SVarInt = i64;

pub fn parse_varint(input: &[u8]) -> FstFileResult<'_, VarInt> {
    let (input, data) = take_while(|b| b & 0b100_0000 != 0)(input)?;
    let (input, last) = take(1u8)(input)?;
    let mut val = 0;
    val += last[0] as u64;
    for s in data.iter().rev() {
        val <<= 7;
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
