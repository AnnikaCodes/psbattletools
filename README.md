# psbattletools
![Crates.io](https://img.shields.io/crates/v/psbattletools) [![codecov](https://codecov.io/gh/AnnikaCodes/psbattletools/branch/main/graph/badge.svg?token=AA6F4XJVIA)](https://codecov.io/gh/AnnikaCodes/psbattletools)

`psbattletools` is a command-line tool written in Rust for manipulating [Pokémon Showdown](https://github.com/smogon/pokemon-showdown) battle logs.
## Installation
`psbattletools` currently requires nightly Rust, since it uses Rust's built-in benchmarker. If you don't have nightly Rust, install [`rustup`](https://rustup.rs/), then run `rustup install nightly`.

Installing `psbattletools` itself is as simple as running `cargo install psbattletools`.
## Usage
### Searching for battles
The `search` or `s` subcommand allows you to search for battles; this is significantly faster than Showdown's built-in `/battlesearch` functionality (TODO: add benchmark).

You must specify a username to search for, and a list of directories to search in (these will be searched recursively, ignoring non-JSON files). You may also specify any of the following options:
- `--forfeits-only` or `-f`: search only for battles that ended by forfeit
- `--wins-only` or `-w`: search only for battles that the specified username won

For example, `psbattletools search --wins-only Annika logs/2020-06/gen8randombattle logs/2020-05/gen8randombattle` would search for [Gen 8] Random Battle battles won by Annika in May and June of 2020.
### Calculating winrates and statistics
The `statistics` (or `stats` or `winrates`) subcommand calculates the winrates (in percentage, raw games, and standard deviations) for each Pokémon used in a given format. Currently, usage stats are handled by other, closed-source scripts, but if there is demand for it I'm happy to consider implementing more complex statistics.

By default, this command prints a human-readable winrates format to standard output, but there are also options to produce CSVs (for easy consumption by scripts and programs), and/or write output to a file.

You must provide a list of directories to analyze, but `psbattletools statistics` also accepts the following optional arguments:
- `--csv [path]`: writes CSV output to the given file
- `--human-readable [path]` or `--pretty [path]`: writes human-readable ([prettytable](https://crates.io/crates/prettytable-rs)) output to the given file
- `--minimum-elo [ELO]` or `--elo [ELO]`: ignores battles where either player is below the given ELO rating at the start of the battle
- `--exclude [text]`: ignores directories and JSON files whose names include the given text

For example, `psbattletools stats --pretty gen8randombattle-1500.txt --minimum-elo 1500 logs/2021-08/gen8randombattle` would write winrates for [Gen 8] Random Battle games in August 2021 with 1500 ELO or higher as human-readable tables to the file `gen8randombattle-1500.txt`.
### Anonymizing battles
The `anonymize` subcommand removes personally-identifying data from battle logs, while assigning each player a unique ID (so it's still possible to identify when two anonymous players are the same).

You must provide a list of directories containing JSON files to anonymize; **all** of the JSON battle logs in these directories will be anonymized, and directory structure will not be preserved in the output. This subcommand accepts only one argument, which must be specified: `--output [directory]` (or `-o [directory]`), which specifies the directory in which anonymized battle logs will be written.

For example, to write anonymized [Gen 8] Random Battle logs from June-August 2021 to the directory `anonymized/`, you'd use the command `psbattletools anonymize -o anonymized logs/2021-06/gen8randombattle logs/2021-07/gen8randombattle logs/2021-08/gen8randombattle`.
## Development
I welcome contributions to `psbattletools`. There's currently no formal contribution guide, but pull requests are always welcome. If possible, make sure your code is `rustfmt`ed and has unit test(s) to detect regressions and/or test added functionality.

Unit and integration tests can be run with `cargo test`, and benchmarks can be run with `cargo bench`. The following features may be enabled during benchmarks (with `cargo bench --features ...`) to run additional benchmarks:
- `bench_old_battlesearch` runs the same benchmark as is used for the integration benchmark of `psbattletools search` on my old [`battlesearch`](https://crates.io/crates/battlesearch) program; this allows for direct performance comparisons. Don't enable this feature unless you have `battlesearch` installed.
- `bench_old_winrates` runs the same benchmark as is used for the integration benchmark of `psbattletools statistics` on my old [`randbats-winrates`](https://crates.io/crates/randbats-winrates) program; this allows for direct performance comparisons. Don't enable this feature unless you have `randbats-winrates` installed.
- `bench_old_anonbattle` runs the same benchmark as is used for the integration benchmark of `psbattletools anonymize` on my old [`anonbattle`](https://crates.io/crates/anonbattle) program; this allows for direct performance comparisons. Don't enable this feature unless you have `anonbattle` installed.