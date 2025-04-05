use std::{cmp::min, env::set_current_dir, fmt::{Arguments, Debug}, fs, io::{BufRead, Write}, ops::{Bound, Deref, Range, RangeBounds}, path::Iter, str};

use hexdump::{AsHexdump, ByteSliceReader, DoHexdum, GroupedOptions, HexdumpIoWriter, HexdumpLineWriter, HexdumpOptions, HexdumpOptionsBuilder, HexdumpRange, MyByteReader, SliceGroupedByteReader, SliceGroupedReader, ToHexdump, WriteHexdump};

mod hexdump;
#[cfg(test)]
mod test;

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

// Start
// bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
// aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
// End

pub struct RangeByteYielder {
    ranges: Vec<(u8, usize)>,
    range_index: usize,
    elt_index: usize
}

impl RangeByteYielder {
    fn new(ranges: Vec<(u8, usize)>) -> Self {
        Self { ranges, range_index: 0, elt_index: 0 }
    }
}

impl Iterator for RangeByteYielder {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.range_index >= self.ranges.len() {
            return None;
        }
        let (b, len) = self.ranges[self.range_index];
        if self.elt_index >= len {
            self.range_index += 1;
            self.elt_index = 0;
            return self.next();
        }
        self.elt_index += 1;
        Some(b)
    }
}

struct ByteYielder {
    len: usize,
    index: usize,
    b: u8
}

impl ByteYielder {
    fn new(b: u8, len: usize) -> Self {
        Self { len, index: 0, b }
    }
}

impl Iterator for ByteYielder {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        self.index += 1;
        Some(self.b)
    }
}

fn main() {
    let opts = HexdumpOptions::default()
        .range(..0xa60)
        .aligned(true)
        .uppercase(true)
        .grouped(hexdump::GroupSize::Int, 8, hexdump::Spacing::None, hexdump::Spacing::Normal)
        .autoskip(true);

    vec_main_rs().as_hexdump()
        .with_options(opts.clone())
        .range(..)
        .absolute_offset(0xffffffff0usize)
        .hexdump();

    let u = 0xffffffffffffffffusize;

    RangeByteYielder::new(vec![
        (b'a', 128),
        (b'b', 129)
    ]).to_hexdump().range(..).with_options(opts.clone()).hexdump();

    ByteYielder::new(8u8, 0x81)
        .to_hexdump()
        .range(..)
        .hexdump();
}
