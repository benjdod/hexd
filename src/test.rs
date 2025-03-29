use std::{option, process::id};

use crate::hexdump::{ByteSliceReader, Grouping, HexdumpFmtWriter, HexdumpLineWriter, HexdumpOptions, HexdumpRange};

#[test]
fn go() -> Result<(), String> {
    for test in get_test_cases() {
        let RenderTestCase { input, output, options } = test;
        let writer = HexdumpFmtWriter(String::new());
        let mut hww = HexdumpLineWriter::new(
            ByteSliceReader::new(&input), 
            writer, 
            options);

        hww.do_hexdump();
        let x = hww.consume().0;
        for (idx, actual) in x.lines().enumerate() {
            let expected = output.get(idx)
                .map(|s| *s)
                .ok_or_else(|| {
                    format!("expected no more lines, but saw lines remaining.\n{actual}")
                })?;
            assert_eq!(expected, actual)
        }
    }
    Ok(())
}

struct RenderTestCase {
    input: Vec<u8>,
    output: Vec<&'static str>,
    options: HexdumpOptions
}

impl RenderTestCase {
    fn new(input: Vec<u8>, output: Vec<&'static str>, options: HexdumpOptions) -> Self {
        Self { input, output, options }
    }
}

fn get_test_cases() -> Vec<RenderTestCase> {
    let default_options = HexdumpOptions {
        grouping: crate::hexdump::Grouping::Grouped { group_size: crate::hexdump::GroupSize::Int, num_groups: 4, byte_spacing: crate::hexdump::Spacing::None, group_spacing: crate::hexdump::Spacing::Normal },
        ..Default::default()
    };
    vec![
        RenderTestCase {
            input: vec![0u8; 16],
            output: vec![
                "00000000: 00000000 00000000 00000000 00000000 |................|"
            ],
            options: default_options.clone()
        },
        RenderTestCase {
            input: vec![0u8; 16],
            output: vec![
                "00000000:       00 00000000 00000000 00000000 |   .............|"
            ],
            options: HexdumpOptions {
                print_range: HexdumpRange::new(3..),
                ..default_options.clone()
            }
        },
        RenderTestCase {
            input: vec![0u8; 32],
            output: vec![
                "00000000: 0000 0000 0000 0000 0000 0000 0000 0000 |................|",
                "00000010: 0000 0000 0000 0000 0000 0000 0000 0000 |................|"
            ],
            options: HexdumpOptions {
                grouping: crate::hexdump::Grouping::Grouped {
                    group_size: crate::hexdump::GroupSize::Short,
                    num_groups: 8,
                    group_spacing: crate::hexdump::Spacing::Normal,
                    byte_spacing: crate::hexdump::Spacing::None
                },
                ..default_options.clone()
            }
        },
        RenderTestCase {
            input: vec![0u8; 8],
            output: vec![
                "00000000: 0000 0000 0000 0000 |........|"
            ],
            options: HexdumpOptions {
                grouping: crate::hexdump::Grouping::Grouped {
                    group_size: crate::hexdump::GroupSize::Short,
                    num_groups: 4,
                    group_spacing: crate::hexdump::Spacing::Normal,
                    byte_spacing: crate::hexdump::Spacing::None
                },
                ..default_options.clone()
            }
        }
    ]
}