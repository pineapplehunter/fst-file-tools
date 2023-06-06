use block_parsers::Block;
use data_types::Blocks;
use error::{FstFileParseError, FstFileResult};
use nom::{combinator::eof, error::context, multi::many_till, Finish};

/// Block data and their parsers
pub mod block_parsers;
pub mod data_types;
pub mod error;

/// Parses blocks
fn parse_blocks(input: &[u8]) -> FstFileResult<'_, Blocks> {
    let input_original = input;
    let (input, (blocks, _)) = many_till(context("parse block", Block::parse_block), eof)(input)?;
    let blocks = Blocks {
        start_of_input: input_original,
        blocks,
    };
    Ok((input, blocks))
}

/// Parse the whole content of the fst file
pub fn parse_file(input: &[u8]) -> Result<Blocks, FstFileParseError<&[u8]>> {
    parse_blocks(input).finish().map(|(_, blocks)| blocks)
}

/// Parsable types
pub(crate) trait FstParsable: Sized {
    /// parse data from &[[u8]] and give [Self]
    fn parse(input: &[u8]) -> FstFileResult<'_, Self>;
}
