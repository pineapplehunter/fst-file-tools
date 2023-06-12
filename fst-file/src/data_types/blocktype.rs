use std::fmt::{self, Debug};

use enum_primitive_derive::Primitive;
use nom::{bytes::complete::take, combinator::map_res, error::context};
use num_traits::FromPrimitive;
use serde::Serialize;

use crate::{error::ParseResult, FstParsable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive, Serialize, Hash, PartialOrd, Ord)]
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
            BlockType::Header => f.pad("Header"),
            BlockType::ValueChangeData => f.pad("Value Change Data (Gzip)"),
            BlockType::Blackout => f.pad("Blackout"),
            BlockType::Geometry => f.pad("Geometry"),
            BlockType::HierarchyGz => f.pad("Hierarchy (Gzip)"),
            BlockType::ValueChangeDataAlias => f.pad("Value Change Data (alias)"),
            BlockType::HierarchyLz4 => f.pad("Hierarchy (Lz4)"),
            BlockType::HierarchyLz4Duo => f.pad("Hierarchy (Lz4 x2)"),
            BlockType::ValueChangeDataAlias2 => f.pad("Value Change Data (alias 2)"),
            BlockType::GZippedWrapper => f.pad("Gzipped FST File"),
            BlockType::Skip => f.pad("Skip"),
        }
    }
}

impl FstParsable for BlockType {
    fn parse(input: &[u8]) -> ParseResult<'_, Self> {
        context(
            "block type",
            map_res(take(1u32), |data: &[u8]| {
                Self::from_u8(data[0]).ok_or((input, std::num::IntErrorKind::InvalidDigit))
            }),
        )(input)
    }
}

#[cfg(test)]
mod test {
    use nom::{error::VerboseErrorKind, Finish};

    use crate::{data_types::BlockType, FstParsable};

    #[test]
    fn test_parse_block_type() {
        let data = BlockType::parse(&[1]);
        let empty: &[u8] = &[];
        assert_eq!(data.unwrap(), (empty, BlockType::ValueChangeData));

        let input = [250];
        let e = BlockType::parse(&input).finish().err().unwrap();
        assert_eq!(e.errors.len(), 2);
        assert_eq!(e.errors[1].1, VerboseErrorKind::Context("block type"));
    }
}
