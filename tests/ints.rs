use std::vec;
use indoc::indoc;

use hexd::AsHexdGrouped;

pub struct ValSequence<T: Copy> {
    ranges: Vec<(T, usize)>,
    range_index: usize,
    elt_index: usize
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

#[test]
fn positive_i16_be_is_grouped_correctly() {
    // Given
    let v = vec![0x72f0i16; 32];

    // When
    let dump = v.as_hexd(hexd::options::Endianness::BigEndian).dump_to::<String>();

    // Then
    similar_asserts::assert_eq!(indoc! {"
            00000000: 72F0 72F0 72F0 72F0 72F0 72F0 72F0 72F0 |r.r.r.r.r.r.r.r.|
            *
            00000030: 72F0 72F0 72F0 72F0 72F0 72F0 72F0 72F0 |r.r.r.r.r.r.r.r.|
        "},
        &dump,
    );
}

#[test]
fn negative_i16_be_is_grouped_correctly() {
    // Given
    let v = vec![-0x79c2i16; 32];

    // When
    let dump = v.as_hexd(hexd::options::Endianness::BigEndian).dump_to::<String>();

    // Then
    similar_asserts::assert_eq!(indoc! {"
            00000000: 863E 863E 863E 863E 863E 863E 863E 863E |.>.>.>.>.>.>.>.>|
            *
            00000030: 863E 863E 863E 863E 863E 863E 863E 863E |.>.>.>.>.>.>.>.>|
        "},
        &dump,
    );
}

#[test]
fn positive_i16_le_is_grouped_correctly() {
    // Given
    let v = vec![0x72f0i16; 32];

    // When
    let dump = v.as_hexd(hexd::options::Endianness::LittleEndian).dump_to::<String>();

    // Then
    similar_asserts::assert_eq!(indoc! {"
            00000000: F072 F072 F072 F072 F072 F072 F072 F072 |.r.r.r.r.r.r.r.r|
            *
            00000030: F072 F072 F072 F072 F072 F072 F072 F072 |.r.r.r.r.r.r.r.r|
        "},
        &dump,
    );
}

#[test]
fn negative_i16_le_is_grouped_correctly() {
    // Given
    let v = vec![-0x79c2i16; 32];

    // When
    let dump = v.as_hexd(hexd::options::Endianness::LittleEndian).dump_to::<String>();

    // Then
    similar_asserts::assert_eq!(indoc! {"
            00000000: 3E86 3E86 3E86 3E86 3E86 3E86 3E86 3E86 |>.>.>.>.>.>.>.>.|
            *
            00000030: 3E86 3E86 3E86 3E86 3E86 3E86 3E86 3E86 |>.>.>.>.>.>.>.>.|
        "},
        &dump,
    );
}