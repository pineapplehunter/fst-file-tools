use enum_primitive_derive::Primitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive)]
#[repr(u8)]
pub enum FileType {
    #[doc(alias = "FST_FT_VERILOG")]
    FstFtVerilog = 0,
    #[doc(alias = "FST_FT_VHDL")]
    FstFtVhdl = 1,
    #[doc(alias = "FST_FT_VERILOG_VHDL")]
    FstFtVerilogVhdl = 2,
}
