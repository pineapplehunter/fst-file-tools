use nom::{branch::alt, bytes::complete::tag, combinator::map, error::context};

use crate::{error::ParseResult, FstParsable};

#[derive(Debug)]
pub enum WriterPackType {
    Zlib,
    FaslLz,
    Lz4,
}

impl FstParsable for WriterPackType {
    fn parse(input: &[u8]) -> ParseResult<'_, Self> {
        context(
            "writer pack",
            alt((
                map(tag(&[b'!']), |_| Self::Zlib),
                map(tag(&[b'Z']), |_| Self::Zlib),
                map(tag(&[b'F']), |_| Self::FaslLz),
                map(tag(&[b'4']), |_| Self::Lz4),
            )),
        )(input)
    }
}
