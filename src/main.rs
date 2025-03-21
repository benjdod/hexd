use std::{cmp::min, env::set_current_dir, fmt::{Arguments, Debug}, fs, io::{BufRead, Write}, ops::{Bound, Deref, Range, RangeBounds}, str};

use hexdump::{hexdump_into_rr, ByteSliceReader, HexdumpIoWriter, HexdumpLineIterator, HexdumpLineWriter, HexdumpOptions, HexdumpRange, MyByteReader, SliceGroupedByteReader, SliceGroupedReader, WriteHexdump};

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

fn vec_long_empty() -> Vec<u8> {
    let mut o = vec![0u8; 4096];
    o.extend_from_slice(&[0x11u8; 4096]);
    o.extend_from_slice(&[0x22u8; 4096]);
    o
}

fn vec_main_rs() -> Vec<u8> {
    let main_file = fs::read("./src/main.rs").unwrap();
    main_file
}

fn vec_inc() -> Vec<u8> {
    let mut o = vec![0u8; 16];
    o.extend_from_slice(&[0x1u8; 16]);
    o.extend_from_slice(&[0x22u8; 16]);
    o.extend_from_slice(&[0x33u8; 16 * 16]);
    o.extend_from_slice(&[0x44u8; 16]);
    o.extend_from_slice(&[0x55u8; 16*8]);
    o.extend_from_slice(&[0x66u8; 16]);
    o
}

fn dump_this(slice: &[u8]) {
    // let mut sg = SliceGroupedByteReader::new(slice, hexdump::Endianness::BigEndian);
    let mut sg = ByteSliceReader::new(slice);
    let mut writer = HexdumpIoWriter(std::io::stdout());
    hexdump_into_rr::<_, _, 4096>(
        &mut writer, 
        &mut sg, HexdumpOptions {
            omit_equal_rows: true,
            print_range: HexdumpRange::new(0xc00..),
            grouping: hexdump::Grouping::Grouped { group_size: hexdump::GroupSize::Int, num_groups: 4, byte_spacing: hexdump::Spacing::None, group_spacing: hexdump::Spacing::Normal},
            ..Default::default()
        }).unwrap();
}

fn dump_this_with_hli(slice: &[u8]) {
    let mut writer = HexdumpIoWriter(std::io::stdout());

    let mut hww = HexdumpLineWriter::new(ByteSliceReader::new(slice), writer, HexdumpOptions {
        print_range: HexdumpRange::new(..),
        omit_equal_rows: true,
        uppercase: false,
        align: false,
        grouping: hexdump::Grouping::Grouped { group_size: hexdump::GroupSize::Int, num_groups: 4, byte_spacing: hexdump::Spacing::None, group_spacing: hexdump::Spacing::Wide },
        ..Default::default()
    });

    hww.do_hexdump();
}

// Start
// bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
// aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
// End


fn main() {
    // println!("{:#x}", usize::MAX);
    dump_this_with_hli(&&vec_inc());
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
