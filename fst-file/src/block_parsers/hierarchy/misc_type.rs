use enum_primitive_derive::Primitive;
use nom::{combinator::map_res, number::complete::be_u8};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{block_parsers::hierarchy::HierarchyParseErrorKind, FstParsable};

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
#[repr(u8)]
pub enum MiscType {
    Comment = 0,
    EnvVar = 1,
    SupVar = 2,
    PathName = 3,
    SourceStem = 4,
    SourceIStem = 5,
    ValueList = 6,
    EnumTable = 7,
    Unknown = 8,
}

impl FstParsable for MiscType {
    fn parse(input: &[u8]) -> crate::error::FstFileResult<'_, Self> {
        map_res(be_u8, |v| {
            MiscType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongMiscType(v)))
        })(input)
    }
}
