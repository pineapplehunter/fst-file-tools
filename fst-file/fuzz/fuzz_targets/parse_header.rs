#![no_main]

use libfuzzer_sys::fuzz_target;
use fst_file::block_parsers::Block;

fuzz_target!(|data: &[u8]| {
    let _ = Block::parse_block_with_position(data);
});
