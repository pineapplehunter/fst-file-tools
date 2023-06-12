// use nom::Offset;
// use serde::{ser::SerializeSeq, Serialize};

// use crate::{
//     block_parsers::{
//         blackout::BlackoutBlock, geometry::GeometryBlock, header::HeaderBlock,
//         hierarchy::HierarchyBlock, value_change_data::ValueChangeDataBlock, Block,
//     },
//     data_types::BlockType,
// };

// use super::BlockInfo;

// impl Blocks {
//     pub fn get(&self, index: usize) -> Option<BlockInfo<'_>> {
//         self.blocks.get(index).map(|block| {
//             BlockInfo::from_offset_and_block(
//                 self.start_of_input.offset(block.get_data_raw()) - 9,
//                 block,
//             )
//         })
//     }

//     pub fn iter(&self) -> BlocksIter {
//         BlocksIter {
//             index: 0,
//             blocks: self,
//         }
//     }

//     pub fn get_header_block(&self) -> Option<HeaderBlock> {
//         self.blocks
//             .iter()
//             .find(|b| b.block_type == BlockType::Header)
//             .map(HeaderBlock::from_block)
//     }

//     pub fn get_hierarchy_block(&self) -> Option<HierarchyBlock> {
//         self.blocks
//             .iter()
//             .find(|b| {
//                 matches!(
//                     b.block_type,
//                     BlockType::HierarchyGz | BlockType::HierarchyLz4 | BlockType::HierarchyLz4Duo
//                 )
//             })
//             .map(HierarchyBlock::from_block)
//     }

//     pub fn get_geometry_block(&self) -> Option<GeometryBlock> {
//         self.blocks
//             .iter()
//             .find(|b| b.block_type == BlockType::Geometry)
//             .map(GeometryBlock::from_block)
//     }

//     pub fn get_blackout_block(&self) -> Option<BlackoutBlock> {
//         self.blocks
//             .iter()
//             .find(|b| b.block_type == BlockType::Blackout)
//             .map(BlackoutBlock::from_block)
//     }

//     pub fn get_value_change_data_blocks<'a>(&'a self) -> impl Iterator<Item = ValueChangeDataBlock<'a>> {
//         let header_num_vars = self
//             .get_header_block()
//             .unwrap()
//             .get_content()
//             .as_ref()
//             .unwrap()
//             .num_vars;
//         self.blocks
//             .iter()
//             .filter(|b| {
//                 matches!(
//                     b.block_type,
//                     BlockType::ValueChangeDataAlias2
//                         | BlockType::ValueChangeDataAlias
//                         | BlockType::ValueChangeData
//                 )
//             })
//             .map(move |block| ValueChangeDataBlock::from_block(block, header_num_vars))
//     }
// }
