use enum_primitive_derive::Primitive;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
#[repr(u8)]
pub enum AttributeType {
    Misc = 0,
    Array = 1,
    Enum = 2,
    Pack = 3,
}
