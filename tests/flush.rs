use std::{convert::Infallible, sync::Arc, vec};

use hexd::{options::HexdOptionsBuilder, writer::WriteHexdump, AsHexd};

struct FlushTester(Vec<usize>, usize);

impl Default for FlushTester {
    fn default() -> Self {
        FlushTester(vec![], 0)
    }
}

impl WriteHexdump for FlushTester {
    type Error = Infallible;

    type Output = Vec<usize>;

    fn write_line(&mut self, _: &str) -> Result<(), Self::Error> {
        self.1 += 1;
        Ok(())
    }

    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap().0
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.push(self.1);
        self.1 = 0;
        Ok(())
    }
}

#[test]
fn test_flush_after_n_lines() {
    // Given
    let v = vec![0u8; 87];

    // When
    let flushes = v.hexd()
        .autoskip(false)
        .ungrouped(8, hexd::options::Spacing::None)
        .flush(hexd::options::FlushMode::AfterNLines(4))
        .dump_to::<FlushTester>();

    // Then
    assert_eq!(vec![4usize, 4usize, 3usize], flushes);
}

#[test]
fn test_flush_at_eof() {
    // Given
    let v = vec![0u8; 71];

    // When
    let flushes = v.hexd()
        .autoskip(false)
        .ungrouped(8, hexd::options::Spacing::None)
        .flush(hexd::options::FlushMode::End)
        .dump_to::<FlushTester>();

    // Then
    assert_eq!(vec![9usize], flushes);
}