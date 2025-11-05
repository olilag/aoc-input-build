use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    IO(String, io::Error),
    Request(String, ureq::Error),
    Date(i8, Box<jiff::Zoned>),
}

impl Error {
    pub fn fatal(&self) -> bool {
        match self {
            Self::IO(_, _) => true,
            Self::Request(_, _) => true,
            Self::Date(_, _) => false,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(file, error) => write!(f, "IO error: '{error}' when accessing '{file}'"),
            Self::Request(url, error) => {
                write!(f, "HTTP error: '{error}' when fetching '{url}'")
            }
            Self::Date(day, release) => write!(
                f,
                "trying to access day {day} input before it is ready on {release}" // TODO: improve date formatting
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(_, error) => Some(error),
            Self::Request(_, error) => Some(error),
            Self::Date(_, _) => None,
        }
    }
}

pub fn cargo_error<T>(res: Result<T, Error>) -> Option<T> {
    match res {
        Ok(t) => Some(t),
        Err(e) => {
            if e.fatal() {
                println!("cargo::error={}", e);
            } else {
                println!("cargo::warning={}", e);
            }
            None
        }
    }
}
