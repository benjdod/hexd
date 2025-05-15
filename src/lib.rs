#![doc = include_str!("../README.md")]

use std::{
    cmp::{max, min}, fmt::Debug, io::Write
};

use options::{
    Endianness, Grouping, HexdOptions, HexdOptionsBuilder, IndexOffset, LeadingZeroChar, Spacing,
};
use reader::{
    ByteSliceReader, EndianBytes, GroupedIteratorReader, GroupedSliceByteReader, IoReader, IteratorByteReader, ReadBytes
};
use writer::{IOWriter, WriteHexdump};

/// All [`Hexd`] options.
pub mod options;

/// A collection of [reader](reader::ReadBytes) types that wrap common data types.
pub mod reader;

/// The [`WriteHexdump`] trait and several foreign type implementations.
pub mod writer;

trait ToHex {
    fn to_hex_lower(self) -> [u8; 2];
    fn to_hex_upper(self) -> [u8; 2];
}

const UPPER_LUT: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
];
const LOWER_LUT: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

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
fn trunc_ceil_usize(n: usize, trunc: usize) -> usize {
    let rem = n / trunc;
    match rem {
        0 => rem * trunc,
        _ => (rem + 1) * trunc,
    }
}

trait HexVisualWidth {
    fn hex_visual_width(&self) -> usize;
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
    len: usize,
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
        Self {
            buffer: [0u8; N],
            len: 0,
        }
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

#[derive(Debug, Clone)]
struct RowBuffer {
    buffer: StackBuffer<MAX_BUFFER_SIZE>,
    length: usize,
    row_index: usize,
    elt_index: usize,
}

impl RowBuffer {
    fn is_right_aligned(&self) -> bool {
        self.elt_index != self.row_index
    }
}

const MAX_BUFFER_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq)]
enum HexdumpLineIteratorState {
    NotStarted,
    InProgress,
    Completed,
}

struct HexdumpLineIterator<R: ReadBytes> {
    reader: R,
    index: usize,
    options: HexdOptions,
    state: HexdumpLineIteratorState,
    elision_match: Option<ElisionMatch>,
}

#[derive(Debug, Clone)]
struct ElisionMatch {
    buffer: StackBuffer<MAX_BUFFER_SIZE>,
}

impl ElisionMatch {
    fn try_match(row: &RowBuffer, options: &HexdOptions) -> Option<Self> {
        let buffer = &row.buffer;
        match options.grouping {
            _ if buffer.len != options.elt_width() => None,
            Grouping::Ungrouped {
                byte_count: _,
                spacing: _,
            } => {
                let sc = buffer.buffer[0];
                if buffer.as_slice().iter().all(|b| *b == sc) {
                    Some(ElisionMatch {
                        buffer: buffer.clone(),
                    })
                } else {
                    None
                }
            }
            Grouping::Grouped {
                group_size,
                num_groups: _,
                byte_spacing: _,
                group_spacing: _,
            } => {
                let group_size = group_size.element_count();
                let s = &buffer.as_slice()[..group_size];
                if s.chunks(group_size).all(|chunk| chunk == s) {
                    Some(ElisionMatch {
                        buffer: buffer.clone(),
                    })
                } else {
                    None
                }
            }
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

impl<'a, R: ReadBytes> HexdumpLineIterator<R> {
    pub fn new(reader: R, options: HexdOptions) -> Self {
        Self {
            reader,
            index: 0,
            options,
            state: HexdumpLineIteratorState::NotStarted,
            elision_match: None,
        }
    }

    fn read_into_buffer(&mut self, len: usize) -> Result<RowBuffer, R::Error> {
        let mut buffer = StackBuffer::<MAX_BUFFER_SIZE>::new();

        let actually_read_len = {
            let n = self
                .reader
                .next_n(&mut buffer.as_mut_slice()[..len])?;
            n.len()
        };

        buffer.len += actually_read_len;

        let o = RowBuffer {
            buffer,
            length: actually_read_len,
            row_index: self.calculate_row_index(),
            elt_index: self.index,
        };
        self.index += actually_read_len;
        self.state = HexdumpLineIteratorState::InProgress;
        Ok(o)
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
    Row(RowBuffer),
}

impl<R: ReadBytes> Iterator for HexdumpLineIterator<R> {
    type Item = Result<LineIteratorResult, R::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let st = self.state;
        match st {
            HexdumpLineIteratorState::NotStarted | HexdumpLineIteratorState::InProgress => {
                if self.state == HexdumpLineIteratorState::NotStarted
                    && self.options.print_range.skip > 0
                {
                    self.reader.skip_n(self.options.print_range.skip).unwrap();
                    self.index = self.options.print_range.skip;
                }

                let read_len = {
                    let limit = self
                        .options
                        .print_range
                        .limit
                        .map(|limit| {
                            if self.index < limit {
                                limit - self.index
                            } else {
                                0
                            }
                        })
                        .unwrap_or(self.options.elt_width());

                    let ew = min(self.options.elt_width(), limit);

                    if self.state == HexdumpLineIteratorState::NotStarted && self.options.align {
                        match self.options.print_range.length() {
                            Some(len) if len < self.options.elt_width() => len,
                            _ => ew - (self.index % ew),
                        }
                    } else {
                        ew
                    }
                };

                self.state = HexdumpLineIteratorState::InProgress;
                let rowbuffer = self.read_into_buffer(read_len);

                if let Err(e) = rowbuffer {
                    return Some(Err(e));
                }

                let rowbuffer = rowbuffer.unwrap();

                if self.options.autoskip {
                    if let Some(em) = &self.elision_match {
                        if em.matches(&rowbuffer, &self.options) {
                            return Some(Ok(LineIteratorResult::Elided(rowbuffer)));
                        } else {
                            self.elision_match = None;
                        }
                    }
                    if let Some(elision_match) = ElisionMatch::try_match(&rowbuffer, &self.options)
                    {
                        self.elision_match = Some(elision_match);
                    }
                }
                if rowbuffer.length > 0 {
                    Some(Ok(LineIteratorResult::Row(rowbuffer)))
                } else {
                    self.state = HexdumpLineIteratorState::Completed;
                    None
                }
            }
            HexdumpLineIteratorState::Completed => None,
        }
    }
}

struct HexdumpLineWriter<R: ReadBytes, W: WriteHexdump> {
    line_iterator: HexdumpLineIterator<R>,
    writer: W,
    elided_row: Option<(RowBuffer, usize)>,
    str_buffer: StackBuffer<512>,
    options: HexdOptions,
    flush_idx: usize,
}

enum HexdError<R, W> {
    Read(R),
    Write(W)
}

impl<R: ReadBytes, W: WriteHexdump> HexdumpLineWriter<R, W> {
    fn new(reader: R, writer: W, options: HexdOptions) -> Self {
        let line_iterator = HexdumpLineIterator::new(reader, options.clone());
        Self {
            line_iterator,
            writer,
            elided_row: None,
            str_buffer: StackBuffer::<512>::new(),
            options,
            flush_idx: 0,
        }
    }

    fn do_hexdump(mut self) -> W::Output {
        let r = self.do_hexdump_internal();
        let ll = match r {
            Ok(_) => Ok(self.writer),
            Err(HexdError::Write(e)) => Err(e),
            _ => panic!("unimplemented")
        };
        WriteHexdump::consume(ll)
    }

    fn do_hexdump_internal(&mut self) -> Result<(), HexdError<R::Error, W::Error>> {
        let mut i = 0usize;
        while let Some(r) = self.line_iterator.next() {
            let r = r.map_err(HexdError::Read)?;
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
                }
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

            // let row_index = (i - 1) * self.options.elt_width();
            let row_index = self.line_iterator.index - self.options.elt_width();

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
        if !self.options.show_index {
            return;
        }

        let v_index = match self.options.index_offset {
            IndexOffset::Absolute(o) => {
                row_index - min(row_index, self.options.print_range.skip) + o
            }
            IndexOffset::Relative(o) => row_index + o,
        };

        let bytes = &v_index.to_be_bytes();
        let bl = bytes.len();

        let slice = self
            .line_iterator
            .reader
            .total_byte_hint()
            .map(|h| match self.options.index_offset {
                IndexOffset::Absolute(a) => a + h,
                IndexOffset::Relative(r) => self.options.print_range.skip + r + h,
            })
            .map(|h| (h.hex_visual_width() + 1) / 2)
            .map(|h| max(h, 4))
            .map(|h| &bytes[(bl - h)..])
            .unwrap_or_else(|| {
                &bytes[(bl - max(4, trunc_ceil_usize(v_index.hex_visual_width() / 2, 2)))..]
            });

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

    #[inline]
    fn bchar_for_u8(&self, b: u8) -> u8 {
        if self.options.uppercase {
            UPPER_LUT[b as usize]
        } else {
            LOWER_LUT[b as usize]
        }
    }

    fn write_row_bytes(&mut self, row: &RowBuffer) {
        let elt_width = self.options.elt_width();
        for i in 0..elt_width {
            self.write_byte(self.read_row_byte_aligned(row, i));
            if i != elt_width - 1 || self.options.show_ascii {
                self.str_buffer
                    .extend_from_slice(self.options.grouping.spacing_for_index(i).as_spaces());
            }
        }

        if self.options.show_ascii && self
            .options
            .grouping
            .spacing_for_index(self.options.elt_width() - 1)
            == Spacing::None
        {
            self.str_buffer.push(b' ');
        }
    }

    fn write_byte(&mut self, b: Option<u8>) {
        match (self.options.base, b) {
            (options::Base::Binary, Some(b)) => {
                let chars = [
                    self.bchar_for_u8((b >> 7) & 1),
                    self.bchar_for_u8((b >> 6) & 1),
                    self.bchar_for_u8((b >> 5) & 1),
                    self.bchar_for_u8((b >> 4) & 1),
                    self.bchar_for_u8((b >> 3) & 1),
                    self.bchar_for_u8((b >> 2) & 1),
                    self.bchar_for_u8((b >> 1) & 1),
                    self.bchar_for_u8((b >> 0) & 1),
                ];
                self.str_buffer.extend_from_slice(&chars);
            }
            (options::Base::Binary, None) => {
                self.str_buffer.extend_from_slice(b"        ");
            }

            (options::Base::Octal(lzc), Some(b)) => {
                let lead_char: u8 = lzc.into();
                let cc = [(b >> 6) & 0x7, (b >> 3) & 0x7, (b >> 0) & 0x7];

                let chars = [
                    if cc[0] == 0 && cc[1] != 0 {
                        lead_char
                    } else {
                        self.bchar_for_u8(cc[0])
                    },
                    if cc[0] == 0 && cc[1] == 0 && cc[2] != 0 {
                        lead_char
                    } else {
                        self.bchar_for_u8(cc[1])
                    },
                    self.bchar_for_u8(cc[2]),
                ];
                self.str_buffer.extend_from_slice(&chars);
            }
            (options::Base::Octal(_), None) => {
                self.str_buffer.extend_from_slice(b"   ");
            }

            (options::Base::Decimal(lzc), Some(b)) => {
                let lead_char: u8 = lzc.into();
                let cc = [(b / 100) % 10, (b / 10) % 10, (b / 1) % 10];

                let chars = [
                    if cc[0] == 0 {
                        lead_char
                    } else {
                        self.bchar_for_u8(cc[0])
                    },
                    if cc[0] == 0 && cc[1] == 0 && cc[2] != 0 {
                        lead_char
                    } else {
                        self.bchar_for_u8(cc[1])
                    },
                    self.bchar_for_u8(cc[2]),
                ];
                self.str_buffer.extend_from_slice(&chars);
            }
            (options::Base::Decimal(_), None) => {
                self.str_buffer.extend_from_slice(b"   ");
            }

            (options::Base::Hex, Some(b)) => {
                let chars = [
                    self.bchar_for_u8((b >> 4) & 0xf),
                    self.bchar_for_u8((b >> 0) & 0xf),
                ];
                self.str_buffer.extend_from_slice(&chars);
            }
            (options::Base::Hex, None) => {
                self.str_buffer.extend_from_slice(b"  ");
            }
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
        if !self.options.show_ascii {
            return;
        }

        self.str_buffer.push(b'|');
        for i in 0..self.options.elt_width() {
            let b = self.read_row_byte_aligned(row, i).unwrap_or(b' ');
            self.str_buffer.push(if Self::is_printable_char(b as char) {
                b
            } else {
                b'.'
            });
        }
        self.str_buffer.push(b'|');
    }

    #[inline]
    fn is_printable_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch.is_ascii_punctuation() || ch == ' '
    }

    #[inline]
    fn flush_line(&mut self) -> Result<(), HexdError<R::Error, W::Error>> {
        if self.str_buffer.len > 0 {
            self.str_buffer.push(b'\n');
        }
        let s = self.str_buffer.as_str();
        if s.len() > 0 {
            self.writer.write_str(s).map_err(HexdError::Write)?;
            self.writer.line_end().map_err(HexdError::Write)?;
        }

        self.flush_idx += 1;

        self.str_buffer.clear();
        Ok(())
    }
}

/// Performs hexdumps.
///
/// Typically this struct is not constructed directly. Instead,
/// you can use the [`AsHexd`] or [`IntoHexd`] traits to create a [`Hexd`] instance
/// from a variety of types.
pub struct Hexd<R: ReadBytes> {
    reader: R,
    options: HexdOptions,
}

impl<R: ReadBytes> Hexd<R> {
    /// Construct a new [`Hexd`] instance with the given reader and [default options](HexdOptions::default).
    pub fn new(reader: R) -> Self {
        Hexd {
            reader,
            options: HexdOptions::default(),
        }
    }

    /// Construct a new [`Hexd`] instance with the given reader and options.
    pub fn new_with_options(reader: R, options: HexdOptions) -> Self {
        Hexd { reader, options }
    }

    /// Print a hexdump to `stdout`. This method is synonymous with [`print`](Hexd::print).
    ///
    /// ```
    /// use hxd::AsHexd;
    ///
    /// let v = [0u8; 64];
    ///
    /// v.hexd().dump(); // print a hexdump
    /// ```
    pub fn dump(self) {
        self.dump_io(std::io::stdout())
            .expect("could not print hexdump to stdout");
    }

    /// Print a hexdump to `stderr`. This method is synonymous with [`print_err`](Hexd::print_err).
    ///
    /// ```
    /// use hxd::AsHexd;
    ///
    /// let v = [0u8; 64];
    ///
    /// v.hexd().dump_err(); // print a hexdump to stderr
    /// ```
    pub fn dump_err(self) {
        self.dump_io(std::io::stderr())
            .expect("could not print hexdump to stderr");
    }

    /// Construct a default instance of `W` and write a hexdump to it, returning its output.
    ///
    /// ```
    /// use hxd::AsHexd;
    ///
    /// let dump = [0u8; 64].hexd().dump_to::<String>();
    /// ```
    pub fn dump_to<W: WriteHexdump + Default>(self) -> W::Output {
        let hlw = HexdumpLineWriter::new(self.reader, W::default(), self.options);
        hlw.do_hexdump()
    }

    /// Write a hexdump to an instance of `W` and return its output.
    ///
    /// ```
    /// use hxd::AsHexd;
    ///
    /// let v: Vec<String> = Vec::new();
    /// let dump = [0u8; 64].hexd().dump_into(v);
    /// ```
    pub fn dump_into<W: WriteHexdump>(self, writer: W) -> W::Output {
        let hlw = HexdumpLineWriter::new(self.reader, writer, self.options);
        hlw.do_hexdump()
    }

    /// Write a hexdump to an object that is [Write].
    /// The object is wrapped in a [BufWriter](std::io::BufWriter)
    /// for improved performance.
    ///
    /// ```no_run
    /// use hxd::AsHexd;
    /// use std::fs::OpenOptions;
    ///
    /// let v = [0u8; 64];
    ///
    /// let f = OpenOptions::new()
    ///     .write(true)
    ///     .create(true)
    ///     .open("hexdump.txt")
    ///     .unwrap();
    ///
    /// v.hexd().dump_io(f).expect("could not write hexdump to file");
    /// ```
    pub fn dump_io<W: Write>(self, write: W) -> Result<(), std::io::Error> {
        let hlw = HexdumpLineWriter::new(self.reader, IOWriter::new(write), self.options);
        hlw.do_hexdump()
    }

    /// Write a hexdump to an object that is [Write].
    /// Unlike [`Self::dump_io`], this method does not wrap the object in a
    /// [BufWriter](std::io::BufWriter).
    ///
    /// ```no_run
    /// use hxd::AsHexd;
    /// use std::fs::OpenOptions;
    ///
    /// let v = [0u8; 64];
    ///
    /// let f = OpenOptions::new()
    ///     .write(true)
    ///     .create(true)
    ///     .open("hexdump.txt")
    ///     .unwrap();
    ///
    /// v.hexd().dump_io(f).expect("could not write hexdump to file");
    /// ```
    pub fn dump_io_unbuffered<W: Write>(self, write: W) -> Result<(), std::io::Error> {
        let hlw =
            HexdumpLineWriter::new(self.reader, IOWriter::new_unbuffered(write), self.options);
        hlw.do_hexdump()
    }

    /// Print a hexdump to [`stdout`](std::io::Stdout). This method is synonymous with [`print`](Hexd::print).
    pub fn print(self) {
        self.dump_io(std::io::stdout())
            .expect("could not print hexdump to stdout");
    }

    /// Print a hexdump to [`stderr`](std::io::Stderr).
    pub fn print_err(self) {
        self.dump_io(std::io::stderr())
            .expect("could not print hexdump to stderr");
    }
}

/// [`Hexd`] implements [`HexdOptionsBuilder`] to allow for fluent
/// configuration.
impl<R: ReadBytes> HexdOptionsBuilder for Hexd<R> {
    fn map_options<F: FnOnce(HexdOptions) -> HexdOptions>(self, f: F) -> Self {
        Hexd {
            options: f(self.options),
            ..self
        }
    }
}

impl From<LeadingZeroChar> for u8 {
    fn from(value: LeadingZeroChar) -> Self {
        match value {
            LeadingZeroChar::Space => b' ',
            LeadingZeroChar::Underscore => b'_',
            LeadingZeroChar::Zero => b'0',
        }
    }
}

/// This trait yields an owning version of [`Hexd`].
pub trait IntoHexd<R: ReadBytes>: Sized {
    fn into_hexd(self) -> Hexd<R>;
    fn hexd(self) -> Hexd<R> {
        self.into_hexd()
    }
}

impl<I: Iterator<Item = u8>> IntoHexd<IteratorByteReader<I>> for I {
    fn into_hexd(self) -> Hexd<IteratorByteReader<I>> {
        Hexd {
            reader: IteratorByteReader::new(self),
            options: HexdOptions::default(),
        }
    }
}

impl<R: std::io::Read> IntoHexd<IoReader<R>> for R {
    fn into_hexd(self) -> Hexd<IoReader<R>> {
        Hexd {
            reader: IoReader::new(self),
            options: HexdOptions::default(),
        }
    }
}

pub trait IntoHexdGrouped<const N: usize>: Sized {
    type Output: ReadBytes;
    /// Construct an instance [`Hexd`] from the current vale
    /// and the given endianness.
    fn into_hexd(self, endianness: Endianness) -> Hexd<Self::Output>;

    /// Construct an instance of [`Hexd`] from
    /// the current value as big-endian bytes.
    /// This is equivalent to calling `self.into_hexd(Endianness::BigEndian)`
    fn into_hexd_be(self) -> Hexd<Self::Output> {
        self.into_hexd(Endianness::BigEndian)
    }

    /// Construct an instance of [`Hexd`] from
    /// the current value as little-endian bytes.
    /// This is equivalent to calling `self.into_hexd(Endianness::LittleEndian)`
    fn into_hexd_le(self) -> Hexd<Self::Output> {
        self.into_hexd(Endianness::LittleEndian)
    }

    /// Construct an instance [`Hexd`] from the current vale
    /// and the given endianness.
    fn hexd(self, endianness: Endianness) -> Hexd<Self::Output> {
        self.into_hexd(endianness)
    }

    /// Construct an instance of [`Hexd`] from
    /// the current value as big-endian bytes.
    /// This is equivalent to calling `self.into_hexd(Endianness::BigEndian)`.
    fn hexd_be(self) -> Hexd<Self::Output> {
        self.into_hexd(Endianness::BigEndian)
    }

    /// Construct an instance of [`Hexd`] from
    /// the current value as little-endian bytes.
    /// This is equivalent to calling `self.into_hexd(Endianness::LittleEndian)`
    fn hexd_le(self) -> Hexd<Self::Output> {
        self.into_hexd(Endianness::LittleEndian)
    }
}

/// This trait can be implemented for reference types to yield
/// a non-owning version of [`Hexd`].
pub trait AsHexd<'a, R: ReadBytes> {
    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value.
    fn as_hexd(&'a self) -> Hexd<R>;

    /// By default, this method simply calls [`as_hexd`](AsHexd::as_hexd).
    /// It is defined for convenience to simplify refactoring to types
    /// implementing [`IntoHexd`].
    fn hexd(&'a self) -> Hexd<R> {
        self.as_hexd()
    }
}

pub trait AsHexdGrouped<'a, R: ReadBytes> {
    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value and the given endianness.
    fn as_hexd(&'a self, endianness: Endianness) -> Hexd<R>;

    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value as big-endian bytes.
    /// This is equivalent to calling `self.as_hexd(Endianness::BigEndian)`
    fn as_hexd_be(&'a self) -> Hexd<R> {
        self.as_hexd(Endianness::BigEndian)
    }

    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value as little-endian bytes.
    /// This is equivalent to calling `self.as_hexd(Endianness::LittleEndian)`
    fn as_hexd_le(&'a self) -> Hexd<R> {
        self.as_hexd(Endianness::LittleEndian)
    }

    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value and the given endianness.
    fn hexd(&'a self, endianness: Endianness) -> Hexd<R> {
        self.as_hexd(endianness)
    }

    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value as big-endian bytes.
    /// This is equivalent to calling `self.as_hexd(Endianness::BigEndian)`
    fn hexd_be(&'a self) -> Hexd<R> {
        self.as_hexd(Endianness::BigEndian)
    }

    /// Construct a non-owning [`Hexd`] from a reference of
    /// the current value as little-endian bytes.
    /// This is equivalent to calling `self.as_hexd(Endianness::LittleEndian)`
    fn hexd_le(&'a self) -> Hexd<R> {
        self.as_hexd(Endianness::LittleEndian)
    }
}

/// Blanket implementation for any type that implements `AsRef<[u8]>`.
/// In other words, any type that can be seen as a slice of `u8` can be
/// quickly made into [`Hexd`].
///
/// ## Examples
/// ```
/// use hxd::AsHexd;
///
/// let v = vec![0u8; 24];
/// let x = [0u8; 4];
/// let s = "greetings earthling!";
///
/// v.as_hexd().dump();
/// x.as_hexd().dump();
/// s.as_hexd().dump();
/// ```
impl<'a, T: AsRef<[u8]>> AsHexd<'a, ByteSliceReader<'a>> for T {
    fn as_hexd(&'a self) -> Hexd<ByteSliceReader<'a>> {
        let slice = self.as_ref();
        let reader = ByteSliceReader::new(slice);
        Hexd {
            reader,
            options: HexdOptions::default(),
        }
    }
}

impl<'a, T: AsRef<[i8]>> AsHexd<'a, GroupedSliceByteReader<'a, i8, 1>> for T {
    fn as_hexd(&'a self) -> Hexd<GroupedSliceByteReader<'a, i8, 1>> {
        let slice = self.as_ref();
        let reader = GroupedSliceByteReader::new(slice, Endianness::BigEndian);
        Hexd {
            reader,
            options: HexdOptions::default(),
        }
    }
}

macro_rules! as_hexd_grouped {
    ($t:ty, $sz:expr, $group_size:expr, $byte_spacing:expr, $num_groups:expr) => {
        impl<'a, T: AsRef<[$t]>> AsHexdGrouped<'a, GroupedSliceByteReader<'a, $t, $sz>> for T {
            fn as_hexd(
                &'a self,
                endianness: Endianness,
            ) -> Hexd<GroupedSliceByteReader<'a, $t, $sz>> {
                let slice = self.as_ref();
                let reader = GroupedSliceByteReader::new(slice, endianness);
                let options = HexdOptions::default().grouping(Grouping::Grouped {
                    group_size: $group_size,
                    byte_spacing: $byte_spacing,
                    num_groups: $num_groups,
                    group_spacing: Spacing::Normal,
                });
                Hexd { reader, options }
            }
        }
    };
}

as_hexd_grouped!(u16, 2, options::GroupSize::Short, Spacing::None, 8);
as_hexd_grouped!(i16, 2, options::GroupSize::Short, Spacing::None, 8);
as_hexd_grouped!(u32, 4, options::GroupSize::Int, Spacing::None, 4);
as_hexd_grouped!(i32, 4, options::GroupSize::Int, Spacing::None, 4);
as_hexd_grouped!(u64, 8, options::GroupSize::Long, Spacing::None, 2);
as_hexd_grouped!(i64, 8, options::GroupSize::Long, Spacing::None, 2);
as_hexd_grouped!(u128, 16, options::GroupSize::ULong, Spacing::Normal, 1);
as_hexd_grouped!(i128, 16, options::GroupSize::ULong, Spacing::Normal, 1);

impl<const N: usize, E: EndianBytes<N>, I: Iterator<Item = E>> IntoHexdGrouped<N> for I {
    type Output = GroupedIteratorReader<E, I, N>;

    fn into_hexd(self, endianness: Endianness) -> Hexd<Self::Output> {
        let reader = GroupedIteratorReader::new(self, endianness);

        let grouping = match N {
            2 => Grouping::Grouped {
                group_size: options::GroupSize::Short,
                byte_spacing: Spacing::None,
                num_groups: 8,
                group_spacing: Spacing::Normal,
            },
            4 => Grouping::Grouped {
                group_size: options::GroupSize::Int,
                byte_spacing: Spacing::None,
                num_groups: 4,
                group_spacing: Spacing::Normal,
            },
            8 => Grouping::Grouped {
                group_size: options::GroupSize::Long,
                byte_spacing: Spacing::None,
                num_groups: 2,
                group_spacing: Spacing::Normal,
            },
            16 => Grouping::Grouped {
                group_size: options::GroupSize::ULong,
                byte_spacing: Spacing::Normal,
                num_groups: 1,
                group_spacing: Spacing::Normal,
            },
            _ => Grouping::default(),
        };

        let options = HexdOptions::default().grouping(grouping);
        Hexd { reader, options }
    }
}