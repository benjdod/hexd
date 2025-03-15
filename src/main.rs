/*
Design a b-tree that is stored on disk.
This must have an ordered key and handle storing records of variable lengths.

The b-tree will be stored in pages of length 4096. Each page will be either a
 - leaf page containing one or more records
 - interior page containing references to leaf pages and 
   their corresponding ranges
*/

use std::{cmp::min, env::set_current_dir, fmt::{Arguments, Debug}, fs, io::{BufRead, Write}, ops::{Bound, Deref, Range, RangeBounds}, str};

use hexdump::{hexdump_into_rr, HexdumpIoWriter, HexdumpOptions, MyByteReader, SliceGroupedReader, WriteHexdump};

mod hexdump;


fn main() {
    let mut f = vec![0u8; 4];
    f.extend_from_slice(&[52u8; 96]);
    f.extend_from_slice(&[0u8; 4092]);
    f.extend_from_slice(&[27u8; 27]);

    let mut sg = SliceGroupedReader::new(&f);
    let mut writer = HexdumpIoWriter(std::io::stdout());
    hexdump_into_rr(
        &mut writer, 
        &mut sg, HexdumpOptions {
            omit_equal_rows: true,
            // group_size: hexdump::GroupSize::Int,
            group_spacing: hexdump::Spacing::Wide,
            ..Default::default()
        }).unwrap();
}
