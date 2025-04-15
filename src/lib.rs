//! # superwhich
//!
//! Smart `which` alternative
//!
//! ## Installation
//!
//! ```bash
//! cargo add superwhich
//! ```
//!
//! Or add this to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! superwhich = "^2"
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use is_executable::IsExecutable;
//! use std::{
//!     collections::BTreeSet,
//!     env::{split_paths, var_os},
//!     path::PathBuf,
//! };
//! use superwhich::find_executables;
//!
//! let paths: Vec<PathBuf> = split_paths(&var_os("PATH").expect("PATH is not set")).collect();
//!
//! // finds executables in the paths provided that match the pattern,
//! // while `is_executable` crate is used in the CLI you can also just use a
//! // closure that returns true or false
//! let found: BTreeSet<PathBuf> = find_executables(&paths, "pattern", 0.7, IsExecutable::is_executable);
//! ```
//!
//! Refer to the README for more details.

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
use colored::{Color, Colorize};
use jaro_winkler::jaro_winkler;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::BTreeSet,
    fs::read_dir,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

/// Find executables within the specified paths that match the pattern
///
/// ## Arguments
///
/// * `paths` - List of paths to search
/// * `pattern` - Search pattern
/// * `threshold` - Jaro-Winkler similarity threshold
/// * `is_executable_check` - Function to check if a file is executable (the [`is_executable`](https://docs.rs/is_executable/latest/is_executable/) crate is used in the CLI)
///
/// ## Returns
///
/// * `BTreeSet<PathBuf>` - Set of found executables
///
/// ## Example
///
/// ```rust,no_run
/// use is_executable::IsExecutable;
/// use std::{
///     collections::BTreeSet,
///     env::{split_paths, var_os},
///     path::PathBuf,
/// };
/// use superwhich::find_executables;
///
/// let paths: Vec<PathBuf> = split_paths(&var_os("PATH").expect("PATH is not set")).collect();
/// let found: BTreeSet<PathBuf> = find_executables(&paths, "pattern", 0.7, IsExecutable::is_executable);
/// ```
#[must_use]
pub fn find_executables<F>(
    paths: &[PathBuf],
    pattern: &str,
    threshold: f64,
    is_executable_check: F,
) -> BTreeSet<PathBuf>
where
    F: Fn(&Path) -> bool + Sync + Send,
{
    let pattern_lower = pattern.to_lowercase();
    let checker = &is_executable_check;

    paths
        .par_iter()
        .filter(|path| path.is_dir())
        .flat_map_iter(move |path| {
            let entries = match read_dir(path) {
                Ok(reader) => reader,
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!(
                            "{} could not read directory `{}`: {e}",
                            "Warning:".bold(),
                            path.display()
                        )
                        .yellow()
                    );
                    return Vec::new().into_iter();
                }
            };

            let results_for_this_path = entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| checker(p))
                .filter_map(|p| {
                    p.file_name()
                        .and_then(|name_os| name_os.to_str())
                        .and_then(|name_str| {
                            let name_lower = name_str.to_lowercase();
                            if name_lower.contains(&pattern_lower)
                                || jaro_winkler(&name_lower, &pattern_lower) >= threshold
                            {
                                Some(p.clone())
                            } else {
                                None
                            }
                        })
                })
                .collect::<Vec<_>>();

            results_for_this_path.into_iter()
        })
        .collect()
}

/// Highlight the specified text with the specified color
///
/// ## Arguments
///
/// * `path` - The path to highlight
/// * `pattern` - The search pattern
/// * `color` - The color to use
/// * `separator` - The separator to use (`None` uses the default for the current platform)
///
/// ## Returns
///
/// * `String` - The highlighted text
///
/// ## Example
///
/// ```rust,no_run
/// use colored::Color;
/// use std::path::Path;
/// use superwhich::highlight_text;
///
/// let text = highlight_text(Path::new("/usr/bin/lsx"), "LS", Color::Blue, None);
/// ```
#[must_use]
pub fn highlight_text(path: &Path, pattern: &str, color: Color, separator: Option<char>) -> String {
    let sep = separator.unwrap_or(MAIN_SEPARATOR);
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return path.display().to_string();
    };
    let stem = match path.parent() {
        Some(p) => p.display().to_string(),
        None => String::new(),
    };

    let pattern_lower = pattern.to_lowercase();
    let name_lower = name.to_lowercase();

    if let Some(index) = name_lower.find(&pattern_lower) {
        format!(
            "{}{}{}{}{}",
            stem,
            if stem.is_empty() || stem.ends_with(sep) {
                String::new()
            } else {
                sep.to_string()
            },
            &name[..index],
            &name[index..index + pattern.len()].color(color).bold(),
            &name[index + pattern.len()..]
        )
    } else {
        path.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::{
        fs::{create_dir_all, File},
        io::Write,
    };
    use tempfile::tempdir;

    #[test]
    fn test_find_executables() -> Result<()> {
        let temp_dir = tempdir()?;
        let example_bins = ["ll", "la", "lsx", "lsa", "as", "grep", "dd", "rm", "rmz"];
        let expected: BTreeSet<PathBuf> = ["lsx", "lsa"]
            .iter()
            .map(|x| temp_dir.path().join(format!("{x}.exe")))
            .collect();

        create_dir_all(temp_dir.path())?;

        for bin in example_bins {
            let path = temp_dir.path().join(format!("{bin}.exe"));
            let mut file = File::create(&path)?;

            file.write_all(b"test")?;
            drop(file);
        }

        let found = find_executables(&[temp_dir.path().to_path_buf()], "LS", 0.7, |_p| true);
        assert_eq!(found, expected, "found: {found:#?} expected: {expected:#?}",);

        Ok(())
    }

    #[test]
    fn test_highlight_text() {
        // windows paths and separator
        assert_eq!(
            highlight_text(
                Path::new("C:\\some\\bin\\lsx.exe"),
                "LS",
                Color::Red,
                Some('\\')
            ),
            "C:\\some\\bin\\\u{1b}[1;31mls\u{1b}[0mx.exe"
        );

        // unix paths and separator
        assert_eq!(
            highlight_text(Path::new("/usr/bin/lsx"), "LS", Color::Red, Some('/')),
            "/usr/bin/\u{1b}[1;31mls\u{1b}[0mx"
        );
    }
}
