//! # superwhich
//!
//! Smart `which` alternative
//!
//! ## Installation
//!
//! ```bash
//! cargo install superwhich
//!
//! # or
//! cargo binstall superwhich
//! ```
//!
//! ## Usage
//!
//! ```bash
//! $ swhich -h
//! Cross-platform smart which alternative
//!
//! Usage: swhich [OPTIONS] <PATTERN>
//!
//! Arguments:
//!   <PATTERN>  The search pattern
//!
//! Options:
//!   -c, --color <COLOR>          Color of the highlighted text (off or set `NO_COLOR` env var to disable) [default: blue]
//!   -T, --threshold <THRESHOLD>  String similarity threshold (0.0 to 1.0) [default: 0.8]
//!   -t, --print-time             Print time elapsed
//!   -h, --help                   Print help
//!   -V, --version                Print version
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

/**
 * superwhich: smart which alternative
 * Copyright (C) 2024 `DarkCeptor44`
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use anyhow::{anyhow, Result};
use clap::Parser;
use colored::{Color, Colorize};
use std::{
    env::{current_exe, split_paths, var_os},
    path::PathBuf,
    process::exit,
    time::Instant,
};
use superwhich::{find_executables, highlight_text};

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
struct App {
    #[arg(help = "The search pattern")]
    pattern: String,

    #[arg(
        short,
        long,
        help = "Color of the highlighted text (off or set `NO_COLOR` env var to disable)",
        default_value = "blue"
    )]
    color: String,

    #[arg(
        short = 'T',
        long,
        help = "String similarity threshold (0.0 to 1.0)",
        default_value_t = 0.7
    )]
    threshold: f64,

    #[arg(short = 't', long, help = "Print time elapsed", default_value_t)]
    print_time: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!(
            "{} {}",
            format!("{}:", exe_name()).red().bold(),
            e.to_string().red()
        );
        exit(1);
    }
}

fn exe_name() -> String {
    current_exe()
        .ok()
        .and_then(|p| {
            p.file_name().map(|s| {
                s.to_string_lossy()
                    .strip_suffix(".exe")
                    .unwrap_or(&s.to_string_lossy())
                    .to_string()
            })
        })
        .unwrap_or("superwhich".into())
}

fn run() -> Result<()> {
    let args = App::parse();

    if args.pattern.trim().is_empty() {
        return Err(anyhow!("search pattern cannot be empty"));
    }

    let color = Color::from(args.color);
    let now = Instant::now();
    let paths: Vec<PathBuf> =
        split_paths(&var_os("PATH").ok_or(anyhow!("PATH is not set"))?).collect();
    let found = find_executables(
        &paths,
        &args.pattern,
        args.threshold,
        is_executable::IsExecutable::is_executable,
    );

    for exe in &found {
        println!("{}", highlight_text(exe, &args.pattern, color, None));
    }

    let elapsed = now.elapsed();
    if args.print_time {
        println!(
            "\nElapsed: {}",
            format!("{elapsed:.3?}").color(color).bold()
        );
    }

    Ok(())
}
