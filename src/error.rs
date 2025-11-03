use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    IO(String, io::Error),
    Request(String, ureq::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(file, error) => write!(f, "IO error: '{error}' when accessing '{file}'"),
            Self::Request(url, error) => {
                write!(f, "HTTP error: '{error}' when fetching '{url}'")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(_, error) => Some(error),
            Self::Request(_, error) => Some(error),
        }
    }
}

pub fn cargo_error<T>(res: Result<T, Error>) -> Option<T> {
    match res {
        Ok(t) => Some(t),
        Err(e) => {
            println!("cargo::error={}", e);
            None
        }
    }
}
