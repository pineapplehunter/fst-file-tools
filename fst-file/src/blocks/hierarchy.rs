// use crate::data_types::{BlockType, FileType};

// #[derive(Debug)]
// #[repr(packed)]
// pub struct HierarchyBlock<'a> {
//     pub block_type: BlockType,
//     pub length: u64,
//     pub uncompressed_length: u64,
//     pub data: Cow<'a, u8>,
// }

// #[derive(Debug)]
// #[repr(packed)]
// pub struct HierarchyLz4DuoBlock<'a> {
//     pub block_type: BlockType,
//     pub length: u64,
//     pub uncompressed_length: u64,
//     pub compressed_once_length: u64,
//     pub data: Cow<'a, u8>,
// }