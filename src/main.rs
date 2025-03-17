use std::{cmp::min, env::set_current_dir, fmt::{Arguments, Debug}, fs, io::{BufRead, Write}, ops::{Bound, Deref, Range, RangeBounds}, str};

use hexdump::{hexdump_into_rr, HexdumpIoWriter, HexdumpOptions, HexdumpRange, MyByteReader, SliceGroupedByteReader, SliceGroupedReader, WriteHexdump};

mod hexdump;

fn vec_000() -> Vec<u8> {
    let mut o: Vec<u8> = vec![0u8; 17];
    o
}

fn vec_001() -> Vec<u8> {
    let mut f = vec![0u8; 4];
    f.extend_from_slice(&[0x11u8; 96]);
    f.extend_from_slice(&[0x22u8; 4092]);
    f.extend_from_slice(&[0x33u8; 27]);
    f
}

fn vec_main_rs() -> Vec<u8> {
    let main_file = fs::read("./src/main.rs").unwrap();
    main_file
}

fn dump_this(slice: &[u8]) {
    let mut sg = SliceGroupedByteReader::new(slice, hexdump::Endianness::BigEndian);
    let mut writer = HexdumpIoWriter(std::io::stdout());
    hexdump_into_rr(
        &mut writer, 
        &mut sg, HexdumpOptions {
            omit_equal_rows: true,
            print_range: HexdumpRange::new(0xc0..0xf0),
            grouping: hexdump::Grouping::Grouped { group_size: hexdump::GroupSize::Int, num_groups: 4, byte_spacing: hexdump::Spacing::None, group_spacing: hexdump::Spacing::Normal},
            ..Default::default()
        }).unwrap();
}

// Start
// bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
// aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
// End


fn main() {
    dump_this(&vec_main_rs());
    // dbg!(usize::BITS);
    // f.extend_from_slice(&[0xaabb88ffu32; 20]);

    // let mut sgbr = SliceGroupedByteReader::new(&f, hexdump::Endianness::LittleEndian);
    // loop {
    //     let mut s_into = [0u8; 16];
    //     let s = sgbr.next_bytes(&mut s_into);
    //     if s.len() == 0 { break; }
    //     dbg!(s);
    // }
}
