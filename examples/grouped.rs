use hxd::{options::Endianness, AsHexdGrouped};

fn main() {
    let v = vec![0xf2f0u16; 32];
    v.as_hexd_grouped(Endianness::BigEndian).dump();

    let v = vec![0x11002233u32; 32];
    v.as_hexd_grouped(Endianness::BigEndian).dump();
    v.as_hexd_grouped(Endianness::LittleEndian).dump();

    let v = vec![0xf2f00u64; 32];
    v.as_hexd_grouped(Endianness::BigEndian).dump();

    let v = vec![0xf2f00u128; 32];
    v.as_hexd_grouped(Endianness::LittleEndian).dump();
}
