use std::{fs::File, io::Read};

fn get_test_file_content() -> Vec<u8> {
    let mut v = Vec::new();
    let mut file = File::open("tests/sample.fst").unwrap();
    file.read_to_end(&mut v).unwrap();
    v
}

#[test]
fn parse_blocks() {
    let content = get_test_file_content();
    fst_file::parse_raw_block_information(&content).unwrap();
}

#[test]
fn parse_contents() {
    let content = get_test_file_content();
    fst_file::parse(&content).unwrap();
}

#[test]
fn parse_header() {
    let content = get_test_file_content();
    let blocks = fst_file::parse(&content).unwrap();
    blocks.header.get_content().unwrap();
}

#[test]
fn parse_geometry() {
    let content = get_test_file_content();
    let blocks = fst_file::parse(&content).unwrap();
    blocks.geometry.unwrap().get_content().unwrap();
}

#[test]
fn parse_hierarchy() {
    let content = get_test_file_content();
    let contents = fst_file::parse(&content).unwrap();
    let hierarchy = contents.hierarchy.unwrap();
    hierarchy.get_content().unwrap();
}

#[test]
fn parse_blackout() {
    let content = get_test_file_content();
    let blocks = fst_file::parse(&content).unwrap();
    blocks.blackout.unwrap().get_content().unwrap();
}
