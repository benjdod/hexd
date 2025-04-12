use common::{RenderTestCase, ByteSequence};
use indoc::indoc;

pub mod common;

use hexd::{options::{GroupSize, Grouping, HexdOptions, HexdOptionsBuilder, HexdRange, IndexOffset, Spacing, FlushMode}, IntoHexd};

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

fn autoskip_test_options() -> HexdOptions {
    default_test_options()
        .grouped(GroupSize::Short, Spacing::None, 2, Spacing::Normal)
}

byte_tests! {
    one_line_is_not_autoskipped: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
        ),
        options: autoskip_test_options()
    },
    two_equal_lines_are_not_autoskipped: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 8)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
            "00000004: 0000 0000 |....|\n",
        ),
        options: autoskip_test_options()
    },
    three_equal_lines_are_autoskipped: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 12)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
            "*\n",
            "00000008: 0000 0000 |....|\n",
        ),
        options: autoskip_test_options()
    },
    four_equal_lines_are_autoskipped: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16)
        ]),
        output: concat!(
            "00000000: 0000 0000 |....|\n",
            "*\n",
            "0000000C: 0000 0000 |....|\n",
        ),
        options: autoskip_test_options()
    },
    partial_ending_line_is_not_included_in_autoskip: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 17)
        ]),
        output: indoc!{"
            00000000: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
            00000010: 00        |.   |
        "},
        options: autoskip_test_options()
    },
    partial_beginning_line_is_not_included_in_autoskip: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 16)
        ]),
        output: indoc!{"
            00000000:        00 |   .|
            00000004: 0000 0000 |....|
            *
            0000000C: 0000 0000 |....|
        "},
        options: autoskip_test_options()
            .range(3..)
    },
    two_aligned_autoskips_are_handled: RenderTestCase {
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
        options: autoskip_test_options()
    },
    one_aligned_autoskip_and_one_false_autoskip_are_handled: RenderTestCase {
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
        options: autoskip_test_options()
    },
    two_unaligned_autoskips_are_handled: RenderTestCase {
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
        options: autoskip_test_options()
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