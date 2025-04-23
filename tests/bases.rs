use common::{ByteSequence, RenderTestCase};
use hxd::options::{Grouping, HexdOptions, HexdOptionsBuilder, HexdRange, IndexOffset};
use indoc::indoc;
mod common;

fn default_test_options() -> HexdOptions {
    let default_options = HexdOptions {
        base: hxd::options::Base::Hex,
        autoskip: true,
        uppercase: true,
        print_ascii: true,
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
    hexadecimal: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4),
            (0x0fu8, 4),
            (0x7au8, 4),
            (0x20u8, 4),
        ]),
        output: indoc! {"
            00000000: 0000 0000 0F0F 0F0F 7A7A 7A7A 2020 2020 |........zzzz    |
        "},
        options: default_test_options()
            .hexadecimal()
    },

    decimal: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4),
            (0x0fu8, 4),
            (0x7au8, 4),
            (0x20u8, 4),
        ]),
        output: indoc! {"
            00000000:  00  00  00  00  15  15  15  15 |........|
            00000008: 122 122 122 122  32  32  32  32 |zzzz    |
        "},
        options: default_test_options()
            .decimal()
    },

    octal: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4),
            (0x0fu8, 4),
            (0x7au8, 4),
            (0x20u8, 4),
        ]),
        output: indoc! {"
            00000000: 000 000 000 000 017 017 017 017 |........|
            00000008: 172 172 172 172 040 040 040 040 |zzzz    |
        "},
        options: default_test_options()
            .octal()
    },

    binary: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4),
            (0x0fu8, 4),
            (0x7au8, 4),
            (0x20u8, 4),
        ]),
        output: indoc! {"
            00000000: 00000000 00000000 00000000 00000000 |....|
            00000004: 00001111 00001111 00001111 00001111 |....|
            00000008: 01111010 01111010 01111010 01111010 |zzzz|
            0000000C: 00100000 00100000 00100000 00100000 |    |
        "},
        options: default_test_options()
            .binary()
    },


    decimal_underscore_lzc: RenderTestCase {
        input: ByteSequence::new(vec![
            (0u8, 4),
            (0x0fu8, 4),
            (0x7au8, 4),
            (0x20u8, 4),
        ]),
        output: indoc! {"
            00000000: _00 _00 _00 _00 _15 _15 _15 _15 |........|
            00000008: 122 122 122 122 _32 _32 _32 _32 |zzzz    |
        "},
        options: default_test_options()
            .decimal()
            .base(hxd::options::Base::Decimal(hxd::options::LeadingZeroChar::Underscore))
    },
}
