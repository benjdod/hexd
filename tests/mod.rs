use indoc::indoc;

#[cfg(test)]
mod flush;

#[cfg(test)]
mod ints;

use hexd::{options::{GroupSize, Grouping, HexdOptions, HexdOptionsBuilder, HexdRange, IndexOffset, Spacing, FlushMode}, IntoHexd};

fn test_range_byte_case(test: RenderTestCase<ByteSequence>) -> anyhow::Result<()> {
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

struct RenderTestCase<T> {
    input: T,
    output: &'static str,
    options: HexdOptions
}

pub struct ByteSequence {
    ranges: Vec<(u8, usize)>,
    range_index: usize,
    elt_index: usize
}

impl ByteSequence {
    fn new(ranges: Vec<(u8, usize)>) -> Self {
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

macro_rules! byte_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() -> anyhow::Result<()> {
            test_range_byte_case($value)
        }
    )*
    };
}

fn default_test_options() -> HexdOptions {
    let default_options = HexdOptions {
        base: hexd::options::Base::Hex,
        autoskip: true,
        uppercase: true,
        print_ascii: true,
        align: true,
        grouping: Grouping::default(),
        print_range: HexdRange { skip: 0, limit: None },
        index_offset: IndexOffset::Relative(0),
        flush: FlushMode::End
    };
    default_options
}

fn elision_test_options() -> HexdOptions {
    default_test_options()
        .grouped(GroupSize::Short, Spacing::None, 2, Spacing::Normal)
}

byte_tests! {
    one_line_is_not_elided: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
        ),
        options: elision_test_options()
    },
    two_equal_lines_are_not_elided: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 8)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
            "00000004: 0000 0000 |....|\n",
        ),
        options: elision_test_options()
    },
    three_equal_lines_are_elided: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 12)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
            "*\n",
            "00000008: 0000 0000 |....|\n",
        ),
        options: elision_test_options()
    },
    four_equal_lines_are_elided: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
            "*\n",
            "0000000C: 0000 0000 |....|\n",
        ),
        options: elision_test_options()
    },
    partial_ending_line_is_not_included_in_elision: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 17)
        ]),
        output: indoc!{"
            00000000: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
            00000010: 00        |.   |
        "},
        options: elision_test_options()
    },
    partial_beginning_line_is_not_included_in_elision: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16)
        ]),
        output: indoc!{"
            00000000:        00 |   .|
            00000004: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
        "},
        options: elision_test_options()
            .range(3..)
    },
    two_aligned_elisions_are_handled: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
            (2u8, 16)
        ]),
        output: indoc! {"
            00000000: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
            00000010: 0202 0202 |....|
            *
            0000001C: 0202 0202 |....|
        "},
        options: elision_test_options()
    },
    one_aligned_elision_and_one_false_elision_are_handled: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
            (2u8, 8),
            (3u8, 4)
        ]),
        output: indoc! {"
            00000000: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
            00000010: 0202 0202 |....|
            00000014: 0202 0202 |....|
            00000018: 0303 0303 |....|
        "},
        options: elision_test_options()
    },
    two_unaligned_elisions_are_handled: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 17),
            (2u8, 15)
        ]),
        output: indoc! {"
            00000000: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
            00000010: 0002 0202 |....|
            00000014: 0202 0202 |....|
            *
            0000001C: 0202 0202 |....|
        "},
        options: elision_test_options()
    },
}

// grouping and spacing tests
byte_tests! {
    ungrouped_no_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00000000 |....|
            00000004: 00000000 |....|
            00000008: 00000000 |....|
            0000000C: 00000000 |....|
        "},
        options: default_test_options()
            .autoskip(false)
            .ungrouped(4, crate::Spacing::None)
    },
    ungrouped_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00 00 00 00 |....|
            00000004: 00 00 00 00 |....|
            00000008: 00 00 00 00 |....|
            0000000C: 00 00 00 00 |....|
        "},
        options: default_test_options()
            .autoskip(false)
            .ungrouped(4, crate::Spacing::Normal)
    },

    ungrouped_wide_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00  00  00  00  |....|
            00000004: 00  00  00  00  |....|
            00000008: 00  00  00  00  |....|
            0000000C: 00  00  00  00  |....|
        "},
        options: default_test_options()
            .autoskip(false)
            .ungrouped(4, crate::Spacing::Wide)
    },

    ungrouped_ultrawide_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00    00    00    00    |....|
            00000004: 00    00    00    00    |....|
            00000008: 00    00    00    00    |....|
            0000000C: 00    00    00    00    |....|
        "},
        options: default_test_options()
            .autoskip(false)
            .ungrouped(4, crate::Spacing::UltraWide)
    },

    grouped_short2_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 0000 0000 |....|
            00000004: 0000 0000 |....|
            00000008: 0000 0000 |....|
            0000000C: 0000 0000 |....|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(GroupSize::Short, Spacing::None, 2, Spacing::Normal)
    },
    grouped_short4_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0000 0000 |........|
            00000008: 0000 0000 0000 0000 |........|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Short, 
                crate::Spacing::None, 
                4, 
                crate::Spacing::Normal
            )
    },
    grouped_short4_wide_and_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00 00  00 00  00 00  00 00  |........|
            00000008: 00 00  00 00  00 00  00 00  |........|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Short, 
                crate::Spacing::Normal, 
                4, 
                crate::Spacing::Wide
            )
    },

    // get a bit tricky
    grouped_short4_none_and_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00 0000 0000 0000 00 |........|
            00000008: 00 0000 0000 0000 00 |........|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Short, 
                crate::Spacing::Normal, 
                4, 
                crate::Spacing::None
            )
    },
    grouped_int2_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00000000 00000000 |........|
            00000008: 00000000 00000000 |........|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Int, 
                crate::Spacing::None, 
                2, 
                crate::Spacing::Normal
            )
    },
    grouped_int2_wide_and_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 00 00 00 00  00 00 00 00  |........|
            00000008: 00 00 00 00  00 00 00 00  |........|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Int, 
                crate::Spacing::Normal, 
                2, 
                crate::Spacing::Wide
            )
    },
    grouped_int4_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000: 00000000 00000000 00000000 00000000 |................|
            00000010: 00000000 00000000 00000000 00000000 |................|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Int, 
                crate::Spacing::None, 
                4, 
                crate::Spacing::Normal
            )
    },
    grouped_long2_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000: 0000000000000000 0000000000000000 |................|
            00000010: 0000000000000000 0000000000000000 |................|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::Long, 
                crate::Spacing::None, 
                2, 
                crate::Spacing::Normal
            )
    },
    grouped_ulong2_normal_spacing_displays_correctly: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 64),
        ]),
        output: indoc! {"
            00000000: 00000000000000000000000000000000 00000000000000000000000000000000 |................................|
            00000020: 00000000000000000000000000000000 00000000000000000000000000000000 |................................|
        "},
        options: default_test_options()
            .autoskip(false)
            .grouped(
                GroupSize::ULong, 
                crate::Spacing::None, 
                2, 
                crate::Spacing::Normal
            )
    },
}

byte_tests! {
    simple_range_start_oneline: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000:        00 0000 0000 0000 0000 0000 0000 |   .............|
        "},
        options: default_test_options()
            .range(3..)
    },

    simple_range_start: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000:        00 0000 0000 0000 0000 0000 0000 |   .............|
            00000010: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .range(3..)
    },
    
    simple_range_end: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            00000010: 0000 0000 0000 0000 0000 0000 00        |.............   |
        "},
        options: default_test_options()
            .range(..29)
    },

    simple_range_end_oneline: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0000 0000 0000 0000 00        |.............   |
        "},
        options: default_test_options()
            .range(..13)
    },

    simple_range_start_and_end: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000:        00 0000 0000 0000 0000 0000 0000 |   .............|
            00000010: 0000 0000 0000 0000 0000 0000 00        |.............   |
        "},
        options: default_test_options()
            .aligned(true)
            .range(3..29)
    },

    simple_range_start_and_end_oneline: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000:        00 0000 0000 0000 0000 00        |   ..........   |
        "},
        options: default_test_options()
            .aligned(true)
            .range(3..13)
    },

    overrange: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: "",
        options: default_test_options()
            .aligned(true)
            .range(16..)
    },

    partial_overrange: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000010:        00 0000 0000 0000 0000 0000 0000 |   .............|
        "},
        options: default_test_options()
            .aligned(true)
            .range(19..48)
    },

    unaligned_simple: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000003: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            00000013: 0000 0000 0000 0000 0000 0000 00        |.............   |
        "},
        options: default_test_options()
            .aligned(false)
            .range(3..)
    },

    unaligned_simple_oneline: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000003: 0000 0000 0000 0000 0000 0000 00        |.............   |
        "},
        options: default_test_options()
            .aligned(false)
            .range(3..)
    },
}

byte_tests! {
    simple_absolute_offset_and_range: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0000 0000 0000 0000 00        |.............   |
        "},
        options: default_test_options()
            .aligned(false)
            .absolute_offset(0)
            .range(3..)
    },
    simple_relative_offset: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            000001B0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            000001C0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .aligned(false)
            .relative_offset(0x1B0)
    },

    simple_relative_offset_and_start_range: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 48),
        ]),
        output: indoc! {"
            000001C0:        00 0000 0000 0000 0000 0000 0000 |   .............|
            000001D0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .aligned(true)
            .range(19..)
            .relative_offset(0x1B0)
    },

    simple_relative_offset_and_start_end_range_no_autoskip: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 64),
        ]),
        output: indoc! {"
            000001C0:        00 0000 0000 0000 0000 0000 0000 |   .............|
            000001D0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            000001E0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .aligned(true)
            .range(19..)
            .autoskip(false)
            .relative_offset(0x1B0)
    },

    simple_relative_offset_and_start_range_false_elision: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 64),
        ]),
        output: indoc! {"
            000001C0:        00 0000 0000 0000 0000 0000 0000 |   .............|
            000001D0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            000001E0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .aligned(true)
            .range(19..)
            .autoskip(true)
            .relative_offset(0x1B0)
    },

    simple_relative_offset_and_start_range_autoskip: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 80),
        ]),
        output: indoc! {"
            000001C0:        00 0000 0000 0000 0000 0000 0000 |   .............|
            000001D0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            *
            000001F0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .aligned(true)
            .range(19..)
            .autoskip(true)
            .relative_offset(0x1B0)
    },
}