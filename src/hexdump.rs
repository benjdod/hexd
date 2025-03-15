use std::{convert::Infallible, fmt::{Arguments, Debug}, ops::{BitAnd, Bound, RangeBounds, RangeFull, Shr}};

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

pub struct HexdumpOptions {
    pub omit_equal_rows: bool,
    // pub width: usize,
    // pub columns: usize,
    pub uppercase: bool,
    pub print_ascii: bool,
    pub align: bool,
    pub group_size: GroupSize,
    pub byte_spacing: Spacing,
    pub num_groups: usize,
    pub group_spacing: Spacing,
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
            group_spacing: Spacing::Normal
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

fn hexdump_into_rr<
    W: WriteHexdump, 
    Reader: MyByteReader,
    R: RangeBounds<usize>
>(w: &mut W, reader: &mut Reader, options: HexdumpOptions, range: R) -> Result<(), W::Error> {
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Excluded(i) => *i + 1,
        Bound::Included(i) => *i
    };

    let end = match range.end_bound() {
        Bound::Unbounded => None,
        Bound::Excluded(i) => Some(*i - 1),
        Bound::Included(i) => Some(*i)
    };

    let mut elide: Option<(usize, u8)> = None;

    let aligned_index_into = |c: &[u8], i: usize, is_right_align: bool| {
        if c.len() == options.elt_width() {
            Some(c[i])
        } else {
            let offset = options.elt_width() - c.len();
            if is_right_align {
                if i >= offset { Some(c[i-offset]) } else {None}
            } else {
                if i < c.len() { Some(c[i]) } else {None}
            }
        }
    };

    let write_row = |writer: &mut W, c: &[u8], is_right_align: bool| {
        let mut b = Vec::with_capacity(options.row_width());

        for i in 0..options.elt_width() {
            let ch = aligned_index_into(c, i, is_right_align);

            let [hi, lo] = match ch {
                Some(ch) => if options.uppercase { ch.to_hex_upper() } else { ch.to_hex_lower() },
                None => [b' ', b' ']
            };
            b.push(hi);
            b.push(lo);
            let spacing_after = options.spacing_for_element_index(i);
            match spacing_after {
                Spacing::None => { }
                Spacing::Normal => {
                    b.push(b' ');
                }
                Spacing::Wide => {
                    b.extend_from_slice(&[b' ', b' ']);
                }
                Spacing::UltraWide => {
                    b.extend_from_slice(&[b' ', b' ', b' ', b' ']);
                }
            }
        }

        let s = std::str::from_utf8(b.as_slice()).unwrap();

        writer.write_hexdump_str(s).unwrap();
    };

    let write_row_idx = |writer: &mut W, idx: Option<usize>| {
        match idx {
            Some(i) => {
                // writer.write_hexdump_fmt(format_args!("{:#010X}    ", i))
                let mut oo = [0u8; 8];
                let o = oo.as_mut_slice();
                number_to_hex(i, o, false);
                writer.write_hexdump_str(std::str::from_utf8(o).unwrap())?;
                writer.write_hexdump_str("    ")
            }
            None => {
                // writer.write_hexdump_str("              ")
                writer.write_hexdump_str(" --snip--     ")// 
            }
        }
    };

    let is_printable = |ch:char| {
        ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch == ' '
    };

    let write_row_ascii = |writer: &mut W, c: &[u8], is_right_align: bool| {
        let mut v: Vec<u8> = Vec::with_capacity(options.elt_width() + 2);
        v.push(b'|');
        for i in 0..options.elt_width() {
            v.push(match aligned_index_into(c, i, is_right_align) {
                Some(ch) => if is_printable(ch as char) { ch } else { b'.' },
                None => b' '
            });
        }
        v.push(b'|');
        let s = std::str::from_utf8(&v).unwrap();
        writer.write_hexdump_str(&s)?;
        Ok::<(), W::Error>(())
    };

    if start > 0 {
        reader.skip_n(start).unwrap();
    }

    let mut bytebuf: Vec<u8> = vec![0u8; options.elt_width()];

    let mut i = start;
    let mut row_i = 0usize;

    let mut elide: Option<(usize, u8)> = None;

    loop {
        let (row_index, row_len, c) = if options.align {
            let csize = if i % options.elt_width() == 0 { options.elt_width() } else { options.elt_width() - i % options.elt_width() };
            let row_index = if i % options.elt_width() == 0 { i } else { (i / options.elt_width()) * options.elt_width() };
            let c = reader.next_n(&mut bytebuf, csize).unwrap();
            (row_index, csize, c)
        } else {
            (i, options.elt_width(), reader.next_n(&mut bytebuf, options.elt_width()).unwrap())
        };
        if c.len() == 0 { break; }

        let is_full_row = options.elt_width() == row_len;
        let is_right_align = row_index != i;

        if options.omit_equal_rows {
            match elide {
                Some((elide_start, search_char)) => {
                    let all_eq = is_full_row && c.iter().all(|ch| *ch == search_char);
                    if all_eq { i += row_len; row_i += 1; continue; }
                    else {
                        let cc = vec![search_char; options.elt_width()];
                        write_row_idx(w, Some(elide_start * options.elt_width()))?;
                        write_row(w, &cc, is_right_align);
                        if options.print_ascii {
                            w.write_hexdump_str(" ")?;
                            write_row_ascii(w, &cc, is_right_align)?;
                        }
                        w.write_hexdump_str("\n")?;

                        if row_i - elide_start >= 3 {
                            write_row_idx(w, None)?;
                            w.write_hexdump_str("\n")?;
                        }

                        write_row_idx(w, Some((row_i-1) * options.elt_width()))?;
                        write_row(w, &cc, is_right_align);
                        if options.print_ascii {
                            w.write_hexdump_str(" ")?;
                            write_row_ascii(w, &cc, is_right_align)?;
                        }
                        w.write_hexdump_str("\n")?;
                        elide = None;
                    }
                }
                None => {
                    let search_char = c[0];
                    let all_eq = is_full_row && c.iter().all(|ch| *ch == search_char);
                    if all_eq { elide = Some((row_i, search_char)); i += row_len; row_i += 1; continue; }
                }
            }
        }

        write_row_idx(w, Some(row_index))?;
        write_row(w, c, is_right_align);
        if options.print_ascii {
            w.write_hexdump_str(" ")?;
            write_row_ascii(w, c, is_right_align)?;
        }
        w.write_hexdump_str("\n")?;
        i += row_len;
        row_i += 1;
    }

    return Ok(());

    // for (i, c) in s.chunks(options.width).enumerate() {
    //     if options.omit_equal_rows {
    //         match elide {
    //             Some((elide_start, search_char)) => {
    //                 let all_eq = c.iter().all(|ch| *ch == search_char);
    //                 if all_eq { continue; }
    //                 else {
    //                     write_row_idx(w, elide_start)?;
    //                     write_row(w, &s[elide_start..(elide_start+options.width)]);
    //                     write_row_ascii(w, &s[elide_start..(elide_start+options.width)])?;
    //                     w.write_hexdump_str("   ...\n")?;
    //                     write_row_idx(w, (i-1) * options.width)?;
    //                     write_row(w, &s[elide_start..(elide_start+options.width)]);
    //                     write_row_ascii(w, &s[elide_start..(elide_start+options.width)])?;
    //                     elide = None;
    //                 }
    //             }
    //             None => {
    //                 let search_char = c[0];
    //                 let all_eq = c.iter().all(|ch| *ch == search_char);
    //                 if all_eq { elide = Some((i * options.width, search_char)); continue; }
    //             }
    //         }
    //     }

    //     write_row_idx(w, start + i * options.width)?;
    //     write_row(w, c);
    //     write_row_ascii(w, c)?;
    // }
    // Ok(())
}

pub trait DoHexdump {
    fn hexdump_into<W, R>(&self, writeable: &mut W, options: HexdumpOptions, range: R) where W: WriteHexdump, R: RangeBounds<usize>;
}

// impl DoHexdump for &[u8]{
//     fn hexdump_into<W, R>(&self, writeable: &mut W, options: HexdumpOptions, range: R) where W: WriteHexdump, R: RangeBounds<usize> {
//         hexdump_into(writeable, &self, options, range).unwrap()
//     }
// }

// impl DoHexdump for [u8] {
//     fn hexdump_into<W, R>(&self, writeable: &mut W, options: HexdumpOptions, range: R) where W: WriteHexdump, R: RangeBounds<usize> {
//         hexdump_into(writeable, &self, options, range).unwrap()
//     }
// }

// impl<T: AsRef<[u8]>> DoHexdump for T {
//     fn hexdump_into<W, R>(&self, writeable: &mut W, options: HexdumpOptions, range: R) where W: WriteHexdump, R: RangeBounds<usize> {
//         hexdump_into(writeable, self.as_ref(), options, range).unwrap()
//     }
// }

pub struct HexDumper<'a, T: DoHexdump, R: RangeBounds<usize>>(&'a T, HexdumpOptions, R);

impl<'a, T: DoHexdump, R: RangeBounds<usize>> HexDumper<'a, T, R> {
    pub fn print(self) {
        self.0.hexdump_into(&mut HexdumpIoWriter(std::io::stdout()), self.1, self.2);
    }

    pub fn eprint(self) {
        self.0.hexdump_into(&mut HexdumpIoWriter(std::io::stderr()), self.1, self.2);
    }

    pub fn to_string(self) {
        self.0.hexdump_into(&mut HexdumpFmtWriter(String::new()), self.1, self.2);
    }

    pub fn options(self, options: HexdumpOptions) -> Self {
        HexDumper(self.0, options, self.2)
    }
}

pub trait AsHexDumper<'a, T: DoHexdump> {
    fn hexdump(&'a self) -> HexDumper<'a, T, RangeFull>;
    fn hexdump_range<R: RangeBounds<usize>>(&'a self, range: R) -> HexDumper<'a, T, R>;
}

impl<'a, T: DoHexdump> AsHexDumper<'a, T> for T {
    fn hexdump(&'a self) -> HexDumper<'a, T, RangeFull> {
        HexDumper(&self, HexdumpOptions::default(), ..)
    }

    fn hexdump_range<R: RangeBounds<usize>>(&'a self, range: R) -> HexDumper<'a, T, R> {
        HexDumper(&self, HexdumpOptions::default(), range)
    }
}

pub trait MyByteReader {
    type Error: Debug;
    fn next_n<'buf>(&mut self, buf: &'buf mut[u8], n: usize) -> Result<&'buf [u8], Self::Error>;
    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error>;
    fn hexdumppp<W: WriteHexdump>(&mut self, w: &mut W) where Self: Sized {
        hexdump_into_rr(w, self, HexdumpOptions::default(), ..).unwrap();
    }
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