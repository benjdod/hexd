use std::{convert::Infallible, fmt::Debug, io::BufWriter};

pub trait WriteHexdump: Sized {
    type Error: Debug;

    /// Type to return when the writer is consumed.
    ///
    /// Although it may seem complex, this can be useful
    /// when implementing this trait
    /// over foreign types:
    ///
    /// ```no_run
    /// use hxd::{AsHexd, writer::WriteHexdump};
    /// use std::convert::Infallible;
    ///
    /// #[derive(Default)]
    /// struct MyByteWriter(Vec<u8>);
    ///
    /// impl WriteHexdump for MyByteWriter {
    ///     type Error = Infallible;
    ///     type Output = Vec<u8>;
    ///
    ///     fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
    ///         self.0.extend_from_slice(s.as_bytes());
    ///         Ok(())
    ///     }
    ///
    ///     fn consume(r: Result<Self, Self::Error>) -> Self::Output {
    ///        r.unwrap().0
    ///     }
    /// }
    ///
    /// let v: Vec<u8> = b"greetings!".hexd().dump_to::<MyByteWriter>();
    /// ```
    type Output;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error>;

    /// This method is called when a line ends, and is provided
    /// to allow the writer to do any necessary processing or flushing.
    ///
    /// > Note: a newline character (`\n`) will be written after each
    /// > line, so that is not necessary to do in this method.
    fn line_end(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Consume the writer or any error encountered during
    /// writing and return the [`Output`](Self::Output) type.
    fn consume(r: Result<Self, Self::Error>) -> Self::Output;
}

#[doc(hidden)]
pub struct IOWriter<W: std::io::Write>(pub W);

impl<W: std::io::Write> IOWriter<W> {
    pub fn new(w: W) -> IOWriter<BufWriter<W>> {
        IOWriter(BufWriter::new(w))
    }
    pub fn new_unbuffered(w: W) -> Self {
        Self(w)
    }
}

impl<W: std::io::Write> WriteHexdump for IOWriter<W> {
    type Error = std::io::Error;
    type Output = Result<(), std::io::Error>;
    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_all(s.as_bytes())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.and_then(|mut s| s.0.flush())
    }
}

impl WriteHexdump for String {
    type Error = Infallible;
    type Output = String;
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push_str(s);
        Ok(())
    }
}

impl WriteHexdump for Vec<String> {
    type Error = Infallible;
    type Output = Vec<String>;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let last = if self.len() > 0 {
            self.last_mut().unwrap()
        } else {
            self.push(String::new());
            self.last_mut().unwrap()
        };

        last.push_str(s);
        Ok(())
    }

    fn line_end(&mut self) -> Result<(), Self::Error> {
        let cap_len = self.last().map(|s| s.len()).unwrap_or(0);
        self.push(String::with_capacity(cap_len));
        Ok(())
    }

    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}

impl WriteHexdump for Vec<u8> {
    type Error = Infallible;
    type Output = Vec<u8>;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}

impl WriteHexdump for Vec<Vec<u8>> {
    type Error = Infallible;
    type Output = Vec<Vec<u8>>;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let last = if self.len() > 0 {
            self.last_mut().unwrap()
        } else {
            self.push(Vec::new());
            self.last_mut().unwrap()
        };

        last.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn line_end(&mut self) -> Result<(), Self::Error> {
        let cap_len = self.last().map(|s| s.len()).unwrap_or(0);
        self.push(Vec::with_capacity(cap_len));
        Ok(())
    }

    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}
