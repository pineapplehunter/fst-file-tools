use enum_primitive_derive::Primitive;
use nom::{combinator::map_res, number::complete::be_u8};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{block_parsers::hierarchy::HierarchyParseErrorKind, FstParsable};

#[derive(Debug, Clone, PartialEq, Primitive, Serialize, Copy)]
#[repr(u8)]
pub enum ScopeType {
    VcdModule = 0,
    VcdTask = 1,
    VcdFunction = 2,
    VcdBegin = 3,
    VcdFork = 4,
    VcdGenerate = 5,
    VcdStruct = 6,
    VcdUnion = 7,
    VcdClass = 8,
    VcdInterface = 9,
    VcdPackage = 10,
    VcdProgram = 11,
    VhdlArchitecture = 12,
    VhdlProcedure = 13,
    VhdlFunction = 14,
    VhdlRecord = 15,
    VhdlProcess = 16,
    VhdlBlock = 17,
    VhdlGorGenerate = 18,
    VhdlIfGenerate = 19,
    VhdlGenerate = 20,
    VhdlPackage = 21,
    GenAttrBegin = 252,
    GenAttrEnd = 253,
    VcdScope = 254,
    VcdUnScope = 255,
}

impl FstParsable for ScopeType {
    fn parse(input: &[u8]) -> crate::error::FstFileResult<'_, Self> {
        map_res(be_u8, |v| {
            ScopeType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongScopeType(v)))
        })(input)
    }
}
