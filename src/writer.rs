use std::{convert::Infallible, fmt::Debug, fs::File, io::{Stderr, Stdout, Write}};

pub trait WriteHexdump: Sized {
    type Error: Debug;
    type Output;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output;
}

#[doc(hidden)]
pub struct IOWriter<W: std::io::Write>(pub W);

impl<W: std::io::Write> WriteHexdump for IOWriter<W> {
    type Error = std::io::Error;
    type Output = Result<(), std::io::Error>;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.map(|_| ())
    }
}

impl WriteHexdump for Stdout {
    type Error = std::io::Error;
    type Output = ();
    fn write_line(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(self)
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap();
        ()
    }
}

impl WriteHexdump for Stderr {
    type Error = std::io::Error;
    type Output = ();
    fn write_line(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(self)
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap();
        ()
    }
}

impl WriteHexdump for File {
    type Error = std::io::Error;
    type Output = Result<(), std::io::Error>;
    fn write_line(&mut self, s: &str) -> Result<(), std::io::Error> {
        self.write_all(s.as_bytes())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(self)
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.map(|_| ())
    }
}

impl WriteHexdump for String {
    type Error = Infallible;
    type Output = String;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push_str(s);
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}

impl WriteHexdump for Vec<String> {
    type Error = Infallible;
    type Output = Vec<String>;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push(s.to_string());
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}

impl WriteHexdump for Vec<u8> {
    type Error = Infallible;
    type Output = Vec<u8>;
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
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
    fn write_line(&mut self, s: &str) -> Result<(), Self::Error> {
        self.push(s.as_bytes().to_vec());
        Ok(())
    }
    fn consume(r: Result<Self, Self::Error>) -> Self::Output {
        r.unwrap()
    }
}