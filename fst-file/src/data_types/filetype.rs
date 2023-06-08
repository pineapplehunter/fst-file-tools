use enum_primitive_derive::Primitive;
use nom::{combinator::map_res, number::complete::be_u8};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{error::BlockParseError, FstParsable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive)]
#[repr(u8)]
pub enum FileType {
    #[doc(alias = "FST_FT_VERILOG")]
    Verilog = 0,
    #[doc(alias = "FST_FT_VHDL")]
    Vhdl = 1,
    #[doc(alias = "FST_FT_VERILOG_VHDL")]
    VerilogVhdl = 2,
}

impl Serialize for FileType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl FstParsable for FileType {
    fn parse(input: &[u8]) -> crate::error::FstFileResult<'_, Self> {
        map_res(be_u8, |i| {
            FileType::from_u8(i).ok_or((input, BlockParseError::WrongFileType))
        })(input)
    }
}
