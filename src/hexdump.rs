use std::{convert::Infallible, fmt::{Arguments, Debug}, io::BufRead, ops::{BitAnd, Bound, Range, RangeBounds, RangeFull, Shr}};

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
    skip: usize,
    limit: Option<usize>
}

pub struct HexdumpOptions {
    pub omit_equal_rows: bool,
    pub uppercase: bool,
    pub print_ascii: bool,
    pub align: bool,
    pub group_size: GroupSize,
    pub byte_spacing: Spacing,
    pub num_groups: usize,
    pub group_spacing: Spacing,
    pub print_range: HexdumpRange
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexOffset {
    Relative(usize),
    Absolute(usize)
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
            group_size: GroupSize::Byte,
            byte_spacing: Spacing::Normal,
            num_groups: 16,
            group_spacing: Spacing::Normal,
            print_range: HexdumpRange { skip: 0, limit: None }
        }
    }
}

impl HexdumpOptions {
    fn row_width(&self) -> usize {
        let m = self.group_size.element_count();
        let single_group_width = (2 * m) + (self.byte_spacing.visual_width() * (m - 1));
        let gg = (single_group_width * self.num_groups) + (self.group_spacing.visual_width() * (self.num_groups - 1));
        gg
    }

    fn elt_width(&self) -> usize {
        self.group_size.element_count() * self.num_groups
    }

    fn spacing_for_element_index(&self, idx: usize) -> Spacing {
        match self.group_size {
            GroupSize::Byte => self.group_spacing,
            gs => {
                let s = gs.element_count();
                if idx % s == s - 1 {
                    self.group_spacing
                } else {
                    self.byte_spacing
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

pub fn hexdump_into_rr<
    W: WriteHexdump, 
    Reader: GroupedReader<N>,
    const N: usize
>(w: &mut W, reader: &mut Reader, options: HexdumpOptions) -> Result<(), W::Error> {
    let start = options.print_range.skip;

    let mut elide: Option<(usize, u8)> = None;

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
        let mut b = Vec::with_capacity(options.row_width());

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

    let write_row_idx = |writer: &mut W, idx: Option<usize>| {
        match idx {
            Some(i) => {
                let mut oo = [0u8; 8];
                let o = oo.as_mut_slice();
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
        let mut v: Vec<u8> = Vec::with_capacity(options.elt_width() + 2);
        v.push(b'|');
        for i in 0..options.elt_width() {
            v.push(match aligned_index_into(c, i) {
                Some(ch) => if is_printable(ch as char) { ch } else { b'.' },
                None => b' '
            });
        }
        v.push(b'|');
        let s = std::str::from_utf8(&v).unwrap();
        writer.write_hexdump_str(&s)?;
        Ok::<(), W::Error>(())
    };

    let mut bytebuf: Vec<u8> = vec![0u8; options.elt_width()];

    let mut i = start;
    let mut row_i = 0usize;

    let mut elide: Option<(usize, u8)> = None;

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

        fn is_right(&self) -> bool {
            match self {
                &Self::Right(_) => true,
                _ => false
            }
        }
    }

    // if start, advance reader by floored of offset
    // where floored of offset is the next lowest aligned offset
    // i.e. 7 -> 4, 9 -> 8, etc

    let floored_start = (start / options.elt_width()) * options.elt_width();

    if floored_start > 0 {
        let adv = floored_start / reader.size();
        for _ in 0..adv {
            let _ = reader.read_next(Endianness::BigEndian);
        }
    }

    let mut row_index = floored_start;

    loop {
        bytebuf.clear();
        let x = if !options.align || i % options.elt_width() == 0 {
            // We are starting on an even offset, no need to handle alignment
            for _ in 0..options.num_groups {
                if let Some(x) = reader.read_next(Endianness::BigEndian) {
                    bytebuf.extend_from_slice(&x);
                }
            }

            if bytebuf.len() == options.elt_width() {
                RowBuf::Full(&bytebuf)
            } else {
                RowBuf::Left(&bytebuf)
            }
        } else {
            for _ in 0..options.num_groups {
                if let Some(x) = reader.read_next(Endianness::BigEndian) {
                    bytebuf.extend_from_slice(&x);
                }
            }

            let st = options.elt_width() - (i % options.elt_width());

            if st > 0 {
                RowBuf::Right(if bytebuf.len() > 0 { &bytebuf[st..] } else { &bytebuf })
            } else {
                RowBuf::Full(&bytebuf)
            }
        };

        if x.is_empty() { break; }

        if options.omit_equal_rows {
            match elide {
                Some((elide_start, search_char)) => {
                    let all_eq = x.is_full() && x.as_slice().iter().all(|ch| *ch == search_char);
                    if all_eq { i += x.len(); row_i += 1; row_index += options.elt_width(); continue; }
                    else {
                        let cc = vec![search_char; options.elt_width()];
                        let xx = RowBuf::Full(&cc);
                        write_row_idx(w, Some(elide_start * options.elt_width()))?;
                        write_row(w, &xx);
                        if options.print_ascii {
                            w.write_hexdump_str(" ")?;
                            write_row_ascii(w, &xx)?;
                        }
                        w.write_hexdump_str("\n")?;

                        if row_i - elide_start >= 3 {
                            write_row_idx(w, None)?;
                            w.write_hexdump_str("\n")?;
                        }

                        write_row_idx(w, Some((row_i-1) * options.elt_width()))?;
                        write_row(w, &xx);
                        if options.print_ascii {
                            w.write_hexdump_str(" ")?;
                            write_row_ascii(w, &xx)?;
                        }
                        w.write_hexdump_str("\n")?;
                        elide = None;
                    }
                }
                None => {
                    let search_char = x.as_slice()[0];
                    let all_eq = x.is_full() && x.as_slice().iter().all(|ch| *ch == search_char);
                    if all_eq { elide = Some((row_i, search_char)); i += x.len(); row_index += options.elt_width(); row_i += 1; continue; }
                }
            }
        }

        write_row_idx(w, Some(row_index))?;
        write_row(w, &x);
        if options.print_ascii {
            w.write_hexdump_str(" ")?;
            write_row_ascii(w, &x)?;
        }
        w.write_hexdump_str("\n")?;
        i += x.len();
        row_i += 1;
        row_index += options.elt_width();
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
    fn next_n<'buf>(&mut self, buf: &'buf mut[u8], n: usize) -> Result<&'buf [u8], Self::Error>;
    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error>;
}

impl<'b, T: Iterator<Item = &'b u8>> MyByteReader for T {
    type Error = Infallible;
    fn next_n<'a>(&mut self, buf: &'a mut[u8], n: usize) -> Result<&'a [u8], Self::Error> {
        let mut i = 0;
        while i < n {
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
}