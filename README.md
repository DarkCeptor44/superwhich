# superwhich

`superwhich` is a cross-platform CLI tool and library that was initially meant to be a faster drop-in replacement for Windows' `where` command but since it uses [Jaro-Winkler distance](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) to calculate the similarity between the strings it can be called a sort of "smart" which, it can handle some typos and highlights the section of the executables that matches the search pattern.

## Installation

- From [crates.io](https://crates.io/crates/superwhich): `cargo install superwhich`
- From [GitHub](https://github.com/DarkCeptor44/superwhich): `cargo install --git https://github.com/DarkCeptor44/superwhich`
- Manually (after cloning the repo locally): `cargo install --path .`
- From [releases](https://github.com/DarkCeptor44/superwhich/releases/latest).

## Usage

![usage-windows](assets/usage-windows.png)
![usage-linux](assets/usage-linux.png)

```sh
$ swhich -h
Cross-platform smart which alternative

Usage: swhich.exe [OPTIONS] <PATTERN>

Arguments:
  <PATTERN>  The search pattern

Options:
  -c, --color <COLOR>          Color of the highlighted text (off or set `NO_COLOR` env var to disable) [default: blue]
  -T, --threshold <THRESHOLD>  String similarity threshold (0.0 to 1.0) [default: 0.7]
  -t, --print-time             Print time elapsed
  -h, --help                   Print help
  -V, --version                Print version
```

## Todo

- Make it faster (currently at `~221ms` on Windows).
- Find a better way to match the pattern to the name when printing the result so it highlights similar strings as well.

## Tests

You can run the tests with `cargo test`.

## Benchmarks

### Library

The library benchmarks can be ran with `cargo bench`.

| Benchmark | Min Mean Max | Outliers |
| --------- | ------------ | -------- |
| `find_executables/fake binaries` | 73.288 µs 73.486 µs 73.725 µs | 4 (4.00%) high mild, 6 (6.00%) high severe |
| `find_executables/real PATH`     | 210.48 ms 211.14 ms 211.84 ms | 4 (4.00%) high mild, 1 (1.00%) high severe |
| `highlight_text`                 | 1.4787 µs 1.4829 µs 1.4872 µs | 2 (2.00%) low mild, 1 (1.00%) high mild, 3 (3.00%) high severe |

### CLI

The CLI was benchmarked using [Hyperfine](https://github.com/sharkdp/hyperfine).

#### Windows

- AMD64, 32GB RAM, Ryzen 7 3800X, Windows 10.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `where pnpm` | 94.7 ± 1.1 | 93.5 | 98.7 | 1.00 |
| `swhich pnpm` | 221.8 ± 3.5 | 215.7 | 229.2 | 2.34 ± 0.05 |

#### Linux

- ARM64, 1GB RAM, Orange Pi Zero2, Debian 12.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `which lookfor` | 2.2 ± 0.2 | 1.9 | 3.7 | 1.00 |
| `swhich lookfor` | 16.2 ± 0.3 | 15.7 | 17.1 | 7.21 ± 0.71 |

## License

This project is licensed under the terms of the [GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0.html).
