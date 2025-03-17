/*
Design a b-tree that is stored on disk.
This must have an ordered key and handle storing records of variable lengths.

The b-tree will be stored in pages of length 4096. Each page will be either a
 - leaf page containing one or more records
 - interior page containing references to leaf pages and 
   their corresponding ranges
*/

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

fn dump_this(slice: &[u8]) {
    let mut sg = SliceGroupedByteReader::new(slice, hexdump::Endianness::BigEndian);
    let mut writer = HexdumpIoWriter(std::io::stdout());
    hexdump_into_rr(
        &mut writer, 
        &mut sg, HexdumpOptions {
            omit_equal_rows: true,
            // group_size: hexdump::GroupSize::Int,
            print_range: HexdumpRange {
               skip: 10,
               limit: None
            },
            grouping: hexdump::Grouping::Grouped { group_size: hexdump::GroupSize::Short, num_groups: 8, byte_spacing: hexdump::Spacing::None, group_spacing: hexdump::Spacing::Normal},
            ..Default::default()
        }).unwrap();
}

fn main() {
    dump_this(&vec_000());
    // f.extend_from_slice(&[0xaabb88ffu32; 20]);

    // let mut sgbr = SliceGroupedByteReader::new(&f, hexdump::Endianness::LittleEndian);
    // loop {
    //     let mut s_into = [0u8; 16];
    //     let s = sgbr.next_bytes(&mut s_into);
    //     if s.len() == 0 { break; }
    //     dbg!(s);
    // }
}
