#![feature(iter_intersperse, test)]
// Command-line program to manage PS battle logs.

mod directory;
mod statistics;
mod testing;

use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
enum Subcommand {
    #[structopt(name = "statistics", alias = "stats", alias = "winrates")]
    Statistics {
        #[structopt(
            help = "A list of directories to calculate statistics for",
            required(true),
            parse(from_os_str)
        )]
        directories: Vec<PathBuf>,
        #[structopt(
            long = "csv",
            help = "A path to a file to which statistics will be written in CSV format"
        )]
        csv_path: Option<PathBuf>,
        #[structopt(
            long = "human-readable",
            alias = "pretty",
            help = "A path to a file to which statistics will be written in human-readable (table) format"
        )]
        human_readable_path: Option<PathBuf>,
        #[structopt(
            long = "minimum-elo",
            alias = "elo",
            help = "Battles in which either player is below this ELO rating will be ignored"
        )]
        minimum_elo: Option<u64>,
    },
    #[structopt(name = "search", alias = "s")]
    Search {
        #[structopt(help = "Search for battles played by this user", required(true))]
        username: String,
        #[structopt(
            help = "A list of directories to search for matching battle logs in",
            required(true),
            parse(from_os_str)
        )]
        directories: Vec<PathBuf>,
        #[structopt(
            long = "wins-only",
            short = "w",
            help = "Search only for battles that were won by the username you're searching for"
        )]
        wins_only: bool,
        #[structopt(
            long = "forfeits-only",
            short = "f",
            help = "Search only for battles that ended by forfeit"
        )]
        forfeits_only: bool,
    },
    #[structopt(name = "anonymize")]
    Anonymize {
        #[structopt(
            help = "A list of directories containing battle logs to anonymize",
            required(true),
            parse(from_os_str)
        )]
        directories: Vec<PathBuf>,
        #[structopt(
            long = "output",
            short = "o",
            help = "The directory to write all anonymized battle logs to",
            required(true),
            parse(from_os_str)
        )]
        output_dir: PathBuf,
    },
}

#[derive(StructOpt)]
#[structopt(
    author = "Annika L.",
    about = "Provides various tools for Pokémon Showdown battle logs"
)]
struct Options {
    // TODO: implement
    #[structopt(
        long = "exclude",
        help = "Filenames and directories including this string will be ignored"
    )]
    exclude: Option<String>,
    // TODO: implement
    #[structopt(
        long = "threads",
        short = "j",
        help = "The maximum number of threads to use for concurrent processing"
    )]
    threads: Option<usize>,
    #[structopt(subcommand)]
    command: Subcommand,
}

#[derive(Debug)]
pub enum BattleToolsError {
    IOError(std::io::Error),
}
impl From<std::io::Error> for BattleToolsError {
    fn from(error: std::io::Error) -> Self {
        BattleToolsError::IOError(error)
    }
}

fn main() {
    let options = Options::from_args();
    match options.command {
        Subcommand::Statistics {
            directories,
            csv_path,
            human_readable_path,
            minimum_elo,
        } => {
            unimplemented!();
        }
        Subcommand::Search {
            username,
            directories,
            wins_only,
            forfeits_only,
        } => {
            unimplemented!();
        }
        Subcommand::Anonymize {
            directories,
            output_dir,
        } => {
            unimplemented!();
        }
    }
}
