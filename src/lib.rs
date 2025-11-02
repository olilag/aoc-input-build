use std::{collections::HashSet, fs, path::PathBuf};

use regex::Regex;
use reqwest::{
    blocking::Client,
    header::{COOKIE, HeaderMap},
};

const DOWNLOAD_DIR_NAME: &str = "input";
const AOC_URL: &str = "https://adventofcode.com";

fn list_days(root_dir: &str) -> Vec<String> {
    let mut src_dir = PathBuf::from(root_dir);
    src_dir.push("src");

    let day_regex = Regex::new(r"^day[0-9][0-9]$").unwrap();

    src_dir
        .read_dir()
        .expect("Failed to read src")
        .flatten()
        .flat_map(|e| e.path().file_stem().map(|x| x.to_os_string()))
        .flat_map(|name| name.into_string())
        .filter(|name| day_regex.is_match(name))
        .collect()
}

fn fetch_input(client: &Client, year: u16, day: &str) -> Result<String, reqwest::Error> {
    let n = day[3..].parse::<u8>().unwrap();
    let url = format!("{AOC_URL}/{year}/day/{n}/input");

    let resp = client.get(url).send()?;
    resp.error_for_status_ref()?;
    resp.text()
}

pub fn download_inputs(root_dir: &str, token: &str, year: u16) {
    println!("cargo::rerun-if-changed=src");
    let days = list_days(root_dir);

    let mut download_dir = PathBuf::from(root_dir);
    download_dir.push(DOWNLOAD_DIR_NAME);

    if !download_dir.exists() {
        fs::create_dir(&download_dir).expect("Failed to create dir");
    }

    let cached: HashSet<String> = download_dir
        .read_dir()
        .expect("Failed to read download director")
        .flatten()
        .flat_map(|e| e.path().file_stem().map(|x| x.to_os_string()))
        .flat_map(|name| name.into_string())
        .collect();

    let formatted_token = format!("session={token}");

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, formatted_token.parse().unwrap());
    let client = Client::builder()
        .user_agent("input downloader by oliver.oli.lago@gmail.com")
        .default_headers(headers)
        .build()
        .unwrap();

    for day in days {
        if !cached.contains(&day) {
            let inp = fetch_input(&client, year, &day).expect("Download failed");
            fs::write(download_dir.join(format!("{day}.txt")), inp).expect("Write to file failed");
        }
    }
}
