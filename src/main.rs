use fst_file::varint::{VarInt, SVarInt};

fn main() {
    println!("Hello, world!");
    let a = VarInt::from(2);
    println!("{a:X?}");
    let a = VarInt::from(255);
    println!("{a:X?}");
    let a = VarInt::from(3141);
    println!("{a:X?}");
    let a = VarInt::from(3141);
    println!("{a:X?}");
    let a = SVarInt::from(-15429);
    println!("{a:X?}");
    let a = SVarInt::from(-3);
    println!("{a:X?}");
}

