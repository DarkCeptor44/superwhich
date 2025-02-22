#![forbid(unsafe_code)]

/**
 * superwhich: smart which alternative
 * Copyright (C) 2024 DarkCeptor44
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
use clap::Parser;
use colored::*;
use is_executable::IsExecutable;
use jaro_winkler::jaro_winkler;
use rayon::prelude::*;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{PathBuf, MAIN_SEPARATOR},
    process::exit,
};

#[derive(Parser)]
#[command(author,version,about,long_about=None)]
struct App {
    #[arg(help = "The search pattern")]
    pattern: String,

    #[arg(
        short,
        long,
        help = "Color of the highlighted text (off for no color)",
        default_value = "blue"
    )]
    color: String,

    #[arg(
        short = 'j',
        long,
        help = "Jaro-Winkler distance threshold (0.0 to 1.0)",
        default_value_t = 0.8
    )]
    threshold: f64,

    #[arg(
        short = 't',
        long,
        help = "Print time elapsed",
        default_value_t = false
    )]
    print_time: bool,
}

#[derive(PartialEq, Eq, Hash)]
struct PathInfo {
    path_str: String,
    stem: String,
    name: String,
}

fn main() {
    let args = App::parse();

    if args.pattern.is_empty() {
        println!("Search pattern cannot be empty");
        exit(1);
    }

    let color = Color::from(args.color);
    let now = std::time::Instant::now();
    let paths: Vec<PathBuf> = env::split_paths(&env::var_os("PATH").unwrap_or_else(|| {
        println!("PATH is not defined in the environment.");
        exit(1);
    }))
    .collect();

    super_which(paths, args.pattern.to_lowercase(), args.threshold, color);
    let elapsed = now.elapsed();

    if args.print_time {
        println!(
            "\nElapsed: {}",
            format!("{:.3?}", elapsed).color(color).bold()
        );
    }
}

fn super_which(paths: Vec<PathBuf>, pattern: String, threshold: f64, color: colored::Color) {
    let found: BTreeSet<PathInfo> = paths
        .par_iter()
        .fold(BTreeSet::new, |mut acc, path| {
            if !path.exists() {
                return acc;
            }

            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();

                if !entry.path().is_executable() {
                    continue;
                }

                let name = entry.file_name().to_string_lossy().to_string();
                let name_lower = name.to_lowercase();
                if name_lower.contains(&pattern) || jaro_winkler(&name_lower, &pattern) >= threshold
                {
                    acc.insert(PathInfo {
                        path_str: entry.path().to_string_lossy().to_string(),
                        stem: entry.path().parent().unwrap().to_string_lossy().to_string(),
                        name,
                    });
                }
            }
            acc
        })
        .reduce(BTreeSet::new, |mut a, b| {
            a.extend(b);
            a
        });

    for exe in found.iter() {
        println!("{}", highlight_text(exe, &pattern, color));
    }
}

fn highlight_text(info: &PathInfo, pattern: &str, color: colored::Color) -> String {
    let index = info.name.to_lowercase().find(pattern).unwrap_or(usize::MAX);

    if index == usize::MAX {
        return info.path_str.to_string();
    }

    format!(
        "{}{}{}{}{}",
        info.stem,
        MAIN_SEPARATOR,
        info.name[..index].normal(),
        info.name[index..index + pattern.len()].bold().color(color),
        info.name[index + pattern.len()..].normal()
    )
}

impl Ord for PathInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path_str
            .cmp(&other.path_str)
            .then_with(|| self.stem.cmp(&other.stem))
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl PartialOrd for PathInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
