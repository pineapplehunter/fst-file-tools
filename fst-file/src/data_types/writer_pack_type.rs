use nom::{branch::alt, bytes::complete::tag, combinator::map};

use crate::{error::FstFileResult, FstParsable};

pub enum WriterPackType {
    Zlib,
    FaslLz,
    Lz4,
}

impl FstParsable for WriterPackType {
    fn parse(input: &[u8]) -> FstFileResult<'_, Self> {
        alt((
            map(tag(&[b'!']), |_| Self::Zlib),
            map(tag(&[b'Z']), |_| Self::Zlib),
            map(tag(&[b'F']), |_| Self::FaslLz),
            map(tag(&[b'4']), |_| Self::Lz4),
        ))(input)
    }
}
