use std::{fs::File, io::Read};

use fst_file::{blocks::parse_block, parse_file};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::open("./sample.fst")?;
    let mut b = Vec::new();
    f.read_to_end(&mut b)?;

    let (_input, s) = parse_block(&b).unwrap();
    // println!("{input:?}");
    println!("{s:#?}");

    let s = parse_file(&b).unwrap();
    println!("{s}");

    Ok(())
}
