use enum_primitive_derive::Primitive;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Primitive, Serialize)]
pub enum VarDir {
    Implicit = 0,
    Input = 1,
    Output = 2,
    Inout = 3,
    Buffer = 4,
    Linkage = 5,
}
