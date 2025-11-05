use std::{collections::HashSet, fs, path::PathBuf, sync::LazyLock};

use jiff::{Zoned, civil};
use regex::Regex;

use crate::error::{Error, cargo_error};

mod error;

fn list_days(root_dir: &str) -> Result<impl Iterator<Item = String>, Error> {
    let mut src_dir = PathBuf::from(root_dir);
    src_dir.push("src");

    static DAY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^day[0-2][0-9]$").unwrap());

    Ok(src_dir
        .read_dir()
        .map_err(|e| Error::IO(src_dir.to_string_lossy().to_string(), e))?
        .flatten()
        .flat_map(|e| e.path().file_stem().map(|x| x.to_os_string()))
        .flat_map(|name| name.into_string())
        .filter(|name| DAY_REGEX.is_match(name)))
}

fn fetch_input(today: &Zoned, session_cookie: &str, year: i16, day: i8) -> Result<String, Error> {
    const AOC_URL: &str = "https://adventofcode.com";
    const AOC_USER_AGENT: &str =
        "https://github.com/olilag/aoc-input-build by oliver.oli.lago@gmail.com";

    const AOC_RELEASE_MONTH: i8 = 12;
    const AOC_RELEASE_HOUR: i8 = 0;
    const AOC_RELEASE_TZ: &str = "America/New_York";

    let puzzle_release = civil::datetime(year, AOC_RELEASE_MONTH, day, AOC_RELEASE_HOUR, 0, 0, 0)
        .in_tz(AOC_RELEASE_TZ)
        .expect("Failed to create puzzle release datetime");

    if today < puzzle_release {
        return Err(Error::Date(day, Box::new(puzzle_release)));
    }

    let url = format!("{AOC_URL}/{year}/day/{day}/input");

    let mut resp = ureq::get(&url)
        .header("User-Agent", AOC_USER_AGENT)
        .header("Cookie", session_cookie)
        .call()
        .map_err(|e| Error::Request(url.clone(), e))?
        .into_body();
    resp.read_to_string().map_err(|e| Error::Request(url, e))
}

fn validate_year(today: &Zoned, year: i16) -> bool {
    // NOTE: this assumes that AoC will be available each year
    if !(2015..=today.year()).contains(&year) {
        println!(
            "cargo::error=AoC for provided year '{year}' does not exist. AoC exists for years 2015 to {}.",
            today.year()
        );
        false
    } else {
        true
    }
}

fn validate_day(year: i16, day: i8) -> bool {
    match year {
        // starting from 2025 there will only be 12 days - https://adventofcode.com/2025/about#faq_num_days
        2025.. if !(1..=12).contains(&day) => {
            println!(
                "cargo::warning=Detected a day with number '{day}' out of valid range 1-12, skipping",
            );
            false
        }
        _ if !(1..=25).contains(&day) => {
            println!(
                "cargo::warning=Detected a day with number '{day}' out of valid range 1-25, skipping",
            );
            false
        }
        _ => true,
    }
}

pub fn download_inputs(root_dir: &str, token: &str, year: i16) -> Option<()> {
    const DOWNLOAD_DIR_NAME: &str = "input";

    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=input"); // ensure re-run when a input file was deleted

    let today = Zoned::now();
    if !validate_year(&today, year) {
        return None;
    }

    let res = list_days(root_dir);
    let days = cargo_error(res)?;

    let mut download_dir = PathBuf::from(root_dir);
    download_dir.push(DOWNLOAD_DIR_NAME);

    if !download_dir.exists() {
        let res = fs::create_dir(&download_dir)
            .map_err(|e| Error::IO(download_dir.to_string_lossy().to_string(), e));
        cargo_error(res)?;
    }

    let res = download_dir
        .read_dir()
        .map_err(|e| Error::IO(download_dir.to_string_lossy().to_string(), e));
    let cached: HashSet<String> = cargo_error(res)?
        .flatten()
        .flat_map(|e| e.path().file_stem().map(|x| x.to_os_string()))
        .flat_map(|name| name.into_string())
        .collect();

    let formatted_token = format!("session={token}");

    for day in days {
        if !cached.contains(&day) {
            let n = day[3..]
                .parse::<i8>()
                .expect("Failed to convert day string to number");

            if !validate_day(year, n) {
                continue;
            }

            let res = fetch_input(&today, &formatted_token, year, n);
            if let Some(inp) = cargo_error(res) {
                let file = download_dir.join(format!("{day}.txt"));
                let res = fs::write(&file, inp)
                    .map_err(|e| Error::IO(file.to_string_lossy().to_string(), e));
                let _ = cargo_error(res);
            }
        }
    }

    Some(())
}
