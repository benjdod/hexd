use std::vec;
use indoc::indoc;

use hexd::{AsHexdGrouped, IntoHexdGrouped};

mod common;
use common::IntRenderTestCase;

macro_rules! ints_tests {
    ($($as_name:ident, $into_name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $as_name() {
            // Given
            let IntRenderTestCase { input, output, endianness } = $value;

            // When
            let dump = input.as_hexd(endianness).dump_to::<String>();

            // Then
            similar_asserts::assert_eq!(
                output,
                &dump,
            );
        }

        #[test]
        fn $into_name() {
            // Given
            let IntRenderTestCase { input, output, endianness } = $value;

            // When
            let dump = input.into_iter().map(|i| i).into_hexd(endianness).dump_to::<String>();

            // Then
            similar_asserts::assert_eq!(
                output,
                &dump,
            );
        }
    )*
    };
}

ints_tests! {
    positive_i16_be_as, positive_i16_be_into: IntRenderTestCase {
        input: vec![0x72f0i16; 32],
        output: indoc! {"
            00000000: 72F0 72F0 72F0 72F0 72F0 72F0 72F0 72F0 |r.r.r.r.r.r.r.r.|
            *
            00000030: 72F0 72F0 72F0 72F0 72F0 72F0 72F0 72F0 |r.r.r.r.r.r.r.r.|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },
    negative_i16_be_as, negative_i16_be_into: IntRenderTestCase {
        input: vec![-0x79c2i16; 32],
        output: indoc! {"
            00000000: 863E 863E 863E 863E 863E 863E 863E 863E |.>.>.>.>.>.>.>.>|
            *
            00000030: 863E 863E 863E 863E 863E 863E 863E 863E |.>.>.>.>.>.>.>.>|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },

    u16_be_as, u16_be_into: IntRenderTestCase {
        input: vec![0xd2f0u16; 32],
        output: indoc! {"
            00000000: D2F0 D2F0 D2F0 D2F0 D2F0 D2F0 D2F0 D2F0 |................|
            *
            00000030: D2F0 D2F0 D2F0 D2F0 D2F0 D2F0 D2F0 D2F0 |................|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },

    positive_i32_be_as, positive_i32_be_into: IntRenderTestCase {
        input: vec![0x72f072f0i32; 32],
        output: indoc! {"
            00000000: 72F072F0 72F072F0 72F072F0 72F072F0 |r.r.r.r.r.r.r.r.|
            *
            00000070: 72F072F0 72F072F0 72F072F0 72F072F0 |r.r.r.r.r.r.r.r.|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },
    negative_i32_be_as, negative_i32_be_into: IntRenderTestCase {
        input: vec![-0x79c279c2i32; 32],
        output: indoc! {"
            00000000: 863D863E 863D863E 863D863E 863D863E |.=.>.=.>.=.>.=.>|
            *
            00000070: 863D863E 863D863E 863D863E 863D863E |.=.>.=.>.=.>.=.>|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },

    u32_be_as, u32_be_into: IntRenderTestCase {
        input: vec![0xd2f0d2f0u32; 32],
        output: indoc! {"
            00000000: D2F0D2F0 D2F0D2F0 D2F0D2F0 D2F0D2F0 |................|
            *
            00000070: D2F0D2F0 D2F0D2F0 D2F0D2F0 D2F0D2F0 |................|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },


    positive_i64_be_as, positive_i64_be_into: IntRenderTestCase {
        input: vec![0x72f072f072f072f0i64; 32],
        output: indoc! {"
            00000000: 72F072F072F072F0 72F072F072F072F0 |r.r.r.r.r.r.r.r.|
            *
            000000F0: 72F072F072F072F0 72F072F072F072F0 |r.r.r.r.r.r.r.r.|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },
    negative_i64_be_as, negative_i64_be_into: IntRenderTestCase {
        input: vec![-0x79c279c279c279c2i64; 32],
        output: indoc! {"
            00000000: 863D863D863D863E 863D863D863D863E |.=.=.=.>.=.=.=.>|
            *
            000000F0: 863D863D863D863E 863D863D863D863E |.=.=.=.>.=.=.=.>|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },

    u64_be_as, u64_be_into: IntRenderTestCase {
        input: vec![0xd2f0d2f0d2f0d2f0u64; 32],
        output: indoc! {"
            00000000: D2F0D2F0D2F0D2F0 D2F0D2F0D2F0D2F0 |................|
            *
            000000F0: D2F0D2F0D2F0D2F0 D2F0D2F0D2F0D2F0 |................|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },


    positive_i128_be_as, positive_i128_be_into: IntRenderTestCase {
        input: vec![0x72f072f072f072f072f072f072f072f0i128; 16],
        output: indoc! {"
            00000000: 72 F0 72 F0 72 F0 72 F0 72 F0 72 F0 72 F0 72 F0 |r.r.r.r.r.r.r.r.|
            *
            000000F0: 72 F0 72 F0 72 F0 72 F0 72 F0 72 F0 72 F0 72 F0 |r.r.r.r.r.r.r.r.|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },
    negative_i128_be_as, negative_i128_be_into: IntRenderTestCase {
        input: vec![-0x79c279c279c279c279c279c279c279c2i128; 16],
        output: indoc! {"
            00000000: 86 3D 86 3D 86 3D 86 3D 86 3D 86 3D 86 3D 86 3E |.=.=.=.=.=.=.=.>|
            *
            000000F0: 86 3D 86 3D 86 3D 86 3D 86 3D 86 3D 86 3D 86 3E |.=.=.=.=.=.=.=.>|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },

    u128_be_as, u128_be_into: IntRenderTestCase {
        input: vec![0xd2f0d2f0d2f0d2f0d2f0d2f0d2f0d2f0u128; 16],
        output: indoc! {"
            00000000: D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 |................|
            *
            000000F0: D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 D2 F0 |................|
        "},
        endianness: hexd::options::Endianness::BigEndian,
    },
}
