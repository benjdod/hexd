use std::{any::Any, cmp::{max, min}, convert::Infallible, fmt::{Arguments, Debug}, io::BufRead, ops::{BitAnd, Bound, Range, RangeBounds, RangeFull, Shr}, ptr::addr_eq};

pub struct HexdumpIoWriter<W>(pub W) where W: std::io::Write;
pub struct HexdumpFmtWriter<W>(pub W) where W: std::fmt::Write;

pub trait WriteHexdump {
    type Error: Debug;
    fn write_hexdump_str(&mut self, s: &str) -> Result<(), Self::Error>;
    fn write_hexdump_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error>;
}

impl<W> WriteHexdump for HexdumpIoWriter<W> where W: std::io::Write {
    type Error = std::io::Error;
    fn write_hexdump_str(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.0.write_all(s.as_bytes())
    }
    fn write_hexdump_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
        self.0.write_fmt(args)
    }
}

impl<W> WriteHexdump for HexdumpFmtWriter<W> where W: std::fmt::Write {
    type Error = std::fmt::Error;
    fn write_hexdump_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_str(s)
    }
    fn write_hexdump_fmt(&mut self, args: Arguments<'_>) -> Result<(), Self::Error> {
        self.0.write_fmt(args)
    }
}

pub struct HexdumpRange {
    pub skip: usize,
    pub limit: Option<usize>
}

impl HexdumpRange {
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
}

pub struct HexdumpOptions {
    pub omit_equal_rows: bool,
    pub uppercase: bool,
    pub print_ascii: bool,
    pub align: bool,
    pub grouping: Grouping,
    // pub group_size: GroupSize,
    // pub byte_spacing: Spacing,
    // pub num_groups: usize,
    // pub group_spacing: Spacing,
    pub print_range: HexdumpRange,
    pub index_offset: IndexOffset
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexOffset {
    Relative(usize),
    Absolute(usize)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grouping {
    Ungrouped {
        byte_count: usize,
        spacing: Spacing
    },
    Grouped {
        group_size: GroupSize,
        num_groups: usize,
        byte_spacing: Spacing,
        group_spacing: Spacing
    }
}

impl Grouping {
    pub fn elt_width(&self) -> usize {
        match self {
            &Grouping::Ungrouped { byte_count, spacing: _ } => byte_count,
            &Grouping::Grouped { group_size, num_groups, byte_spacing: _, group_spacing: _ } => {
                group_size.element_count() * num_groups
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
    None,
    Normal,
    Wide,
    UltraWide
}

trait VisualWidth {
    fn visual_width(&self) -> usize;
}

impl VisualWidth for Spacing {
    fn visual_width(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Normal => 1,
            Self::Wide => 2,
            Self::UltraWide => 4
        }
    }
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

impl Default for HexdumpOptions {
    fn default() -> Self {
        Self {
            omit_equal_rows: true,
            uppercase: true,
            print_ascii: true,
            align: true,
            grouping: Grouping::Grouped { group_size: GroupSize::Int, num_groups: 4, byte_spacing: Spacing::None, group_spacing: Spacing::Normal },
            print_range: HexdumpRange { skip: 0, limit: None },
            index_offset: IndexOffset::Relative(0)
        }
    }
}

impl HexdumpOptions {
    fn row_width(&self) -> usize {
        match self.grouping {
            Grouping::Ungrouped { byte_count, spacing } => {
                (byte_count * 2) + (spacing.visual_width() * (byte_count - 1))
            }
            Grouping::Grouped { group_size, num_groups, byte_spacing, group_spacing } => {
                let m = group_size.element_count();
                let single_group_width = (2 * m) + (byte_spacing.visual_width() * (m - 1));
                let gg = (single_group_width * num_groups) + (group_spacing.visual_width() * (num_groups - 1));
                gg
            }
        }
    }

    fn elt_width(&self) -> usize {
        self.grouping.elt_width()
    }

    fn spacing_for_element_index(&self, idx: usize) -> Spacing {
        match self.grouping {
            Grouping::Ungrouped { byte_count: _, spacing } => spacing,
            Grouping::Grouped { group_size, num_groups: _, byte_spacing, group_spacing } => {
                match group_size {
                    GroupSize::Byte => group_spacing,
                    gs => {
                        let s = gs.element_count();
                        if idx % s == s - 1 {
                            group_spacing
                        } else {
                            byte_spacing
                        }
                    }
                }
            }
        }
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

fn number_to_hex<T: BitAnd<usize, Output = T> + Shr<T, Output = T> + Copy + TryInto<u8> + From<u8>>(n: T, o: &mut [u8], is_upper: bool) where <T as TryInto<u8>>::Error: Debug {
    let shr: T = 4.into();
    let mut nn = n;
    for i in 0..o.len() {
        let i = o.len() - 1 - i;
        let idx = nn & 0xf;
        let f: u8 = idx.try_into().unwrap();
        let b = if f > 0 {
            if is_upper { UPPER_LUT[f as usize] } else { LOWER_LUT[f as usize] }
        } else {
            UPPER_LUT[0]
        };

        o[i] = b;
        nn = nn >> shr;
    }
}

enum ElideSearch {
    Byte([u8; 1]),
    Short([u8; 2]),
    Int([u8; 4]),
    Long([u8; 8]),
    ULong([u8; 16])
}

struct StackBuffer<const N: usize> {
    buffer: [u8; N],
    len: usize
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

    fn as_mut_range_slice<'a>(&'a mut self, start: usize, end: usize) -> &'a mut [u8] {
        let i = &mut self.buffer.as_mut_slice()[start..end];
        i
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
}

impl<const N: usize> AsRef<[u8]> for StackBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
}

impl ElideSearch {
    fn as_slice<'a>(&'a self) -> &'a [u8] {
        match self {
            Self::Byte(b) => b,
            Self::Short(b) => b,
            Self::Int(b) => b,
            Self::Long(b) => b,
            Self::ULong(b) => b
        }
    }

    // fn into_full_vec(&self, options: &HexdumpOptions) -> Vec<u8> {
    //     let mut o: Vec<u8> = Vec::with_capacity(options.elt_width());
    //     while o.len() < options.elt_width() {
    //         o.extend_from_slice(self.as_slice());
    //     }
    //     o
    // }

    fn into_stack_buffer<const N: usize>(&self, options: &HexdumpOptions) -> StackBuffer::<N> {
        let mut o = StackBuffer::<N>::new();
        while o.len < options.elt_width() {
            o.extend_from_slice(self.as_slice());
        }
        o
    }
}

enum RowBuf<'a> {
    Full(&'a [u8]),
    Left(&'a [u8]),
    Right(&'a [u8])
}

impl<'a> RowBuf<'a> {
    fn as_slice(&'a self) -> &'a [u8] {
        match self {
            &Self::Full(f) => f,
            &Self::Left(f) => f,
            &Self::Right(f) => f
        }
    }
    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn is_empty(&self) -> bool {
        self.as_slice().len() == 0
    }

    fn is_full(&self) -> bool {
        match self {
            &Self::Full(_) => true,
            _ => false
        }
    }
}

#[inline]
fn truncate_byte_index(byte_index: usize, options: &HexdumpOptions) -> usize {
    (byte_index / options.elt_width()) * options.elt_width()
}

#[inline]
fn hexwidth_of(u: usize) -> usize {
    let mut u = u;
    let mut i = 0usize;
    while u > 0 {
        u >>= 4;
        i += 1;
    }
    i
}

pub fn hexdump_into_rr<
    W: WriteHexdump, 
    Reader: MyByteReader
>(w: &mut W, reader: &mut Reader, options: HexdumpOptions) -> Result<(), W::Error> {
    let start = options.print_range.skip;
    let end = options.print_range.limit;

    let aligned_index_into = |c: &RowBuf, i: usize| {
        match c {
            &RowBuf::Full(b) => Some(b[i]),
            &RowBuf::Left(b) => if i < b.len() { Some(b[i]) } else { None },
            &RowBuf::Right(b) => {
                if i < (options.elt_width() - b.len()) {
                    None
                } else {
                    Some(b[i - (options.elt_width() - b.len())])
                }
            }
        }
    };

    let write_row = |writer: &mut W, c: &RowBuf| {
        let mut b = StackBuffer::<128>::new();

        for i in 0..options.elt_width() {
            let ch = aligned_index_into(c, i);

            let [hi, lo] = match ch {
                Some(ch) => if options.uppercase { ch.to_hex_upper() } else { ch.to_hex_lower() },
                None => [b' ', b' ']
            };
            b.push(hi);
            b.push(lo);
            b.extend_from_slice(options.spacing_for_element_index(i).as_spaces());
        }

        let s = std::str::from_utf8(b.as_slice()).unwrap();

        writer.write_hexdump_str(s).unwrap();
    };

    let index_max_hexwidth = max(8, hexwidth_of(reader.total_byte_hint().unwrap_or(0)));

    let write_row_idx = |writer: &mut W, idx: Option<usize>| {
        match idx {
            Some(i) => {
                let i = match options.index_offset {
                    IndexOffset::Relative(o) => i + o,
                    IndexOffset::Absolute(o) => i - start + o
                };
                let mut oo = [0u8; 32];
                let o = &mut oo.as_mut_slice()[..index_max_hexwidth];
                number_to_hex(i, o, false);
                writer.write_hexdump_str(std::str::from_utf8(o).unwrap())?;
                writer.write_hexdump_str("    ")
            }
            None => {
                writer.write_hexdump_str(" --snip--     ")// 
            }
        }
    };

    let is_printable = |ch:char| {
        ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch == ' '
    };

    let write_row_ascii = |writer: &mut W, c: &RowBuf| {
        // let mut v: Vec<u8> = Vec::with_capacity(options.elt_width() + 2);
        let mut v = StackBuffer::<128>::new();
        v.push(b'|');
        for i in 0..options.elt_width() {
            v.push(match aligned_index_into(c, i) {
                Some(ch) => if is_printable(ch as char) { ch } else { b'.' },
                None => b' '
            });
        }
        v.push(b'|');
        let s = std::str::from_utf8(v.as_slice()).unwrap();
        writer.write_hexdump_str(&s)?;
        Ok::<(), W::Error>(())
    };

    let mut bytebuf = StackBuffer::<256>::new();

    let mut i = start;

    if start > 0 {
        reader.skip_n(start).unwrap();
    }

    let mut elide_search: Option<(usize, ElideSearch)> = None;

    let slice_equals_elision = |s: &RowBuf, search: &ElideSearch| {
        s.is_full() && match search {
            ElideSearch::Byte(c) => s.as_slice().iter().all(|b| *b == c[0]),
            c => {
                let c = c.as_slice();
                for (i, b) in s.as_slice().iter().enumerate() {
                    if c[i % c.len()] != *b { return false; }
                }
                true
            }
        }
    };

    loop {
        if let Some(end_at) = end {
            if i >= end_at {
                break;
            }
        }
        let x = if !options.align || i % options.elt_width() == 0 {
            let take = end
                .map(|end_at| if i <= end_at { min(end_at - i, options.elt_width()) } else { 0 })
                .unwrap_or(options.elt_width());
            let bb = reader.next_n(&mut bytebuf.as_mut_slice()[0..take]).unwrap();

            if bb.len() == options.elt_width() {
                RowBuf::Full(&bb)
            } else {
                RowBuf::Left(&bb)
            }
        } else {
            let bb = reader.next_n(&mut bytebuf.as_mut_slice()[i % options.elt_width() .. options.elt_width()]).unwrap();

            if bb.len() > 0 {
                RowBuf::Right(bb)
            } else {
                RowBuf::Full(bb)
            }
        };

        let byte_index = truncate_byte_index(i, &options);

        if x.is_empty() { break; }

        if options.omit_equal_rows {
            match &elide_search {
                Some((elide_start, search)) => {
                    match slice_equals_elision(&x, search) {
                        true => {
                            i += x.len();
                            continue;
                        },
                        false if !x.is_full() => { }
                        false => {
                            let cc = search.into_stack_buffer::<256>(&options);
                            let xx = RowBuf::Full(cc.as_slice());

                            if byte_index - elide_start >= options.elt_width() * 3 {
                                write_row_idx(w, None)?;
                                w.write_hexdump_str("\n")?;
                            }

                            write_row_idx(w, Some(byte_index - options.elt_width()))?;
                            write_row(w, &xx);
                            if options.print_ascii {
                                w.write_hexdump_str(" ")?;
                                write_row_ascii(w, &xx)?;
                            }
                            w.write_hexdump_str("\n")?;
                            elide_search = None;
                        }
                    }
                }
                None => {
                    let current_elide_search = match options.grouping {
                        Grouping::Ungrouped { byte_count , spacing } if x.is_full() => {
                            let search_char = x.as_slice().get(0);
                            if x.as_slice().iter().all(|ch| *ch == *search_char.unwrap()) {
                                Some((byte_index, ElideSearch::Byte([x.as_slice()[0]])))
                            } else {
                                None
                            }
                        }
                        Grouping::Grouped { group_size, num_groups, byte_spacing, group_spacing } if x.is_full() => {
                            let search_slice = &x.as_slice()[..group_size.element_count()];
                            let all_eq = x.as_slice().chunks(search_slice.len()).skip(1).all(|s| s == search_slice);
                            if all_eq {
                                Some((byte_index, match search_slice.len() {
                                    1 => ElideSearch::Byte(<[u8; 1]>::try_from(search_slice).unwrap()),
                                    2 => ElideSearch::Short(<[u8; 2]>::try_from(search_slice).unwrap()),
                                    4 => ElideSearch::Int(<[u8; 4]>::try_from(search_slice).unwrap()),
                                    8 => ElideSearch::Long(<[u8; 8]>::try_from(search_slice).unwrap()),
                                    16 => ElideSearch::ULong(<[u8; 16]>::try_from(search_slice).unwrap()),
                                    _ => unreachable!()
                                }))
                            } else {
                                None
                            }
                        },
                        _ => None
                    };

                    match current_elide_search {
                        Some(es) => { elide_search = Some(es); }
                        None => { }
                    }
                }
            };
        }

        write_row_idx(w, Some(byte_index))?;
        write_row(w, &x);
        if options.print_ascii {
            w.write_hexdump_str(" ")?;
            write_row_ascii(w, &x)?;
        }
        w.write_hexdump_str("\n")?;
        i += x.len();
    }

    return Ok(());
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

impl EndianBytes<4> for u32 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 4] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes()
        }
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
        // dbg!(self.current_elt, self.u_index);
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

pub trait MyByteReader {
    type Error: Debug;
    fn next_n<'buf>(&mut self, buf: &'buf mut[u8]) -> Result<&'buf [u8], Self::Error>;
    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error>;
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