use enum_primitive_derive::Primitive;
use nom::{combinator::map_res, number::complete::be_u8};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{block_parsers::hierarchy::HierarchyParseErrorKind, FstParsable};

/// Signal direction
#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
pub enum VarDir {
    Implicit = 0,
    Input = 1,
    Output = 2,
    Inout = 3,
    Buffer = 4,
    Linkage = 5,
}

impl FstParsable for VarDir {
    fn parse(input: &[u8]) -> crate::error::FstFileResult<'_, Self> {
        map_res(be_u8, |v| {
            VarDir::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongVarDir(v)))
        })(input)
    }
}
