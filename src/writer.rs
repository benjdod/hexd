use std::{convert::Infallible, env::set_current_dir, fmt::Debug, io::BufWriter};

pub trait WriteHexdump: Sized {
    type Error: Debug;
    type Output;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error>;

    fn line_end(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

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
        let cap_len  = self.last().map(|s| s.len()).unwrap_or(0);
        self.push(String::with_capacity(cap_len));
        Ok(())
    }

    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}
