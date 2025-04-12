use hxd::{options::HexdOptionsBuilder, AsHexd, AsHexdGrouped};

fn main() {
    let v = concat!(
        "The Nellie, a cruising yawl, swung to her anchor without a flutter of the sails, ",
        "and was at rest. The flood had made, the wind was nearly calm, and being bound ",
        "down the river, the only thing for it was to come to and wait for the turn of the tide."
    );

    // Dumping is as simple as this.
    v.hexd().dump();

    println!();

    // Hexd can write a hexdump into any type
    // that is `WriteHexdump`.`
    v.hexd().range(0x23..0x5c).aligned(true).dump();

    println!();
    x();
}

fn x() {
    vec![0x6120u16; 8].as_hexd_be().dump();
    vec![0x7fa06120i32; 4].as_hexd_be().dump();
    vec![0xff3007fa06120u64; 2].as_hexd_le().dump();
    vec![0x7fa06120u128; 1].as_hexd_be().dump();
}
