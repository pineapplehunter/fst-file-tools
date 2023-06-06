use std::fmt::{self, Debug};

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
    HierarchyGz = 4,
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

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockType::Header => Debug::fmt(&self, f),
            BlockType::ValueChangeData => write!(f, "Value Change Data (Gzip)"),
            BlockType::Blackout => Debug::fmt(&self, f),
            BlockType::Geometry => Debug::fmt(&self, f),
            BlockType::HierarchyGz => write!(f, "Hierarchy (Gzip)"),
            BlockType::ValueChangeDataAlias => write!(f, "Value Change Data (alias)"),
            BlockType::HierarchyLz4 => write!(f, "Hierarchy (Lz4)"),
            BlockType::HierarchyLz4Duo => write!(f, "Hierarchy (Lz4 x2)"),
            BlockType::ValueChangeDataAlias2 => write!(f, "Value Change Data (alias 2)"),
            BlockType::GZippedWrapper => write!(f, "Gzipped FST File"),
            BlockType::Skip => Debug::fmt(&self, f),
        }
    }
}

pub fn parse_block_type<'a>(input: &'a [u8]) -> FstFileResult<'a, BlockType> {
    map_res(take(1u32), |data: &[u8]| {
        BlockType::from_u8(data[0]).ok_or((data, BlockParseError::BlockTypeUnknown(data[0])))
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
