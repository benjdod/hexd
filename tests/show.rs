use common::{ByteSequence, RenderTestCase};
use hxd::options::{Grouping, HexdOptions, HexdOptionsBuilder, HexdRange, IndexOffset};
use indoc::indoc;

mod common;

fn default_test_options() -> HexdOptions {
    let default_options = HexdOptions {
        base: hxd::options::Base::Hex,
        autoskip: true,
        uppercase: true,
        show_index: true,
        show_ascii: true,
        align: true,
        grouping: Grouping::default(),
        print_range: HexdRange {
            skip: 0,
            limit: None,
        },
        index_offset: IndexOffset::Relative(0),
    };
    default_options
}

byte_tests! {
    dont_show_index_or_ascii: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            0000 0000 0000 0000 0000 0000 0000 0000
            0000 0000 0000 0000 0000 0000 0000 0000
        "},
        options: default_test_options()
            .show_ascii(false)
            .show_index(false)
    },

    dont_show_index: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            0000 0000 0000 0000 0000 0000 0000 0000 |................|
            0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .show_ascii(true)
            .show_index(false)
    },

    dont_show_ascii: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0000 0000 0000 0000 0000 0000
            00000010: 0000 0000 0000 0000 0000 0000 0000 0000
        "},
        options: default_test_options()
            .show_ascii(false)
            .show_index(true)
    },

    show_index_and_ascii: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 32),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
            00000010: 0000 0000 0000 0000 0000 0000 0000 0000 |................|
        "},
        options: default_test_options()
            .show_ascii(true)
            .show_index(true)
    },
}