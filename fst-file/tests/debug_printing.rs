use std::{fs::File, io::Read};

fn get_test_file_content() -> Vec<u8> {
    let mut v = Vec::new();
    let mut file = File::open("tests/sample.fst").unwrap();
    file.read_to_end(&mut v).unwrap();
    v
}

#[test]
fn print_debug() {
    let content = get_test_file_content();
    let blocks = fst_file::parse_file(&content).unwrap();

    let header = blocks.get_header_block().unwrap();
    let header_content = header.get_content().as_ref().unwrap();
    dbg!(&blocks);
    dbg!(serde_json::to_string(&blocks).unwrap());

    dbg!(header_content);
    dbg!(serde_json::to_string(header_content).unwrap());

    let geometry = blocks.get_geometry_block().unwrap();
    let geometry_content = geometry.get_geometry().as_ref().unwrap();
    dbg!(&geometry_content);
    dbg!(serde_json::to_string(geometry_content).unwrap());

    let hierarchy = blocks.get_hierarchy_block().unwrap();
    let hierarchy_content = hierarchy.get_hierarchy().as_ref().unwrap();
    dbg!(hierarchy_content);
    dbg!(serde_json::to_string(hierarchy_content).unwrap());

    let blackout = blocks.get_blackout_block().unwrap();
    let blackout_content = blackout.get_content().as_ref().unwrap();
    dbg!(blackout_content);
    dbg!(serde_json::to_string(blackout_content).unwrap());
}
