use std::{cmp::min, convert::Infallible, fmt::Debug};

use crate::Endianness;

#[doc(hidden)]
pub trait GroupedReader<const N: usize> {
    fn read_next(&mut self, end: Endianness) -> Option<[u8; N]>;
    fn size(&self) -> usize {
        N
    }
}

pub struct ByteSliceReader<'a> {
    slice: &'a [u8],
    index: usize,
}

impl<'a> ByteSliceReader<'a> {
    pub fn new(slice: &'a [u8]) -> ByteSliceReader<'a> {
        Self {
            slice,
            index: 0usize,
        }
    }
}

impl<'a> ReadBytes for ByteSliceReader<'a> {
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut [u8]) -> Result<&'buf [u8], Self::Error> {
        if self.index >= self.slice.len() {
            return Ok(&[]);
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

/// This trait provides a method to convert
/// integer types into sized byte arrays.
/// Under the hood, implementations for primitive integer
/// types call `to_be_bytes()` or `to_le_bytes()`
/// depending on the [endianness](crate::options::Endianness).
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
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<2> for i16 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 2] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<4> for u32 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 4] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<4> for i32 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 4] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<8> for u64 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 8] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<8> for i64 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 8] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<16> for u128 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 16] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}

impl EndianBytes<16> for i128 {
    fn to_bytes(&self, endianness: Endianness) -> [u8; 16] {
        match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        }
    }
}
pub struct GroupedSliceReader<'a, U: EndianBytes<N>, const N: usize> {
    slice: &'a [U],
    index: usize,
}

pub struct GroupedSliceByteReader<'a, U: EndianBytes<N>, const N: usize> {
    slice: &'a [U],
    elt_index: usize,
    u_index: usize,
    current_elt: Option<[u8; N]>,
    endianness: Endianness,
}

impl<'a, U: EndianBytes<N>, const N: usize> ReadBytes for GroupedSliceByteReader<'a, U, N> {
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut [u8]) -> Result<&'buf [u8], Self::Error> {
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

impl<'a, U: EndianBytes<N>, const N: usize> GroupedSliceByteReader<'a, U, N> {
    pub fn new(slice: &'a [U], endianness: Endianness) -> Self {
        let current_elt = if slice.len() > 0 {
            Some(slice[0].to_bytes(endianness))
        } else {
            None
        };
        Self {
            slice,
            elt_index: 0,
            u_index: 0,
            current_elt,
            endianness,
        }
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

        self.current_elt = if self.elt_index < self.slice.len() {
            Some(self.slice[self.elt_index].to_bytes(self.endianness))
        } else {
            None
        };

        if adv > 0 {
            self.u_index = adv;
        }
    }
}

impl<'a, U: EndianBytes<N>, const N: usize> GroupedSliceReader<'a, U, N> {
    pub fn new(slice: &'a [U]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, U: EndianBytes<N>, const N: usize> GroupedSliceReader<'a, U, N> {
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

impl<'a, const N: usize, U: EndianBytes<N>> GroupedReader<N> for GroupedSliceReader<'a, U, N> {
    fn read_next(&mut self, end: Endianness) -> Option<[u8; N]> {
        self.next(end)
    }
}

pub struct IteratorByteReader<I: Iterator<Item = u8>> {
    iterator: I,
}

impl<I: Iterator<Item = u8>> IteratorByteReader<I> {
    pub fn new(iterator: I) -> Self {
        Self { iterator }
    }
}

impl<I: Iterator<Item = u8>> ReadBytes for IteratorByteReader<I> {
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut [u8]) -> Result<&'buf [u8], Self::Error> {
        let mut i = 0usize;
        while i < buf.len() {
            match self.iterator.next() {
                Some(b) => {
                    buf[i] = b;
                }
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
                return Ok(i);
            }
        }
        Ok(n)
    }
}

pub struct GroupedIteratorReader<U: EndianBytes<N>, I: Iterator<Item = U>, const N: usize> {
    iterator: I,
    current: Option<[u8; N]>,
    index: usize,
    endianness: Endianness,
}

impl<U: EndianBytes<N>, I: Iterator<Item = U>, const N: usize> GroupedIteratorReader<U, I, N> {
    pub fn new(mut iterator: I, endianness: Endianness) -> Self {
        let current = iterator.next().map(|u| u.to_bytes(endianness));
        Self {
            iterator,
            current,
            index: 0,
            endianness,
        }
    }

    pub fn next_byte(&mut self) -> Option<u8> {
        let b = self.current.map(|c| c[self.index]);
        self.index += 1;
        if self.index >= N {
            self.index = 0;
            self.current = self.iterator.next().map(|u| u.to_bytes(self.endianness));
        }
        b
    }
}

impl<U: EndianBytes<N>, I: Iterator<Item = U>, const N: usize> ReadBytes
    for GroupedIteratorReader<U, I, N>
{
    type Error = Infallible;

    fn next_n<'buf>(&mut self, buf: &'buf mut [u8]) -> Result<&'buf [u8], Self::Error> {
        let mut i = 0usize;
        while i < buf.len() {
            if let Some(b) = self.next_byte() {
                buf[i] = b;
                i += 1;
            } else {
                return Ok(&buf[..i]);
            }
        }
        Ok(&buf[..i])
    }

    fn skip_n(&mut self, n: usize) -> Result<usize, Self::Error> {
        for _ in 0..n {
            if self.next_byte().is_none() {
                return Ok(n);
            }
        }
        Ok(n)
    }

    fn total_byte_hint(&self) -> Option<usize> {
        None
    }
}

pub trait ReadBytes {
    type Error: Debug;
    fn next_n<'buf>(&mut self, buf: &'buf mut [u8]) -> Result<&'buf [u8], Self::Error>;

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

impl<'b, T: Iterator<Item = &'b u8>> ReadBytes for T {
    type Error = Infallible;
    fn next_n<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a [u8], Self::Error> {
        let mut i = 0;
        while i < buf.len() {
            match self.next() {
                Some(u) => {
                    buf[i] = *u;
                }
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
                Some(_) => {}
                None => {
                    return Ok(i);
                }
            }
        }
        Ok(n)
    }

    fn total_byte_hint(&self) -> Option<usize> {
        match self.size_hint() {
            (_, Some(upper)) => Some(upper),
            (lower, None) if lower > 0 => Some(lower),
            _ => None,
        }
    }
}
