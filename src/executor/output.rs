use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use crate::parser::Redirect;

pub enum Output {
    Terminal(io::Stdout),
    File(File),
}

impl Output {
    pub fn from_redirect(redirect: Option<Redirect>) -> io::Result<Self> {
        match redirect {
            None => Ok(Self::Terminal(io::stdout())),

            Some(Redirect::Write(path)) => {
                let file = File::create(path)?;
                Ok(Self::File(file))
            }

            Some(Redirect::Append(path)) => {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;

                Ok(Self::File(file))
            }
        }
    }
}

impl Write for Output {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self {
            Self::Terminal(stdout) => stdout.write(buffer),
            Self::File(file) => file.write(buffer),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Terminal(stdout) => stdout.flush(),
            Self::File(file) => file.flush(),
        }
    }
}
