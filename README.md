# aoc-input-build

## About

`aoc-input-build` is a Rust library to be used as build dependency from [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
It downloads input files for [Advent of Code](https://adventofcode.com/).

## Installing

Add this library as build dependency.

```bash
cargo add --build aoc-input-build
```

## Usage

```rust
// in build.rs
use aoc_input_build::download_inputs;

fn main() {
    let root_dir = env!("CARGO_MANIFEST_DIR"); // root of the project, should always be set to CARGO_MANIFEST_DIR env var
    let token = env!("AOC_TOKEN"); // session cookie from https://adventofcode.com/
    let year = 2025; // which year of Advent of Code to use

    download_inputs(root_dir, token, year);
}
```

To download input file for a day, a `dayXX.rs` file must be present in `src/` (where `XX` stands for the day's number e.g. `day01.rs`, single digit numbers with a leading zero). The input files will be downloaded to `input/` subdirectory and called `dayXX.txt`.

Downloading files with build script ensures that they will be available when the project is built. That means you can use `include_str!` macro to load input. Or you can use an aoc helper that doesn't support downloading input files and you don't have to manually get them.

Here is an example of a correct project structure:

```text
.
├── build.rs
├── Cargo.lock
├── Cargo.toml
├── input - will download input files here
│   ├── day01.txt
│   └── day02.txt
└── src
    ├── day01.rs
    ├── day02.rs
    └── main.rs
```

The build script will re-run when any file inside `src/` or `input/` is changed. But once the input files are downloaded, subsequent execution of the build script won't download those days again. Change of year also won't trigger input file re-download.

To download an input file again, you need to delete it and rebuild the project.

## Contributing

Feedback and pull requests are welcome.

## AoC automation

This script does follow the automation guidelines on the [/r/adventofcode community wiki](https://www.reddit.com/r/adventofcode/wiki/faqs/automation).

Once inputs are downloaded, they are cached locally (see `download_inputs()`).

If you suspect your input is corrupted, you can delete the file and rebuild project with `cargo build` to re-run build script.

The `User-Agent` header in `fetch_input()` is set to me.
