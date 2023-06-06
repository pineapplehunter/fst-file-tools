use enum_primitive_derive::Primitive;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive)]
#[repr(u8)]
pub enum FileType {
    #[doc(alias = "FST_FT_VERILOG")]
    Verilog = 0,
    #[doc(alias = "FST_FT_VHDL")]
    Vhdl = 1,
    #[doc(alias = "FST_FT_VERILOG_VHDL")]
    VerilogVhdl = 2,
}

impl Serialize for FileType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}
