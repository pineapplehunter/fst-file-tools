use enum_primitive_derive::Primitive;
use serde::Serialize;

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
