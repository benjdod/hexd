use common::{ByteSequence, RenderTestCase};
use hexd::options::{FlushMode, Grouping, HexdOptions, HexdOptionsBuilder, HexdRange, IndexOffset};
use indoc::indoc;

mod common;

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

    simple_relative_offset_and_start_range_false_autoskip: RenderTestCase {
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