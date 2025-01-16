# superwhich

`superwhich` is a cross-platform CLI tool that was initially meant to be a faster drop-in replacement for Windows' `where` command but since it uses [Jaro-Winkler distance](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) to calculate the similarity between the strings it can be called a sort of "smart" which, it can handle some typos and highlights the section of the executables that matches the search pattern.

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

Usage: swhich [OPTIONS] <PATTERN>

Arguments:
  <PATTERN>  The search pattern

Options:
  -c, --color <COLOR>          Color of the highlighted text (off for no color) [default: blue]
  -j, --threshold <THRESHOLD>  Jaro-Winkler distance threshold (0.0 to 1.0) [default: 0.8]
  -t, --print-time             Print time elapsed
  -h, --help                   Print help
  -V, --version                Print version
```

## Todo

- Make it faster (currently at `~240ms` on Windows).
- Find a better way to match the pattern to the name when printing the result.

## Benchmarks

The benchmarks were run using [Hyperfine](https://github.com/sharkdp/hyperfine).

### Machine A

- AMD64, 32GB RAM, Ryzen 7 3800X, Windows 10.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `where pnpm` | 98.9 ± 1.0 | 97.4 | 101.1 | 1.00 |
| `swhich pnpm` | 235.1 ± 4.2 | 229.7 | 243.8 | 2.38 ± 0.05 |

### Machine B

- ARM64, 1GB RAM, Orange Pi Zero2, Debian 12.

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `which lookfor` | 2.2 ± 0.2 | 1.9 | 3.7 | 1.00 |
| `swhich lookfor` | 16.2 ± 0.3 | 15.7 | 17.1 | 7.21 ± 0.71 |

## License

This project is licensed under the terms of the [GNU General Public License](LICENSE) v3.0.
