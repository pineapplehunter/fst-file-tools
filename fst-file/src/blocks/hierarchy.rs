use std::{cell::OnceCell, ffi::CString};

use enum_primitive_derive::Primitive;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    combinator::{eof, map, map_res},
    multi::many_till,
    number::complete::be_u8,
    Finish, Offset,
};
use num_traits::FromPrimitive;
use serde::Serialize;
use thiserror::Error;
use tracing::warn;

use crate::{
    data_types::{parse_varint, VarInt},
    error::{BlockParseError, FstFileParseError, FstFileResult},
};

use super::Block;

#[derive(Clone)]
pub struct HierarchyBlock<'a> {
    block: &'a Block<'a>,
    uncompressed_data: OnceCell<Vec<u8>>,
    tokens: OnceCell<Result<Vec<(Span, HierarchyToken)>, FstFileParseError<&'a [u8]>>>,
}

impl<'a> HierarchyBlock<'a> {
    pub(crate) fn from_block(block: &'a Block) -> Self {
        Self {
            block,
            uncompressed_data: OnceCell::new(),
            tokens: OnceCell::new(),
        }
    }

    fn get_uncompressed_data_cache(&self) -> &[u8] {
        self.uncompressed_data
            .get_or_init(|| self.block.extract_data())
    }

    fn get_tokens_cache(
        &'a self,
    ) -> &Result<Vec<(Span, HierarchyToken)>, FstFileParseError<&[u8]>> {
        self.tokens.get_or_init(|| {
            let data = self.get_uncompressed_data_cache();
            self.parse_tokens(data).finish().map(|(_, tokens)| tokens)
        })
    }

    pub fn offset_from_uncompressed_data(&self, data: &[u8]) -> usize {
        self.get_uncompressed_data_cache().offset(data)
    }

    pub fn get_tokens(&'a self) -> &Result<Vec<(Span, HierarchyToken)>, FstFileParseError<&[u8]>> {
        self.get_tokens_cache()
    }

    fn parse_tokens(&'a self, input: &'a [u8]) -> FstFileResult<'_, Vec<(Span, HierarchyToken)>> {
        let (input, (token, _)) = many_till(
            alt((
                |i| self.parse_attr_begin(i),
                |i| self.parse_attr_end(i),
                |i| self.parse_scope_begin(i),
                |i| self.parse_scope_end(i),
                |i| self.parse_vcd(i),
                |i| self.parse_unknown(i),
            )),
            eof,
        )(input)?;
        Ok((input, token))
    }

    fn parse_attr_begin(&'a self, input: &'a [u8]) -> FstFileResult<'_, (Span, HierarchyToken)> {
        let original_input = input;
        let (input, _) = tag(&[ScopeType::GenAttrBegin as u8])(input)?;
        let (input, attr_type) = map_res(be_u8, |v| {
            AttributeType::from_u8(v).ok_or((input, BlockParseError::LengthTooLargeForMachine))
        })(input)?;
        let (input, misc_type) = map_res(be_u8, |v| {
            MiscType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongMiscType(v)))
        })(input)?;
        let (input, name) = map(take_while(|c| c != 0), |s| {
            CString::new(s).unwrap().to_string_lossy().to_string()
        })(input)?;
        let (input, _) = take(1u8)(input)?;
        let (input, value) = parse_varint(input)?;

        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::Attribute {
                    attr_type,
                    misc_type,
                    name,
                    value,
                },
            ),
        ))
    }

    fn parse_attr_end(&'a self, input: &'a [u8]) -> FstFileResult<'_, (Span, HierarchyToken)> {
        let original_input = input;
        let (input, _) = tag(&[ScopeType::GenAttrEnd as u8])(input)?;
        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::AttributeEnd,
            ),
        ))
    }

    fn parse_scope_begin(&'a self, input: &'a [u8]) -> FstFileResult<'_, (Span, HierarchyToken)> {
        let original_input = input;
        let (input, _) = tag(&[ScopeType::VcdScope as u8])(input)?;
        let (input, scope_type) = map_res(be_u8, |v| {
            ScopeType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongScopeType(v)))
        })(input)?;
        let (input, name) = map(take_while(|c| c != 0), |s| {
            CString::new(s).unwrap().to_string_lossy().to_string()
        })(input)?;
        let (input, _) = take(1u8)(input)?;
        let (input, component) = map(take_while(|c| c != 0), |s| {
            CString::new(s).unwrap().to_string_lossy().to_string()
        })(input)?;
        let (input, _) = take(1u8)(input)?;

        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::ScopeBegin {
                    scope_type,
                    name,
                    component,
                },
            ),
        ))
    }

    fn parse_scope_end(&'a self, input: &'a [u8]) -> FstFileResult<'_, (Span, HierarchyToken)> {
        let original_input = input;
        let (input, _) = tag(&[ScopeType::VcdUnScope as u8])(input)?;
        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::ScopeEnd,
            ),
        ))
    }

    fn parse_vcd(&'a self, input: &'a [u8]) -> FstFileResult<'_, (Span, HierarchyToken)> {
        let original_input = input;
        let (input, var_type) = map_res(be_u8, |v| {
            VarType::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongVarType(v)))
        })(input)?;
        let (input, direction) = map_res(be_u8, |v| {
            VarDir::from_u8(v).ok_or((input, HierarchyParseErrorKind::WrongVarDir(v)))
        })(input)?;
        let (input, name) = map(take_while(|c| c != 0), |s| {
            CString::new(s).unwrap().to_string_lossy().to_string()
        })(input)?;
        let (input, _) = take(1u8)(input)?;
        let (input, length_of_variable) = parse_varint(input)?;
        let (input, alias_variable_id) = parse_varint(input)?;

        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::Vcd {
                    var_type,
                    direction,
                    name,
                    length_of_variable,
                    alias_variable_id,
                },
            ),
        ))
    }

    fn parse_unknown(&'a self, input: &'a [u8]) -> FstFileResult<'_, (Span, HierarchyToken)> {
        let original_input = input;
        let (input, b) = take(1u8)(input)?;
        warn!("unknown byte while parsing hierarchy");
        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::Unknown(b[0]),
            ),
        ))
    }

    fn make_span(&self, start: &[u8], end: &[u8]) -> Span {
        let from = self.get_uncompressed_data_cache().offset(start);
        let length = start.offset(end);
        Span::new(from, length)
    }
}

#[derive(Debug, Clone, Serialize)]

pub enum HierarchyToken {
    Attribute {
        attr_type: AttributeType,
        misc_type: MiscType,
        name: String,
        value: VarInt,
    },
    AttributeEnd,
    ScopeBegin {
        scope_type: ScopeType,
        name: String,
        component: String,
    },
    ScopeEnd,
    Vcd {
        var_type: VarType,
        direction: VarDir,
        name: String,
        length_of_variable: VarInt,
        alias_variable_id: VarInt,
    },
    Unknown(u8),
}

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
#[repr(u8)]
pub enum AttributeType {
    Misc = 0,
    Array = 1,
    Enum = 2,
    Pack = 3,
}

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

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
pub enum VarDir {
    Implicit = 0,
    Input = 1,
    Output = 2,
    Inout = 3,
    Buffer = 4,
    Linkage = 5,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum HierarchyParseErrorKind {
    #[error("misc type was wrong on attribute. the value was {0}")]
    WrongMiscType(u8),
    #[error("scope type was wrong on scope. the value was {0}")]
    WrongScopeType(u8),
    #[error("var type was wrong. the value was {0}")]
    WrongVarType(u8),
    #[error("var dir was wrong. the value was {0}")]
    WrongVarDir(u8),
}

#[derive(Debug, Serialize, Clone)]
pub struct Span {
    pub from: usize,
    pub length: usize,
}

impl Span {
    fn new(from: usize, length: usize) -> Self {
        Self { from, length }
    }
}
