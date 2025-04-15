use colored::Color;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{
    env::{split_paths, var_os},
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};
use superwhich::{find_executables, highlight_text};
use tempfile::tempdir;

fn bench_find_executables(c: &mut Criterion) {
    let mut g = c.benchmark_group("find_executables");
    let temp_dir = tempdir().unwrap();
    let example_bins = ["ll", "la", "lsx", "lsa", "as", "grep", "dd", "rm", "rmz"];

    create_dir_all(temp_dir.path()).unwrap();

    for bin in example_bins {
        let path = temp_dir.path().join(format!("{bin}.exe"));
        let mut file = File::create(&path).unwrap();

        file.write_all(b"test").unwrap();
        drop(file);
    }

    g.bench_function("fake binaries", |b| {
        b.iter(|| {
            black_box(find_executables(
                black_box(&[temp_dir.path().to_path_buf()]),
                black_box("LS"),
                black_box(0.8),
                |_p| black_box(true),
            ))
        });
    });

    let paths: Vec<PathBuf> = split_paths(&var_os("PATH").unwrap()).collect();
    g.bench_function("real PATH", |b| {
        b.iter(|| {
            black_box(find_executables(
                black_box(&paths),
                black_box("LS"),
                black_box(0.8),
                black_box(is_executable::IsExecutable::is_executable),
            ))
        });
    });
}

fn bench_highlight_text(c: &mut Criterion) {
    c.bench_function("highlight_text", |b| {
        b.iter(|| {
            black_box(highlight_text(
                Path::new("/usr/bin/lsx"),
                "LS",
                Color::Red,
                None,
            ))
        });
    });
}

criterion_group!(benches, bench_find_executables, bench_highlight_text);
criterion_main!(benches);
