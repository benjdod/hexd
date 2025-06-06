use hxd::{options::HexdOptionsBuilder, AsHexd};

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
    // that is `WriteHexdump`.
    v.hexd().range(0x23..0x5c).aligned(true).dump_to::<String>();
}
