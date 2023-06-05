use enum_primitive_derive::Primitive;
use nom::{bytes::complete::take, combinator::map_res};
use num_traits::FromPrimitive;

use crate::error::{BlockParseError, FstFileResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive)]
#[repr(u8)]
pub enum BlockType {
    #[doc(alias = "FST_BL_HDR")]
    Header = 0,
    #[doc(alias = "FST_BL_VCDATA")]
    ValueChangeData = 1,
    #[doc(alias = "FST_BL_BLACKOUT")]
    Blackout = 2,
    #[doc(alias = "FST_BL_GEOM")]
    Geometry = 3,
    #[doc(alias = "FST_BL_HIER")]
    Hierarchy = 4,
    #[doc(alias = "FST_BL_VCDATA_DYN_ALIAS")]
    ValueChangeDataAlias = 5,
    #[doc(alias = "FST_BL_HIER_LZ4")]
    HierarchyLz4 = 6,
    #[doc(alias = "FST_BL_HIER_LZ4DUO")]
    HierarchyLz4Duo = 7,
    #[doc(alias = "FST_BL_VCDATA_DYN_ALIAS2")]
    ValueChangeDataAlias2 = 8,
    #[doc(alias = "FST_BL_ZWRAPPER")]
    GZippedWrapper = 254,
    #[doc(alias = "FST_BL_SKIP")]
    Skip = 255,
}

pub fn parse_block_type(input: &[u8]) -> FstFileResult<'_, BlockType> {
    map_res(take(1u32), |data: &[u8]| {
        BlockType::from_u8(data[0]).ok_or(BlockParseError::BlockTypeUnknown(data[0]))
    })(input)
}

#[cfg(test)]
mod test {
    use nom::Finish;

    use crate::{
        data_types::{parse_block_type, BlockType},
        error::{BlockParseError, FstFileParseErrorInner},
    };

    #[test]
    fn test_parse_block_type() {
        let data = parse_block_type(&[1]);
        let empty: &[u8] = &[];
        assert_eq!(data.unwrap(), (empty, BlockType::ValueChangeData));

        let input = [250];

        let e = parse_block_type(&input).finish().err().unwrap();
        assert_eq!(e.errors.len(), 1);
        assert_eq!(
            e.errors[0].1,
            FstFileParseErrorInner::BlockParseError(BlockParseError::BlockTypeUnknown(250))
        );
    }
}
