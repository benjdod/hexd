use hexd::{options::{HexdOptions, HexdOptionsBuilder}, IntoHexd};

pub struct IntRenderTestCase<T: Copy> {
    pub input: Vec<T>,
    pub output: &'static str,
    pub endianness: hexd::options::Endianness,
}

pub struct ValSequence<T: Copy> {
    pub ranges: Vec<(T, usize)>,
    pub range_index: usize,
    pub elt_index: usize
}

impl<T: Copy> ValSequence<T> {
    fn new(ranges: Vec<(T, usize)>) -> Self {
        Self { ranges, range_index: 0, elt_index: 0 }
    }
    fn single(val: T, count: usize) -> Self {
        Self::new(vec![(val, count)])
    }
}

impl<T: Copy> Iterator for ValSequence<T> {
    type Item = T;
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let sh = self.ranges.iter().map(|(_, len)| len).sum();
        (sh, Some(sh))
    }
}

pub struct RenderTestCase<T> {
    pub input: T,
    pub output: &'static str,
    pub options: HexdOptions
}

pub struct ByteSequence {
    pub ranges: Vec<(u8, usize)>,
    pub range_index: usize,
    pub elt_index: usize
}

impl ByteSequence {
    pub fn new(ranges: Vec<(u8, usize)>) -> Self {
        Self { ranges, range_index: 0, elt_index: 0 }
    }
}

impl Iterator for ByteSequence {
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
    fn size_hint(&self) -> (usize, Option<usize>) {
        let sh = self.ranges.iter().map(|(_, len)| len).sum();
        (sh, Some(sh))
    }
}

pub fn test_byte_case(test: RenderTestCase<ByteSequence>) -> anyhow::Result<()> {
    // Given
    let RenderTestCase { 
        input, 
        output, 
        options 
    } = test;

    // When
    let dump_lines = input.hexd().with_options(options).dump_to::<Vec<String>>();
    let dump = dump_lines.join("");

    // Then
    similar_asserts::assert_eq!(output, &dump, "hexdump output did not equal expected value");
    Ok(())
}


#[macro_export]
macro_rules! byte_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> anyhow::Result<()> {
            crate::common::test_byte_case($value)
        }
    )*
    };
}