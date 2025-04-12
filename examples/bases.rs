use hexd::{options::HexdOptionsBuilder, AsHexd};

fn main() {
    let v = vec![0x2u8; 32];
    let v = std::fs::read("./src/lib.rs").unwrap();

    // v.hexd()
    //     .hexadecimal()
    //     .dump();
    // println!();

    v.hexd().decimal().dump();
    println!();

    // v.hexd()
    //     .octal()
    //     .dump();
    // println!();

    // v.hexd()
    //     .binary()
    //     .dump();
}
