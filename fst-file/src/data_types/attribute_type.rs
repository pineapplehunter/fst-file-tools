use enum_primitive_derive::Primitive;
use serde::Serialize;

/// Types of attributes in [crate::block_parsers::hierarchy]
#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
#[repr(u8)]
pub enum AttributeType {
    Misc = 0,
    Array = 1,
    Enum = 2,
    Pack = 3,
}
