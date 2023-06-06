use std::{ffi::CString, fmt, io::Read};

use enum_primitive_derive::Primitive;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    combinator::{eof, map, map_res},
    multi::{many0, many_till},
    number::complete::{be_u64, be_u8},
};
use num_traits::FromPrimitive;

use crate::{
    data_types::{parse_varint, VarInt},
    error::{BlockParseError, FstFileResult},
};

#[derive(Clone)]
pub struct HierarchyBlock(pub Vec<HierarchyToken>);

impl fmt::Debug for HierarchyBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // f.debug_tuple("HierarchyBlock").field(&"..").finish()
        f.debug_tuple("HierarchyBlock").field(&self.0).finish()
    }
}

#[derive(Debug, Clone)]

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
    Other(u8),
}

#[derive(Debug, Clone, PartialEq, Primitive)]
#[repr(u8)]
pub enum AttributeType {
    Misc = 0,
    Array = 1,
    Enum = 2,
    Pack = 3,
}

#[derive(Debug, Clone, PartialEq, Primitive)]
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

#[derive(Debug, Clone, PartialEq, Primitive)]
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

#[derive(Debug, Clone, PartialEq, Primitive)]
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

#[derive(Debug, Clone, PartialEq, Primitive)]
pub enum VarDir {
    Implicit = 0,
    Input = 1,
    Output = 2,
    Inout = 3,
    Buffer = 4,
    Linkage = 5,
}

pub fn parse_hierarchy_gzip_block(input: &[u8]) -> FstFileResult<'_, HierarchyBlock> {
    let (input, uncompressed_length) = map_res(be_u64, |v| {
        usize::try_from(v).map_err(|_e| BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let mut decompressor = flate2::read::GzDecoder::new(input);
    let mut data = Vec::new();
    decompressor.read_to_end(&mut data).unwrap();
    assert_eq!(data.len(), uncompressed_length);

    println!("{:?}", &data[..100]);
    let (_, hierarchy) = parse_hierarchy(&data[..]).unwrap();
    Ok((input, hierarchy))
}

pub fn parse_hierarchy_lz4_block(input: &[u8]) -> FstFileResult<'_, HierarchyBlock> {
    let (input, uncompressed_length) = map_res(be_u64, |v| {
        usize::try_from(v).map_err(|_e| BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let mut data = Vec::new();
    lzzzz::lz4::decompress(input, &mut data).unwrap();
    assert_eq!(data.len(), uncompressed_length);

    let (_, hierarchy) = parse_hierarchy(&data[..]).unwrap();
    Ok((input, hierarchy))
}

pub fn parse_hierarchy_lz4_twice_block(input: &[u8]) -> FstFileResult<'_, HierarchyBlock> {
    let (input, uncompressed_length) = map_res(be_u64, |v| {
        usize::try_from(v).map_err(|_e| BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let (input, uncompressed_once_length) = map_res(be_u64, |v| {
        usize::try_from(v).map_err(|_e| BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let mut data = Vec::new();
    lzzzz::lz4::decompress(input, &mut data).unwrap();
    assert_eq!(data.len(), uncompressed_once_length);
    let mut data2 = Vec::new();
    lzzzz::lz4::decompress(&data, &mut data2).unwrap();
    assert_eq!(data2.len(), uncompressed_length);

    let (_, hierarchy) = parse_hierarchy(&data2[..]).unwrap();
    Ok((input, hierarchy))
}

fn parse_hierarchy(input: &[u8]) -> FstFileResult<'_, HierarchyBlock> {
    let (input, (token,_)) = many_till(
        alt((
            parse_attr_begin,
            parse_attr_end,
            parse_scope_begin,
            parse_scope_end,
            parse_vcd,
            parse_other,
        )),
        eof,
    )(input)?;
    Ok((input, HierarchyBlock(token)))
}

fn parse_attr_begin(input: &[u8]) -> FstFileResult<'_, HierarchyToken> {
    let (input, _) = tag(&[ScopeType::GenAttrBegin as u8])(input)?;
    let (input, attr_type) = map_res(be_u8, |v| {
        AttributeType::from_u8(v).ok_or(BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let (input, misc_type) = map_res(be_u8, |v| {
        MiscType::from_u8(v).ok_or(BlockParseError::LengthTooLargeForMachine)
    })(input)?;
    let (input, name) = map(take_while(|c| c != 0), |s| {
        CString::new(s).unwrap().to_string_lossy().to_string()
    })(input)?;
    let (input, _) = take(1u8)(input)?;
    let (input, value) = parse_varint(input)?;

    Ok((
        input,
        dbg!(HierarchyToken::Attribute {
            attr_type,
            misc_type,
            name,
            value,
        }),
    ))
}

fn parse_attr_end(input: &[u8]) -> FstFileResult<'_, HierarchyToken> {
    let (input, _) = tag(&[ScopeType::GenAttrEnd as u8])(input)?;
    Ok((input, dbg!(HierarchyToken::AttributeEnd)))
}

fn parse_scope_begin(input: &[u8]) -> FstFileResult<'_, HierarchyToken> {
    let (input, _) = tag(&[ScopeType::VcdScope as u8])(input)?;
    let (input, scope_type) = map_res(be_u8, |v| {
        ScopeType::from_u8(v).ok_or(BlockParseError::BlockWrongLength)
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
        dbg!(HierarchyToken::ScopeBegin {
            scope_type,
            name,
            component
        }),
    ))
}

fn parse_scope_end(input: &[u8]) -> FstFileResult<'_, HierarchyToken> {
    let (input, _) = tag(&[ScopeType::VcdUnScope as u8])(input)?;
    Ok((input, dbg!(HierarchyToken::ScopeEnd)))
}

fn parse_vcd(input: &[u8]) -> FstFileResult<'_, HierarchyToken> {
    let (input, var_type) = map_res(be_u8, |v| {
        VarType::from_u8(v).ok_or(BlockParseError::BlockWrongLength)
    })(input)?;
    let (input, direction) = map_res(be_u8, |v| {
        VarDir::from_u8(v).ok_or(BlockParseError::BlockWrongLength)
    })(input)?;
    let (input, name) = map(take_while(|c| c != 0), |s| {
        CString::new(s).unwrap().to_string_lossy().to_string()
    })(input)?;
    let (input, _) = take(1u8)(input)?;
    let (input, length_of_variable) = parse_varint(input)?;
    let (input, alias_variable_id) = parse_varint(input)?;

    Ok((
        input,
        dbg!(HierarchyToken::Vcd {
            var_type,
            direction,
            name,
            length_of_variable,
            alias_variable_id
        }),
    ))
}

fn parse_other(input: &[u8]) -> FstFileResult<'_, HierarchyToken> {
    let (input, b) = take(1u8)(input)?;
    Ok((input, HierarchyToken::Other(b[0])))
}
