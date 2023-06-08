use std::cell::OnceCell;

use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take},
        streaming::take_while_m_n,
    },
    combinator::{eof, opt},
    error::{ErrorKind, ParseError},
    multi::many_till,
    Finish, IResult, Offset,
};
use serde::Serialize;
use thiserror::Error;
use tracing::{debug_span, warn};

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
    data_types::VarInt,
    error::{FstFileParseError, FstFileResult},
    FstParsable,
};

use super::Block;

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
pub struct Scope<'a> {
    scope_type: &'a ScopeType,
    name: &'a String,
    component: &'a String,
    attributes: Vec<&'a Attribute>,
    signals: Vec<&'a Vcd>,
    scopes: Vec<Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(scope_type: &'a ScopeType, name: &'a String, component: &'a String) -> Self {
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

type TokensResult<'a> = Result<Vec<(Span, HierarchyToken)>, FstFileParseError<&'a [u8]>>;
type ScopeResult<'a> = Result<Scope<'a>, FstFileParseError<&'a [(Span, HierarchyToken)]>>;
#[derive(Clone)]
pub struct HierarchyBlock<'a> {
    block: &'a Block<'a>,
    uncompressed_data: OnceCell<Vec<u8>>,
    tokens: OnceCell<TokensResult<'a>>,
    root_scope: OnceCell<ScopeResult<'a>>,
}

impl<'a> HierarchyBlock<'a> {
    pub(crate) fn from_block(block: &'a Block) -> Self {
        Self {
            block,
            uncompressed_data: OnceCell::new(),
            tokens: OnceCell::new(),
            root_scope: OnceCell::new(),
        }
    }

    fn get_uncompressed_data_cache(&self) -> &[u8] {
        self.uncompressed_data
            .get_or_init(|| self.block.extract_data())
    }

    fn get_tokens_cache(&'a self) -> &TokensResult<'a> {
        self.tokens.get_or_init(|| {
            let _span = debug_span!("caching hierarchy tokens").entered();
            let data = self.get_uncompressed_data_cache();
            self.parse_tokens(data).finish().map(|(_, tokens)| tokens)
        })
    }

    fn get_hierarchy_cache(&'a self) -> &ScopeResult<'a> {
        self.root_scope.get_or_init(|| {
            let _span = debug_span!("caching hierarchy").entered();
            let data = self.get_tokens_cache().as_ref().unwrap();
            Self::parse_structual_hierarchy(&data[..])
                .finish()
                .map(|(_, scope)| scope)
        })
    }

    pub fn offset_from_uncompressed_data(&self, data: &[u8]) -> usize {
        self.get_uncompressed_data_cache().offset(data)
    }

    pub fn get_tokens(&'a self) -> &TokensResult<'a> {
        self.get_tokens_cache()
    }

    pub fn get_hierarchy(&'a self) -> &ScopeResult<'a> {
        self.get_hierarchy_cache()
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
        let (input, attr_type) = AttributeType::parse(input)?;
        let (input, misc_type) = MiscType::parse(input)?;
        let (input, name) = c_str_up_to_512(input)?;
        let (input, value) = VarInt::parse(input)?;

        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::Attribute(Attribute {
                    attr_type,
                    misc_type,
                    name,
                    value,
                }),
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
        let (input, scope_type) = ScopeType::parse(input)?;
        let (input, name) = c_str_up_to_512(input)?;
        let (input, component) = c_str_up_to_512(input)?;

        Ok((
            input,
            (
                self.make_span(original_input, input),
                HierarchyToken::ScopeBegin(ScopeBegin {
                    scope_type,
                    name,
                    component,
                }),
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
        let (input, var_type) = VarType::parse(input)?;
        let (input, direction) = VarDir::parse(input)?;
        let (input, name) = c_str_up_to_512(input)?;
        let (input, length_of_variable) = VarInt::parse(input)?;
        let (input, alias_variable_id) = VarInt::parse(input)?;

        Ok((
            input,
            (
                self.make_span(original_input, input),
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

    fn parse_structual_hierarchy(
        input: &'a [(Span, HierarchyToken)],
    ) -> FstFileResult<'a, Scope, &'a [(Span, HierarchyToken)]> {
        let (input, t) = scope_begin(input)?;
        let HierarchyToken::ScopeBegin(ScopeBegin { scope_type, name, component }) = t else {unreachable!()};
        let mut scope = Scope::new(scope_type, name, component);

        let mut input = input;
        loop {
            let (input_t, t) = opt(attr_begin)(input)?;
            if let Some(HierarchyToken::Attribute(attribute)) = t {
                input = input_t;
                scope.attributes.push(attribute);
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
                scope.signals.push(vcd);
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
            return Err(nom::Err::Error(
                (input, HierarchyParseErrorKind::Unreachable).into(),
            ));
        }
        Ok((input, scope))
    }
}

fn attr_begin(
    input: &[(Span, HierarchyToken)],
) -> FstFileResult<'_, &HierarchyToken, &[(Span, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::Attribute(_)))(input)
}

fn attr_end(
    input: &[(Span, HierarchyToken)],
) -> FstFileResult<'_, &HierarchyToken, &[(Span, HierarchyToken)]> {
    token(HierarchyToken::AttributeEnd)(input)
}

fn scope_begin(
    input: &[(Span, HierarchyToken)],
) -> FstFileResult<'_, &HierarchyToken, &[(Span, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::ScopeBegin(_)))(input)
}

fn vcd(
    input: &[(Span, HierarchyToken)],
) -> FstFileResult<'_, &HierarchyToken, &[(Span, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::Vcd(_)))(input)
}

fn unknown(
    input: &[(Span, HierarchyToken)],
) -> FstFileResult<'_, &HierarchyToken, &[(Span, HierarchyToken)]> {
    token_condition(|t| matches!(t, HierarchyToken::Unknown(_)))(input)
}

fn scope_end(
    input: &[(Span, HierarchyToken)],
) -> FstFileResult<'_, &HierarchyToken, &[(Span, HierarchyToken)]> {
    token(HierarchyToken::ScopeEnd)(input)
}

fn token<'a, Error: ParseError<&'a [(Span, HierarchyToken)]>>(
    token: HierarchyToken,
) -> impl Fn(&'a [(Span, HierarchyToken)]) -> IResult<&'a [(Span, HierarchyToken)], &HierarchyToken, Error>
{
    move |i: &'a [(Span, HierarchyToken)]| match &i[0] {
        t if t.1 == token => {
            let (t, rest) = i.split_first().unwrap();
            Ok((rest, &t.1))
        }
        _ => Err(nom::Err::Error(Error::from_error_kind(i, ErrorKind::Eof))),
    }
}

fn token_condition<'a, Error: ParseError<&'a [(Span, HierarchyToken)]>>(
    condition: impl Fn(&'a HierarchyToken) -> bool,
) -> impl Fn(
    &'a [(Span, HierarchyToken)],
) -> IResult<&'a [(Span, HierarchyToken)], &'a HierarchyToken, Error> {
    move |i: &'a [(Span, HierarchyToken)]| match &i[0] {
        t if condition(&t.1) => {
            let (t, rest) = i.split_first().unwrap();
            Ok((rest, &t.1))
        }
        _ => Err(nom::Err::Error(Error::from_error_kind(i, ErrorKind::Eof))),
    }
}

fn c_str_up_to_512(input: &[u8]) -> FstFileResult<'_, String> {
    let (input, raw_str) = take_while_m_n(0, 511, |c| c != 0)(input)?;
    // for the last 0
    let (input, _) = take(1u8)(input)?;
    Ok((input, String::from_utf8_lossy(raw_str).to_string()))
}
