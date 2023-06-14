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
    let blocks = fst_file::parse(&content).unwrap();
    dbg!(&blocks);

    let header = blocks.header;
    let header_content = header.unwrap().get_content().unwrap();
    dbg!(&header_content);
    dbg!(serde_json::to_string(&header_content).unwrap());

    let geometry = blocks.geometry.unwrap();
    let geometry_content = geometry.get_content().unwrap();
    dbg!(&geometry_content);
    dbg!(serde_json::to_string(&geometry_content).unwrap());

    let hierarchy = blocks.hierarchy.unwrap();
    let hierarchy_content = hierarchy.get_content().unwrap();
    dbg!(&hierarchy_content);
    dbg!(serde_json::to_string(&hierarchy_content).unwrap());

    let blackout = blocks.blackout.unwrap();
    let blackout_content = blackout.get_content().unwrap();
    dbg!(&blackout_content);
    dbg!(serde_json::to_string(&blackout_content).unwrap());
}
