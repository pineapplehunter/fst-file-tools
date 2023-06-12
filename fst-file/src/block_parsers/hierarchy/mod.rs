use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take},
        streaming::take_while_m_n,
    },
    combinator::{eof, opt},
    error::{ErrorKind, ParseError, VerboseError, VerboseErrorKind},
    multi::many_till,
    Finish, Offset,
};
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, debug_span, trace, warn};

mod attribute_type;
mod misc_type;
mod scope_type;
mod var_dir;
mod var_type;

pub use attribute_type::*;
pub use misc_type::*;
pub use scope_type::*;
pub use var_dir::*;
pub use var_type::*;

use crate::{
    data_types::{BlockType, VarInt},
    error::{ParseResult, PositionError},
    FstParsable,
};

use super::{Block, DecompressError};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Vcd {
    var_type: VarType,
    direction: VarDir,
    name: String,
    length_of_variable: VarInt,
    alias_variable_id: VarInt,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScopeBegin {
    scope_type: ScopeType,
    name: String,
    component: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Attribute {
    attr_type: AttributeType,
    misc_type: MiscType,
    name: String,
    value: VarInt,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HierarchyToken {
    Attribute(Attribute),
    AttributeEnd,
    ScopeBegin(ScopeBegin),
    ScopeEnd,
    Vcd(Vcd),
    Unknown(u8),
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
    #[error("did not start with scope")]
    DidNotStartWithScope,
    #[error("unreachable error")]
    Unreachable,
    #[error("unknown attribute type {0}")]
    WrongAttributeType(u8),
}

#[derive(Debug, Clone, Serialize)]
pub struct Scope {
    scope_type: ScopeType,
    name: String,
    component: String,
    attributes: Vec<Attribute>,
    signals: Vec<Vcd>,
    scopes: Vec<Scope>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, name: String, component: String) -> Self {
        Self {
            scope_type,
            name,
            component,
            attributes: vec![],
            signals: vec![],
            scopes: vec![],
        }
    }
}

type Span<'a> = (&'a [u8], &'a [u8]);

// impl Span {
//     fn new(from: usize, length: usize) -> Self {
//         Self { from, length }
//     }
// }

#[derive(Debug, Clone, Serialize)]
pub struct HierarchyContent {
    root_scope: Scope,
}

#[derive(Debug, Clone)]
pub struct HierarchyBlock(Block);
impl HierarchyBlock {
    pub(crate) fn from_block(block: Block) -> HierarchyBlock {
        Self(block)
    }

    pub fn get_content(&self) -> Result<HierarchyContent, HierarchyBlockConvertError> {
        let _span = debug_span!("get content").entered();
        let tokens = self.get_tokens()?;

        let scope = HierarchyContent::parse_structual_hierarchy(&tokens)
            .finish()
            .map(|(_, v)| v)
            .unwrap();

        Ok(HierarchyContent { root_scope: scope })
    }

    pub fn get_tokens(
        &self,
    ) -> Result<Vec<(PosistionAndSize, HierarchyToken)>, HierarchyBlockConvertError> {
        let _scope = debug_span!("get tokens").entered();
        if !matches!(
            self.0.block_type,
            BlockType::HierarchyGz | BlockType::HierarchyLz4 | BlockType::HierarchyLz4Duo
        ) {
            return Err(HierarchyBlockConvertError::NotHierarchyBlock);
        }

        let uncompressed_data = self.0.extract_data()?;

        debug!(uncompressed_data_len = uncompressed_data.len());

        let tokens = HierarchyContent::parse_tokens(&uncompressed_data)
            .finish()
            .map(|(_, tokens)| {
                tokens
                    .into_iter()
                    .map(|((start, end), token)| {
                        (
                            PosistionAndSize {
                                position: uncompressed_data[..].offset(start),
                                size: start.offset(end),
                            },
                            token,
                        )
                    })
                    .collect::<Vec<(PosistionAndSize, HierarchyToken)>>()
            })
            .map_err(|e| PositionError::from_verbose_parse_error(e, &uncompressed_data))?;
        Ok(tokens)
    }
}

#[derive(Debug, Error)]
pub enum HierarchyBlockConvertError {
    #[error("not a hierarchy block")]
    NotHierarchyBlock,
    #[error("error during parse of hierarchy {0}")]
    TokenParseError(#[from] PositionError<VerboseErrorKind>),
    #[error("error during uncompressing hierarchy data: {0}")]
    DataDecompressError(#[from] DecompressError),
}

#[derive(Debug, Serialize)]
pub struct PosistionAndSize {
    pub position: usize,
    pub size: usize,
}

impl HierarchyContent {
    fn parse_tokens(input: &[u8]) -> ParseResult<Vec<(Span, HierarchyToken)>> {
        let (input, (token, _)) = many_till(
            alt((
                HierarchyContent::parse_attr_begin,
                HierarchyContent::parse_attr_end,
                HierarchyContent::parse_scope_begin,
                HierarchyContent::parse_scope_end,
                HierarchyContent::parse_vcd,
                HierarchyContent::parse_unknown,
            )),
            eof,
        )(input)?;
        Ok((input, token))
    }

    fn parse_attr_begin(input: &[u8]) -> ParseResult<'_, (Span, HierarchyToken)> {
        trace!("attr begin");
        let original_input = input;
        let (input, _) = tag(&[ScopeType::GenAttrBegin as u8])(input)?;
        let (input, attr_type) = AttributeType::parse(input)?;
        let (input, misc_type) = MiscType::parse(input)?;
        let (input, name) = c_str_up_to_512(input)?;
        let (input, value) = VarInt::parse(input)?;

        Ok((
            input,
            (
                (original_input, input),
                HierarchyToken::Attribute(Attribute {
                    attr_type,
                    misc_type,
                    name,
                    value,
                }),
            ),
        ))
    }

    fn parse_attr_end(input: &[u8]) -> ParseResult<(Span, HierarchyToken)> {
        trace!("attr end");
        let original_input = input;
        let (input, _) = tag(&[ScopeType::GenAttrEnd as u8])(input)?;
        Ok((
            input,
            ((original_input, input), HierarchyToken::AttributeEnd),
        ))
    }

    fn parse_scope_begin(input: &[u8]) -> ParseResult<(Span, HierarchyToken)> {
        trace!("scope begin");
        let original_input = input;
        let (input, _) = tag(&[ScopeType::VcdScope as u8])(input)?;
        let (input, scope_type) = ScopeType::parse(input)?;
        let (input, name) = c_str_up_to_512(input)?;
        let (input, component) = c_str_up_to_512(input)?;

        Ok((
            input,
            (
                (original_input, input),
                HierarchyToken::ScopeBegin(ScopeBegin {
                    scope_type,
                    name,
                    component,
                }),
            ),
        ))
    }

    fn parse_scope_end(input: &[u8]) -> ParseResult<(Span, HierarchyToken)> {
        trace!("scope end");
        let original_input = input;
        let (input, _) = tag(&[ScopeType::VcdUnScope as u8])(input)?;
        Ok((input, ((original_input, input), HierarchyToken::ScopeEnd)))
    }

    fn parse_vcd(input: &[u8]) -> ParseResult<(Span, HierarchyToken)> {
        trace!("vcd data");
        let original_input = input;
        let (input, var_type) = VarType::parse(input)?;
        let (input, direction) = VarDir::parse(input)?;
        let (input, name) = c_str_up_to_512(input)?;
        let (input, length_of_variable) = VarInt::parse(input)?;
        let (input, alias_variable_id) = VarInt::parse(input)?;

        Ok((
            input,
            (
                (original_input, input),
                HierarchyToken::Vcd(Vcd {
                    var_type,
                    direction,
                    name,
                    length_of_variable,
                    alias_variable_id,
                }),
            ),
        ))
    }

    fn parse_unknown(input: &[u8]) -> ParseResult<(Span, HierarchyToken)> {
        let original_input = input;
        let (input, b) = take(1u8)(input)?;
        warn!("unknown byte while parsing hierarchy");
        Ok((
            input,
            ((original_input, input), HierarchyToken::Unknown(b[0])),
        ))
    }

    fn parse_structual_hierarchy(
        input: &[(PosistionAndSize, HierarchyToken)],
    ) -> ParseResult<Scope, [(PosistionAndSize, HierarchyToken)]> {
        let (input, t) = scope_begin(input)?;
        let HierarchyToken::ScopeBegin(ScopeBegin { scope_type, name, component }) = t else {unreachable!()};
        let mut scope = Scope::new(*scope_type, name.clone(), component.clone());

        let mut input = input;
        loop {
            let (input_t, t) = opt(attr_begin)(input)?;
            if let Some(HierarchyToken::Attribute(attribute)) = t {
                input = input_t;
                scope.attributes.push(attribute.clone());
                continue;
            }

            let (input_t, t) = opt(attr_end)(input)?;
            if let Some(HierarchyToken::AttributeEnd) = t {
                input = input_t;
                continue;
            }

            let (input_t, t) = opt(vcd)(input)?;
            if let Some(HierarchyToken::Vcd(vcd)) = t {
                input = input_t;
                scope.signals.push(vcd.clone());
                continue;
            }

            let (input_t, s) = opt(unknown)(input)?;
            if let Some(HierarchyToken::Unknown(u)) = s {
                input = input_t;
                warn!("ignoring unknown token {}", u);
                continue;
            }

            let (input_t, s) = opt(Self::parse_structual_hierarchy)(input)?;
            if let Some(s) = s {
                input = input_t;
                scope.scopes.push(s);
                continue;
            }

            let (input_t, s) = opt(scope_end)(input)?;
            if let Some(HierarchyToken::ScopeEnd) = s {
                input = input_t;
                break;
            }
            return Err(nom::Err::Error(VerboseError::from_error_kind(
                input,
                ErrorKind::IsNot,
            )));
        }
        Ok((input, scope))
    }
}

fn attr_begin(
    input: &[(PosistionAndSize, HierarchyToken)],
) -> ParseResult<&HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::Attribute(_)))(input)
}

fn attr_end(
    input: &[(PosistionAndSize, HierarchyToken)],
) -> ParseResult<&HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    token(HierarchyToken::AttributeEnd)(input)
}

fn scope_begin(
    input: &[(PosistionAndSize, HierarchyToken)],
) -> ParseResult<&HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::ScopeBegin(_)))(input)
}

fn vcd(
    input: &[(PosistionAndSize, HierarchyToken)],
) -> ParseResult<&HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::Vcd(_)))(input)
}

fn unknown(
    input: &[(PosistionAndSize, HierarchyToken)],
) -> ParseResult<&HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::Unknown(_)))(input)
}

fn scope_end(
    input: &[(PosistionAndSize, HierarchyToken)],
) -> ParseResult<&HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    token(HierarchyToken::ScopeEnd)(input)
}

fn token<'a>(
    token: HierarchyToken,
) -> impl Fn(
    &'a [(PosistionAndSize, HierarchyToken)],
) -> ParseResult<'a, &'a HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    move |i: &[(PosistionAndSize, HierarchyToken)]| match &i[0] {
        t if t.1 == token => {
            let (t, rest) = i.split_first().unwrap();
            Ok((rest, &t.1))
        }
        _ => Err(nom::Err::Error(VerboseError::from_error_kind(
            i,
            ErrorKind::Eof,
        ))),
    }
}

fn token_condition<'a>(
    condition: impl Fn(&HierarchyToken) -> bool,
) -> impl Fn(
    &'a [(PosistionAndSize, HierarchyToken)],
) -> ParseResult<'a, &'a HierarchyToken, [(PosistionAndSize, HierarchyToken)]> {
    move |i: &'a [(PosistionAndSize, HierarchyToken)]| match &i[0] {
        t if condition(&t.1) => {
            let (t, rest) = i.split_first().unwrap();
            Ok((rest, &t.1))
        }
        _ => Err(nom::Err::Error(VerboseError::from_error_kind(
            i,
            ErrorKind::Eof,
        ))),
    }
}

fn c_str_up_to_512(input: &[u8]) -> ParseResult<'_, String> {
    let (input, raw_str) = take_while_m_n(0, 511, |c| c != 0)(input)?;
    // for the last 0
    let (input, _) = take(1u8)(input)?;
    Ok((input, String::from_utf8_lossy(raw_str).to_string()))
}
