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
extern crate clap;
extern crate colored;
extern crate jaro_winkler;
extern crate rayon;

use clap::Parser;
use colored::*;
use jaro_winkler::jaro_winkler;
use rayon::prelude::*;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf, MAIN_SEPARATOR},
    process::exit,
    sync::{Arc, Mutex},
};

#[cfg(windows)]
const EXTENSIONS: [&str; 8] = ["exe", "sh", "bat", "cmd", "com", "ps1", "vbs", "py"];

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

    super_which(paths, args.pattern.to_lowercase(), color);
    let elapsed = now.elapsed();

    if args.print_time {
        println!(
            "\nElapsed: {}",
            format!("{:.2?}", elapsed).color(color).bold()
        );
    }
}

fn super_which(paths: Vec<PathBuf>, pattern: String, color: colored::Color) {
    let found: Arc<Mutex<BTreeSet<PathInfo>>> = Arc::new(Mutex::new(BTreeSet::new()));

    paths.par_iter().for_each(|path| {
        if !path.exists() {
            return;
        }

        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();

            if !is_executable(&entry.path()) {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();
            if name_lower.contains(&pattern) || jaro_winkler(&name_lower, &pattern) >= 0.8 {
                found.lock().unwrap().insert(PathInfo {
                    path_str: entry.path().to_string_lossy().to_string(),
                    stem: entry.path().parent().unwrap().to_string_lossy().to_string(),
                    name,
                });
            }
        }
    });

    let found = found.lock().unwrap();
    for exe in found.iter() {
        println!("{}", highlight_text(exe, &pattern, color));
    }
}

fn is_executable(path: &Path) -> bool {
    if !path.exists() || !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        let p = path.metadata().unwrap().permissions();
        p.readonly() || p.execute()
    }

    #[cfg(not(unix))]
    {
        EXTENSIONS.contains(
            &path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase()
                .as_str(),
        )
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
