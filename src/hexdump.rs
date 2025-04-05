use std::{cmp::{max, min}, fmt::Debug, convert::Infallible, io::{Stderr, Stdout, Write}, ops::{Bound, Div, Mul, RangeBounds}};

pub trait WriteHexdump: Sized {
    type Error: Debug;
    type Output;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output;
}

struct IOWriter<W: std::io::Write>(W);

impl<W: std::io::Write> WriteHexdump for IOWriter<W> {
    type Error = std::io::Error;
    type Output = Result<(), std::io::Error>;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.map(|_| ())
    }
}

impl WriteHexdump for Stdout {
    type Error = std::io::Error;
    type Output = ();
    fn write_line(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(self)
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap();
        ()
    }
}

impl WriteHexdump for Stderr {
    type Error = std::io::Error;
    type Output = ();
    fn write_line(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(self)
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap();
        ()
    }
}

impl WriteHexdump for String {
    type Error = Infallible;
    type Output = String;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push_str(s);
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}

impl WriteHexdump for Vec<String> {
    type Error = Infallible;
    type Output = Vec<String>;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push(s.to_string());
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HexdRange {
    pub skip: usize,
    pub limit: Option<usize>
}

impl HexdRange {
    pub fn new<R: RangeBounds<usize>>(r: R) -> Self {
        let skip = match r.start_bound() {
            Bound::Unbounded => 0usize,
            Bound::Included(s) => *s,
            Bound::Excluded(s) => s + 1
        };
        let limit = match r.end_bound() {
            Bound::Unbounded => None,
            Bound::Included(s) => Some(*s + 1),
            Bound::Excluded(s) => Some(*s)
        };

        Self { skip, limit }
    }

    pub fn length(&self) -> Option<usize> {
        self.limit.map(|lim| lim - self.skip)
    }
}

#[derive(Debug, Clone)]
pub struct HexdOptions {
    pub autoskip: bool,
    pub uppercase: bool,
    pub print_ascii: bool,
    pub align: bool,
    pub grouping: Grouping,
    pub print_range: HexdRange,
    pub index_offset: IndexOffset
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexOffset {
    Relative(usize),
    Absolute(usize)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupedOptions {
    pub group_size: GroupSize,
    pub byte_spacing: Spacing,
    pub num_groups: usize,
    pub group_spacing: Spacing
}

impl Default for GroupedOptions {
    fn default() -> Self {
        Self {
            group_size: GroupSize::Short,
            byte_spacing: Spacing::None,
            num_groups: 8,
            group_spacing: Spacing::Normal
        }
    }
}

impl GroupedOptions {
    pub fn byte_spacing(self, byte_spacing: Spacing) -> Self {
        Self { byte_spacing, ..self }
    }

    pub fn group_spacing(self, group_spacing: Spacing) -> Self {
        Self { group_spacing, ..self }
    }

    pub fn num_groups(self, num_groups: usize) -> Self {
        Self { num_groups, ..self }
    }

    pub fn group_size(self, group_size: GroupSize) -> Self {
        Self { group_size, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grouping {
    Ungrouped {
        byte_count: usize,
        spacing: Spacing
    },
    Grouped(GroupedOptions)
}

impl Grouping {
    pub fn elt_width(&self) -> usize {
        match self {
            &Grouping::Ungrouped { byte_count, spacing: _ } => byte_count,
            &Grouping::Grouped(GroupedOptions { group_size, num_groups, byte_spacing: _, group_spacing: _ }) => {
                group_size.element_count() * num_groups
            }
        }
    }

    pub fn spacing_for_index(&self, index: usize) -> Spacing {
        match self {
            &Grouping::Ungrouped { byte_count: _, spacing } => spacing,
            &Grouping::Grouped(GroupedOptions { group_size, num_groups: _, byte_spacing, group_spacing }) => {
                let elt_count = group_size.element_count();
                if index % elt_count == elt_count - 1 { group_spacing } else { byte_spacing }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupSize {
    Byte,
    Short,
    Int,
    Long,
    ULong
}

impl GroupSize {
    fn element_count(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Short => 2,
            Self::Int => 4,
            Self::Long => 8,
            Self::ULong => 16
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spacing {
    /// No spacing is included between elements.
    None,

    /// A single space is included between elements.
    Normal,

    /// Two spaces are included between elements.
    Wide,

    /// Four spaces are included between elements.
    UltraWide
}

impl Spacing {
    fn as_spaces(&self) -> &'static [u8] {
        match self {
            Self::None => &[],
            Self::Normal => " ".as_bytes(),
            Self::Wide => "  ".as_bytes(),
            Self::UltraWide => "    ".as_bytes()
        }
    }
}

impl Default for HexdOptions {
    fn default() -> Self {
        Self {
            autoskip: true,
            uppercase: true,
            print_ascii: true,
            align: true,
            grouping: Grouping::Grouped(GroupedOptions::default()),
            print_range: HexdRange { skip: 0, limit: None },
            index_offset: IndexOffset::Relative(0)
        }
    }
}

pub trait HexdOptionsBuilder: Sized {
    fn as_options<'a>(&'a self) -> &'a HexdOptions;
    fn with_options(self, o: HexdOptions) -> Self;
    fn range<R: RangeBounds<usize>>(self, range: R) -> Self {
        let o = self.as_options();
        let opts = HexdOptions {
            print_range: HexdRange::new(range),
            ..o.clone()
        };
        self.with_options(opts)
    }
    fn aligned(self, align: bool) -> Self {
        let o = self.as_options();
        let options = HexdOptions {
            align,
            ..o.clone()
        };
        self.with_options(options)
    }
    fn uppercase(self, uppercase: bool) -> Self {
        let o = self.as_options();
        let options = HexdOptions {
            uppercase,
            ..o.clone()
        };
        self.with_options(options)
    }
    fn grouping(self, grouping: Grouping) -> Self {
        let o = self.as_options();
        let options = HexdOptions {
            grouping,
            ..o.clone()
        };
        self.with_options(options)
    }
    fn ungrouped(self, num_bytes: usize, spacing: Spacing) -> Self {
        let o = self.as_options();
        let grouping = Grouping::Ungrouped { byte_count: num_bytes, spacing };
        let options = HexdOptions {
            grouping,
            ..o.clone()
        };
        self.with_options(options)
    }
    fn grouped(self, group_size: GroupSize, num_groups: usize, byte_spacing: Spacing, group_spacing: Spacing) -> Self {
        let o = self.as_options();
        let grouping = Grouping::Grouped(GroupedOptions { group_size, num_groups, byte_spacing, group_spacing });
        let options = HexdOptions {
            grouping,
            ..o.clone()
        };
        self.with_options(options)
    }
    fn autoskip(self, autoskip: bool) -> Self {
        let o = self.as_options();
        let options = HexdOptions {
            autoskip,
            ..o.clone()
        };
        self.with_options(options)
    }
    fn relative_offset(self, offset: usize) -> Self {
        let o = self.as_options();
        let options = HexdOptions {
            index_offset: IndexOffset::Relative(offset),
            ..o.clone()
        };
        self.with_options(options)
    }
    fn absolute_offset(self, offset: usize) -> Self {
        let o = self.as_options();
        let options = HexdOptions {
            index_offset: IndexOffset::Absolute(offset),
            ..o.clone()
        };
        self.with_options(options)
    }
}

impl HexdOptionsBuilder for HexdOptions {
    fn as_options<'a>(&'a self) -> &'a HexdOptions {
        self
    }
    fn with_options(self, o: HexdOptions) -> Self {
        o
    }
}

impl HexdOptions {
    fn elt_width(&self) -> usize {
        self.grouping.elt_width()
    }
}

trait ToHex {
    fn to_hex_lower(self) -> [u8; 2];
    fn to_hex_upper(self) -> [u8; 2];
}

const UPPER_LUT: [u8; 16] = [ b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F' ];
const LOWER_LUT: [u8; 16] = [ b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f' ];

impl ToHex for u8 {
    fn to_hex_upper(self) -> [u8; 2] {
        let mut x = [0u8; 2];
        x[1] = UPPER_LUT[(self & 0xf) as usize];
        x[0] = UPPER_LUT[(self >> 4) as usize];
        x
    }
    fn to_hex_lower(self) -> [u8; 2] {
        let mut x = [0u8; 2];
        x[1] = LOWER_LUT[(self & 0xf) as usize];
        x[0] = LOWER_LUT[(self >> 4) as usize];
        x
    }
}
#[inline]
fn truncate_to<N: Mul<N, Output = N> + Div<N, Output = N> + Copy>(n: N, trunc: N) -> N {
    n / trunc * trunc
}

#[inline]
fn trunc_ceil_usize(n: usize, trunc: usize) -> usize {
    let rem = n / trunc;
    match rem {
        0 => rem * trunc,
        _ => (rem + 1) * trunc
    }
}

trait TruncateInteger: Copy + Sized {
    type Output;
    fn trunc_floor(self, trunc: Self) -> Self::Output;
    fn trunc_ceil(self, trunc: Self) -> Self::Output;
}

trait HexVisualWidth {
    fn hex_visual_width(&self) -> usize;
    fn byte_visual_width(&self) -> usize {
        self.hex_visual_width() / 2
    }
}

impl HexVisualWidth for usize {
    fn hex_visual_width(&self) -> usize {
        let mut u = *self;
        let mut i = 0usize;
        while u > 0 {
            u >>= 4;
            i += 1;
        }
        i
    }
}

#[derive(Clone, PartialEq, Eq)]
struct StackBuffer<const N: usize> {
    buffer: [u8; N],
    len: usize
}

impl<const N: usize> std::fmt::Debug for StackBuffer<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackBuffer")
            .field("slice", &self.as_slice())
            .field("len", &self.len)
            .finish()
    }
}

impl<const N: usize> StackBuffer<N> {
    fn new() -> Self {
        Self { buffer: [0u8; N], len: 0 }
    }
    fn as_slice<'a>(&'a self) -> &'a [u8] {
        &self.buffer[..self.len]
    }

    fn clear(&mut self) {
        self.len = 0
    }

    fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
        self.buffer.as_mut_slice()
    }

    fn push(&mut self, b: u8) {
        self.check_extension(1);
        self.buffer[self.len] = b;
        self.len += 1;
    }

    fn check_extension(&self, extend_by: usize) {
        if self.len + extend_by >= N {
            panic!("Stack-based buffer overflow");
        }
    }

    fn extend_from_slice(&mut self, other: &[u8]) {
        self.check_extension(other.len());
        self.buffer[self.len..self.len + other.len()].copy_from_slice(other);
        self.len += other.len();
    }

    fn as_str<'a>(&'a self) -> &'a str {
        std::str::from_utf8(self.as_slice()).unwrap()
    }
}

impl<const N: usize> AsRef<[u8]> for StackBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    BigEndian,
    LittleEndian
}

pub trait GroupedReader<const N: usize> {
    fn read_next(&mut self, end: Endianness) -> Option<[u8; N]>;
    fn size(&self) -> usize { N }
}

pub trait EndianBytes<const N: usize> {
    fn to_bytes(&self, end: Endianness) -> [u8; N];
}

impl EndianBytes<1> for u8 {
    fn to_bytes(&self, _: Endianness) -> [u8; 1] {
        [*self]
    }
}

impl EndianBytes<1> for i8 {
    fn to_bytes(&self, _: Endianness) -> [u8; 1] {
        [*self as u8]
    }
}

impl EndianBytes<2> for u16 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 2] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<2> for i16 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 2] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<4> for u32 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 4] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<4> for i32 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 4] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<8> for u64 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 8] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<8> for i64 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 8] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<16> for u128 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 16] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

impl EndianBytes<16> for i128 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 16] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
    }
}

pub struct ByteSliceReader<'a> {
    slice: &'a [u8],
    index: usize
}

impl<'a> ByteSliceReader<'a> {
    pub fn new(slice: &'a [u8]) -> ByteSliceReader<'a> {
        Self { slice, index: 0usize }
    }
}

impl<'a> MyByteReader for ByteSliceReader<'a> {
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut[u8]) -> Result<&'buf [u8], Self::Error> {
        if self.index >= self.slice.len() {
            return Ok(&[])
        }
        let end = min(self.index + buf.len(), self.slice.len()) - self.index;
        buf[..end].copy_from_slice(&self.slice[self.index..self.index + end]);
        self.index += end;
        Ok(&buf[..end])
    }

    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.index += n;
        Ok(self.index)
    }

    fn total_byte_hint(&self) -> Option<usize> {
        Some(self.slice.len())
    }
}

pub struct SliceGroupedReader<'a, U: EndianBytes<N>, const N: usize> {
    slice: &'a[U],
    index: usize
}

pub struct SliceGroupedByteReader<'a, U: EndianBytes<N>, const N: usize> {
    slice: &'a [U],
    elt_index: usize,
    u_index: usize,
    current_elt: Option<[u8; N]>,
    endianness: Endianness
}

impl<'a, U: EndianBytes<N>, const N: usize> MyByteReader for SliceGroupedByteReader<'a, U, N> {
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut[u8]) -> Result<&'buf [u8], Self::Error> {
        Ok(self.next_bytes(buf))
    }

    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error> {
        self.advance_indices_by(n);
        Ok(n)
    }

    fn total_byte_hint(&self) -> Option<usize> {
        Some(self.slice.len() * N)
    }
}

impl<'a, U: EndianBytes<N>, const N: usize> SliceGroupedByteReader<'a, U, N> {
    pub fn new(slice: &'a [U], endianness: Endianness) -> Self {
        let current_elt = if slice.len() > 0 { Some(slice[0].to_bytes(endianness)) } else { None };
        Self { slice, elt_index: 0, u_index: 0, current_elt, endianness }
    }
    pub fn next_bytes<'buf>(&mut self, o: &'buf mut [u8]) -> &'buf [u8] {
        for i in 0..o.len() {
            if let Some(cb) = self.next_byte() {
                o[i] = cb;
            } else {
                return &o[..i];
            }
        }
        &o[..]
    }

    fn next_byte(&mut self) -> Option<u8> {
        let o = self.current_elt.map(|ce| ce[self.u_index]);
        self.advance_indices();
        o
    }

    fn advance_indices(&mut self) {
        self.u_index += 1;
        if self.u_index >= N {
            self.u_index = 0;
            self.elt_index += 1;
            self.current_elt = if self.elt_index < self.slice.len() {
                Some(self.slice[self.elt_index].to_bytes(self.endianness))
            } else { 
                None 
            }
        }
    }

    fn advance_indices_by(&mut self, adv: usize) {
        if N == 1 {
            self.elt_index = adv;
            self.u_index = 0;
            self.current_elt = if self.elt_index < self.slice.len() {
                Some(self.slice[self.elt_index].to_bytes(self.endianness))
            } else {
                None
            };
            return;
        }
        let mut adv = adv;
        if self.u_index > 0 {
            adv -= N - self.u_index;
            self.u_index = 0;
            self.elt_index += 1;
        }

        while adv > N {
            adv -= N;
            self.u_index = 0;
            self.elt_index += 1;
        }

        self.current_elt = if self.elt_index < self.slice.len() { Some(self.slice[self.elt_index].to_bytes(self.endianness)) } else { None };

        if adv > 0 {
            self.u_index = adv;
        }
    }
}

impl<'a, U: EndianBytes<N>, const N: usize> SliceGroupedReader<'a, U, N> {
    pub fn new(slice: &'a [U]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, U: EndianBytes<N>, const N: usize> SliceGroupedReader<'a, U, N> {
    pub fn next(&mut self, end: Endianness) -> Option<[u8; N]> {
        if self.index < self.slice.len() {
            let s = Some(self.slice[self.index].to_bytes(end));
            self.index += 1;
            s
        } else {
            None
        }
    }
}

impl<'a, const N: usize, U: EndianBytes<N>> GroupedReader<N> for SliceGroupedReader<'a, U, N> {
    fn read_next(&mut self, end: Endianness) -> Option<[u8; N]> {
        self.next(end)
    }
}

pub struct IteratorByteReader<I: Iterator<Item = u8>> {
    iterator: I
}

impl<I: Iterator<Item = u8>> IteratorByteReader<I> {
    pub fn new(iterator: I) -> Self {
        Self { iterator }
    }
}

impl<I: Iterator<Item = u8>> MyByteReader for IteratorByteReader<I> {
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut[u8]) -> Result<&'buf [u8], Self::Error> {
        let mut i = 0usize;
        while i < buf.len() {
            match self.iterator.next() {
                Some(b) => { buf[i] = b; },
                None => {
                    return Ok(&buf[..i]);
                }
            }
            i += 1;
        }
        Ok(&buf[..i])
    }

    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error> {
        for i in 0..n {
            if let None = self.iterator.next() {
                return Ok(i)
            }
        }
        Ok(n)
    }
}

pub trait MyByteReader {
    type Error: Debug;
    fn next_n<'buf>(&mut self, buf: &'buf mut[u8]) -> Result<&'buf [u8], Self::Error>;

    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error> {
        const SKIP_LEN: usize = 64usize;
        let mut skipbuf = [0u8; SKIP_LEN];
        let mut i = 0usize;
        while i < n {
            let skipbuf = &mut skipbuf[..min(n - i, SKIP_LEN)];
            let b = self.next_n(skipbuf)?;
            if b.len() == 0 {
                return Ok(i);
            }
            i += b.len();
        }
        Ok(n)

    }
    fn total_byte_hint(&self) -> Option<usize> {
        None
    }
}

impl<'b, T: Iterator<Item = &'b u8>> MyByteReader for T {
    type Error = Infallible;
    fn next_n<'a>(&mut self, buf: &'a mut[u8]) -> Result<&'a [u8], Self::Error> {
        let mut i = 0;
        while i < buf.len() {
            match self.next() {
                Some(u) => { buf[i] = *u; }
                None => {
                    break;
                }
            };
            i += 1;
        }
        Ok(&buf[..i])
    }
    
    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error> {
        for i in 0..n {
            match self.next() {
                Some(_) => { },
                None => {
                    return Ok(i);
                }
            }
        }
        Ok(n)
    }

    fn total_byte_hint(&self) -> Option<usize> {
        match self.size_hint() {
            (_, Some(upper)) => { Some(upper) },
            (lower, None) if lower > 0 => { Some(lower) },
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
struct RowBuffer {
    buffer: StackBuffer<MAX_BUFFER_SIZE>,
    length: usize,
    row_index: usize,
    elt_index: usize
}

impl RowBuffer {
    fn is_right_aligned(&self) -> bool {
        self.elt_index != self.row_index
    }
}

const MAX_BUFFER_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
enum HexdumpLineIteratorState {
    NotStarted,
    InProgress,
    Completed
}

struct HexdumpLineIterator<R: MyByteReader> {
    reader: R,
    index: usize,
    options: HexdOptions,
    state: HexdumpLineIteratorState,
    elision_match: Option<ElisionMatch>
}

#[derive(Debug, Clone)]
struct ElisionMatch {
    starting_index: usize,
    buffer: StackBuffer<MAX_BUFFER_SIZE>,
    grouping: Grouping
}

impl ElisionMatch {
    fn try_match(row: &RowBuffer, options: &HexdOptions) -> Option<Self> {
        let buffer = &row.buffer;
        match options.grouping {
            _ if buffer.len != options.elt_width() => None,
            Grouping::Ungrouped { byte_count: _, spacing: _ } => {
                let sc = buffer.buffer[0];
                if buffer.as_slice().iter().all(|b| *b == sc) {
                    Some(ElisionMatch { starting_index: row.elt_index, buffer: buffer.clone(), grouping: options.grouping })
                } else {
                    None
                }
            },
            Grouping::Grouped(GroupedOptions { group_size, num_groups: _, byte_spacing: _, group_spacing: _ }) => {
                let group_size = group_size.element_count();
                let s = &buffer.as_slice()[..group_size];
                if s.chunks(group_size).all(|chunk| { 
                    chunk == s 
                }) {
                    Some(ElisionMatch { starting_index: row.elt_index, buffer: buffer.clone(), grouping: options.grouping })
                } else {
                    None
                }
            },
        }
    }

    fn matches(&self, row: &RowBuffer, options: &HexdOptions) -> bool {
        if row.buffer.len == options.elt_width() {
            self.buffer == row.buffer
        } else {
            false
        }
    }
}

impl<'a, R: MyByteReader> HexdumpLineIterator<R> {
    pub fn new(reader: R, options: HexdOptions) -> Self {
        Self { reader, index: 0, options, state: HexdumpLineIteratorState::NotStarted, elision_match: None }
    }

    fn read_into_buffer(&mut self, len: usize) -> RowBuffer {
        let mut buffer = StackBuffer::<MAX_BUFFER_SIZE>::new();

        let actually_read_len = {
            let n = self.reader.next_n(&mut buffer.as_mut_slice()[..len]).unwrap();
            n.len()
        };

        buffer.len += actually_read_len;

        let o = RowBuffer { buffer, length: actually_read_len, row_index: self.calculate_row_index(), elt_index: self.index };
        self.index += actually_read_len;
        self.state = HexdumpLineIteratorState::InProgress;
        o
    }

    fn calculate_row_index(&self) -> usize {
        if !self.options.align {
            self.index
        } else {
            self.index / self.options.elt_width() * self.options.elt_width()
        }
    }
}

enum LineIteratorResult {
    Elided(RowBuffer),
    Row(RowBuffer)
}

impl<R: MyByteReader> Iterator for HexdumpLineIterator<R> {
    type Item = LineIteratorResult;
    
    fn next(&mut self) -> Option<Self::Item> {
        let st = self.state;
        match st {
            HexdumpLineIteratorState::NotStarted | HexdumpLineIteratorState::InProgress => {
                if self.state == HexdumpLineIteratorState::NotStarted && self.options.print_range.skip > 0 {
                    self.reader.skip_n(self.options.print_range.skip).unwrap();
                    self.index = self.options.print_range.skip;
                }

                let read_len = {
                    let limit = self.options.print_range.limit
                        .map(|limit| if self.index < limit { limit - self.index } else { 0 })
                        .unwrap_or(self.options.elt_width());

                    let ew = min(self.options.elt_width(), limit);

                    if self.state == HexdumpLineIteratorState::NotStarted && self.options.align {
                        match self.options.print_range.length() {
                            Some(len) if len < self.options.elt_width() => len,
                            _ => ew - (self.index % ew)
                        }
                    } else {
                        ew
                    }
                };

                self.state = HexdumpLineIteratorState::InProgress;
                let rowbuffer = self.read_into_buffer(read_len);

                if self.options.autoskip {
                    if let Some(em) = &self.elision_match {
                        if em.matches(&rowbuffer, &self.options) {
                            return Some(LineIteratorResult::Elided(rowbuffer))
                        } else {
                            self.elision_match = None;
                        }
                    }
                    if let Some(elision_match) = ElisionMatch::try_match(&rowbuffer, &self.options)  {
                        self.elision_match = Some(elision_match);
                    }
                }
                if rowbuffer.length > 0 {
                    Some(LineIteratorResult::Row(rowbuffer))
                } else {
                    self.state = HexdumpLineIteratorState::Completed;
                    None
                }
            }
            HexdumpLineIteratorState::Completed => None
        }
    }
}

struct HexdumpLineWriter<R: MyByteReader, W: WriteHexdump> {
    line_iterator: HexdumpLineIterator<R>,
    writer: W,
    elided_row: Option<(RowBuffer, usize)>,
    str_buffer: StackBuffer<256>,
    options: HexdOptions
}

impl<R: MyByteReader, W: WriteHexdump> HexdumpLineWriter<R, W> {
    fn new(reader: R, writer: W, options: HexdOptions) -> Self {
        let line_iterator = HexdumpLineIterator::new(reader, options.clone());
        Self { line_iterator, writer, elided_row: None, str_buffer: StackBuffer::<256>::new(), options }
    }

    fn do_hexdump(mut self) -> W::Output {
        let r = self.do_hexdump_internal();
        let ll = match r {
            Ok(_) => Ok(self.writer),
            Err(e) => Err(e)
        }.and_then(|mut w| {
            match w.flush() {
                Ok(_) => Ok(w),
                Err(e) => Err(e)
            }
        });
        WriteHexdump::consume(ll)
    }

    fn do_hexdump_internal(&mut self) -> Result<(), W::Error> {
        let mut i = 0usize;
        while let Some(r) = self.line_iterator.next() {
            match r {
                LineIteratorResult::Row(r) => {
                    if self.elided_row.is_some() {
                        let (elided_row, start) = self.elided_row.clone().unwrap();

                        if (i - start) > 1 {
                            self.write_elision();
                            self.flush_line()?;
                        }

                        self.write_row_index(r.row_index - self.options.elt_width());
                        self.write_row_bytes(&elided_row);
                        self.write_row_ascii(&elided_row);
                        self.flush_line()?;
                    }
                    self.elided_row = None;
                    self.write_row_index(r.row_index);
                    self.write_row_bytes(&r);
                    self.write_row_ascii(&r);
                },
                LineIteratorResult::Elided(r) => {
                    if self.elided_row.is_none() {
                        self.elided_row = Some((r, i));
                    } 
                }
            }

            self.flush_line()?;
            i += 1;
        }
        if let Some((r, start)) = self.elided_row.clone() {
            if (i - start) > 1 {
                self.write_elision();
                self.flush_line()?;
            }

            let row_index = (i - 1) * self.options.elt_width();

            let elided_row = r;
            self.write_row_index(row_index);
            self.write_row_bytes(&elided_row);
            self.write_row_ascii(&elided_row);
            self.flush_line()?;
        };
        Ok(())
    }

    #[inline]
    fn u8_to_hex(&self, b: u8) -> [u8; 2] {
        if self.options.uppercase {
            b.to_hex_upper()
        } else {
            b.to_hex_lower()
        }
    }

    fn write_row_index(&mut self, row_index: usize) {
        let v_index = match self.options.index_offset {
            IndexOffset::Absolute(o) => {
                row_index - min(row_index, self.options.print_range.skip) + o
            }
            IndexOffset::Relative(o) => row_index + o
        };

        let bytes = &v_index.to_be_bytes();
        let bl = bytes.len();

        let slice = self.line_iterator.reader
            .total_byte_hint()
            .map(|h| match self.options.index_offset {
                IndexOffset::Absolute(a) => a + h,
                IndexOffset::Relative(r) => self.options.print_range.skip + r + h
            })
            .map(|h| (h.hex_visual_width() + 1) / 2)
            .map(|h| max(h, 4))
            .map(|h| &bytes[(bl - h)..])
            .unwrap_or_else(|| &bytes[(bl - max(4, trunc_ceil_usize(v_index.hex_visual_width() / 2, 2)))..]);

        for b in slice {
            let [high, low] = self.u8_to_hex(*b);
            self.str_buffer.push(high);
            self.str_buffer.push(low);
        }
        self.str_buffer.extend_from_slice(b": ");
    }

    fn write_elision(&mut self) {
        self.str_buffer.extend_from_slice(b"*");
    }

    fn write_row_bytes(&mut self, row: &RowBuffer) {
        for i in 0..self.options.elt_width() {
            let [high, low] = match self.read_row_byte_aligned(row, i) {
                Some(b) => self.u8_to_hex(b),
                None => [b' ', b' ']
            };
            self.str_buffer.push(high);
            self.str_buffer.push(low);
            self.str_buffer.extend_from_slice(self.options.grouping.spacing_for_index(i).as_spaces());
        }

        if self.options.grouping.spacing_for_index(self.options.elt_width() - 1) == Spacing::None {
            self.str_buffer.push(b' ');
        }
    }

    #[inline]
    fn read_row_byte_aligned(&self, row: &RowBuffer, i: usize) -> Option<u8> {
        let ee = row.elt_index % self.options.elt_width();
        if self.options.align && row.is_right_aligned() {
            if i < ee || i >= row.buffer.len + ee {
                None
            } else {
                Some(row.buffer.as_slice()[i - ee])
            } 
        } else {
            if i < row.buffer.len {
                Some(row.buffer.as_slice()[i])
            } else {
                None
            }
        }
    }

    #[inline]
    fn write_row_ascii(&mut self, row: &RowBuffer) {
        // self.str_buffer.push(b' ');
        self.str_buffer.push(b'|');
        for i in 0..self.options.elt_width() {
            let b = self.read_row_byte_aligned(row, i).unwrap_or(b' ');
            self.str_buffer.push(if Self::is_printable_char(b as char) { b } else { b'.' });
        }
        self.str_buffer.push(b'|');
    }
        
    #[inline]
    fn is_printable_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch == ' '
    }

    #[inline]
    fn flush_line(&mut self) -> Result<(), W::Error> {
        if self.str_buffer.len > 0 {
            self.str_buffer.push(b'\n');
        }
        let s = self.str_buffer.as_str();
        self.writer.write_line(s)?;
        self.str_buffer.clear();
        Ok(())
    }
}

pub struct Hexd<R: MyByteReader> {
    reader: R,
    options: HexdOptions
}

impl<R: MyByteReader> Hexd<R> {
    pub fn dump<W: WriteHexdump + Default>(self) -> W::Output {
        let hlw = HexdumpLineWriter::new(self.reader, W::default(), self.options);
        hlw.do_hexdump()
    }

    pub fn dump_to<W: WriteHexdump>(self, writer: W) -> W::Output {
        let hlw = HexdumpLineWriter::new(self.reader, writer, self.options);
        hlw.do_hexdump()
    }

    pub fn dump_io<W: Write>(self, write: W) -> Result<(), std::io::Error> {
        let hlw = HexdumpLineWriter::new(self.reader, IOWriter(write), self.options);
        hlw.do_hexdump()
    }

    pub fn print(self) {
        self.dump_to(std::io::stdout());
    }

    pub fn print_err(self) {
        self.dump_to(std::io::stderr());
    }
}

impl<R: MyByteReader> HexdOptionsBuilder for Hexd<R> {
    fn as_options<'a>(&'a self) -> &'a HexdOptions {
        &self.options
    }

    fn with_options(self, o: HexdOptions) -> Self {
        Self {
            options: o,
            ..self
        }
    }
}

pub trait ToHexd: Sized {
    type Output: MyByteReader;
    fn to_hexd(self) -> Hexd<Self::Output>;
    fn hexd(self) -> Hexd<Self::Output> {
        self.to_hexd()
    }
}

impl<I: Iterator<Item = u8>> ToHexd for I {
    type Output = IteratorByteReader<I>;
    fn to_hexd(self) -> Hexd<Self::Output> {
        Hexd {
            reader: IteratorByteReader { iterator: self },
            options: HexdOptions::default()
        }
    }
}

pub trait AsHexd<'a, R: MyByteReader> {
    fn as_hexd(&'a self) -> Hexd<R>;
    fn hexd(&'a self) -> Hexd<R> {
        self.as_hexd()
    }
}

impl<'a, T: AsRef<[u8]>> AsHexd<'a, ByteSliceReader<'a>> for T {
    fn as_hexd(&'a self) -> Hexd<ByteSliceReader<'a>> {
        let slice = self.as_ref();
        let reader = ByteSliceReader::new(slice);
        Hexd { reader, options: HexdOptions::default() }
    }
}