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
    fst_file::parse_file(&content).unwrap();
}

#[test]
fn parse_header() {
    let content = get_test_file_content();
    let blocks = fst_file::parse_file(&content).unwrap();
    let header = blocks.get_header_block().unwrap();
    header.get_content().as_ref().unwrap();
}

#[test]
fn parse_geometry() {
    let content = get_test_file_content();
    let blocks = fst_file::parse_file(&content).unwrap();
    let geometry = blocks.get_geometry_block().unwrap();
    geometry.get_geometry().as_ref().unwrap();
}

#[test]
fn parse_hierarchy() {
    let content = get_test_file_content();
    let blocks = fst_file::parse_file(&content).unwrap();
    let hierarchy = blocks.get_hierarchy_block().unwrap();
    hierarchy.get_hierarchy().as_ref().unwrap();
}

#[test]
fn parse_blackout() {
    let content = get_test_file_content();
    let blocks = fst_file::parse_file(&content).unwrap();
    let blackout = blocks.get_blackout_block().unwrap();
    blackout.get_content().as_ref().unwrap();
}
