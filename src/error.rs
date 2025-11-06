use std::{fmt::Display, io, sync::LazyLock};

use icu::{
    calendar::Iso,
    datetime::{
        DateTimeFormatter,
        fieldsets::{self, YMDT},
    },
    locale,
    time::ZonedDateTime,
};
use jiff::tz::TimeZone;
use jiff_icu::ConvertFrom as _;
use locale_config::Locale;

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

fn localized_formatter() -> DateTimeFormatter<YMDT> {
    static LOCALE: LazyLock<locale::Locale> = LazyLock::new(|| {
        Locale::current()
            .tags_for("time")
            .next()
            .map_or_else(|| "en-US".to_string(), |l| l.as_ref().to_owned())
            .parse()
            .expect("Failed to parse locale")
    });

    DateTimeFormatter::try_new((&*LOCALE).into(), fieldsets::YMDT::short())
        .expect("Failed to construct date time formatter")
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(file, error) => write!(f, "IO error: '{error}' when accessing '{file}'"),
            Self::Request(url, error) => {
                write!(f, "HTTP error: '{error}' when fetching '{url}'")
            }
            Self::Date(day, release) => {
                let local_tz = TimeZone::system();
                let icu_zdt =
                    ZonedDateTime::<Iso, _>::convert_from(&release.with_time_zone(local_tz));
                let formatter = localized_formatter();

                write!(
                    f,
                    "trying to access day {day} input before it is ready on {}",
                    formatter.format(&icu_zdt)
                )
            }
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
