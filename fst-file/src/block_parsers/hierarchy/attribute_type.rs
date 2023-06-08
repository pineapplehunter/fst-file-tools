use enum_primitive_derive::Primitive;
use nom::{combinator::map_res, number::complete::be_u8};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{block_parsers::hierarchy::HierarchyParseErrorKind, FstParsable};

/// Types of attributes in [crate::block_parsers::hierarchy]
#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
#[repr(u8)]
pub enum AttributeType {
    Misc = 0,
    Array = 1,
    Enum = 2,
    Pack = 3,
}

impl FstParsable for AttributeType {
    fn parse(input: &[u8]) -> crate::error::FstFileResult<'_, Self> {
        map_res(be_u8, |v| {
            AttributeType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongAttributeType(v)))
        })(input)
    }
}
