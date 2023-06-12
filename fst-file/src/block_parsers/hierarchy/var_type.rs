use enum_primitive_derive::Primitive;
use nom::{combinator::map_res, number::complete::be_u8};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{block_parsers::hierarchy::HierarchyParseErrorKind, error::ParseResult, FstParsable};

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
pub enum VarType {
    VcdEvent = 0,
    VcdInteger = 1,
    VcdParameter = 2,
    VcdReal = 3,
    VcdRealParameter = 4,
    VcdReg = 5,
    VcdSupply0 = 6,
    VcdSupply1 = 7,
    VcdTime = 8,
    VcdTri = 9,
    VcdTriAnd = 10,
    VcdTriOr = 11,
    VcdTriReg = 12,
    VcdTri0 = 13,
    VcdTri1 = 14,
    VcdWand = 15,
    VcdWire = 16,
    VcdWor = 17,
    VcdPort = 18,
    VcdSparray = 19,
    VcdRealtime = 20,
    GenString = 21,
    SvBit = 22,
    SvLogic = 23,
    SvInt = 24,
    SvShortInt = 25,
    SvLongInt = 26,
    SvByte = 27,
    SvEnum = 28,
    SvShortReal = 29,
}

impl FstParsable for VarType {
    fn parse(input: &[u8]) -> ParseResult<Self> {
        map_res(be_u8, |v| {
            VarType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongVarType(v)))
        })(input)
    }
}
