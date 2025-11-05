use std::{collections::HashSet, fs, path::PathBuf, sync::LazyLock};

use regex::Regex;

use crate::error::{Error, cargo_error};

const DOWNLOAD_DIR_NAME: &str = "input";
const AOC_URL: &str = "https://adventofcode.com";
const AOC_USER_AGENT: &str =
    "https://github.com/olilag/aoc-input-build by oliver.oli.lago@gmail.com";

mod error;

fn list_days(root_dir: &str) -> Result<impl Iterator<Item = String>, Error> {
    let mut src_dir = PathBuf::from(root_dir);
    src_dir.push("src");

    static DAY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^day[0-9][0-9]$").unwrap());

    Ok(src_dir
        .read_dir()
        .map_err(|e| Error::IO(src_dir.to_string_lossy().to_string(), e))?
        .flatten()
        .flat_map(|e| e.path().file_stem().map(|x| x.to_os_string()))
        .flat_map(|name| name.into_string())
        .filter(|name| DAY_REGEX.is_match(name)))
}

fn fetch_input(session_cookie: &str, year: u16, day: u8) -> Result<String, Error> {
    let url = format!("{AOC_URL}/{year}/day/{day}/input");

    let mut resp = ureq::get(&url)
        .header("User-Agent", AOC_USER_AGENT)
        .header("Cookie", session_cookie)
        .call()
        .map_err(|e| Error::Request(url.clone(), e))?
        .into_body();
    resp.read_to_string().map_err(|e| Error::Request(url, e))
}

pub fn download_inputs(root_dir: &str, token: &str, year: u16) -> Option<()> {
    println!("cargo::rerun-if-changed=src");
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
            let n = day[3..].parse::<u8>().unwrap();
            let res = fetch_input(&formatted_token, year, n);
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
