use common::{RenderTestCase, ByteSequence};
use hexd::options::{FlushMode, GroupSize, Grouping, HexdOptions, HexdOptionsBuilder, HexdRange, IndexOffset, Spacing};
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
            .ungrouped(4, Spacing::None)
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
            .ungrouped(4, Spacing::Normal)
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
            .ungrouped(4, Spacing::Wide)
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
            .ungrouped(4, Spacing::UltraWide)
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
                Spacing::None, 
                4, 
                Spacing::Normal
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
                Spacing::Normal, 
                4, 
                Spacing::Wide
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
                Spacing::Normal, 
                4, 
                Spacing::None
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
                Spacing::None, 
                2, 
                Spacing::Normal
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
                Spacing::Normal, 
                2, 
                Spacing::Wide
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
                Spacing::None, 
                4, 
                Spacing::Normal
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
                Spacing::None, 
                2, 
                Spacing::Normal
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
                Spacing::None, 
                2, 
                Spacing::Normal
            )
    },
}