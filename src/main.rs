#![feature(iter_intersperse, test)]
// Command-line program to manage PS battle logs.

mod anonymize;
mod directory;
mod id;
mod search;
mod statistics;

use anonymize::AnonymizingDirectoryParser;
use directory::ParallelDirectoryParser;
use search::BattleSearcher;
use statistics::{StatisticsDirectoryParser, StatsOutput};
use std::{fs, path::PathBuf};
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
    #[structopt(subcommand)]
    command: Subcommand,
    #[structopt(
        long = "exclude",
        help = "Filenames and directories including this string will be ignored"
    )]
    exclude: Option<String>,
    #[structopt(
        long = "threads",
        short = "j",
        help = "The maximum number of threads to use for concurrent processing"
    )]
    threads: Option<usize>,
}

#[derive(Debug)]
pub enum BattleToolsError {
    IOError(std::io::Error),
    JSONParsingError(serde_json::Error),
    RegexError(regex::Error),
    ThreadPoolError(rayon::ThreadPoolBuildError),
    InvalidLog(String),
    PathConversion(String),
    IncompleteAnonymization(String),
}
impl From<std::io::Error> for BattleToolsError {
    fn from(error: std::io::Error) -> Self {
        BattleToolsError::IOError(error)
    }
}
impl From<String> for BattleToolsError {
    fn from(error: String) -> Self {
        BattleToolsError::InvalidLog(error)
    }
}
impl From<serde_json::Error> for BattleToolsError {
    fn from(error: serde_json::Error) -> Self {
        BattleToolsError::JSONParsingError(error)
    }
}
impl From<regex::Error> for BattleToolsError {
    fn from(error: regex::Error) -> Self {
        BattleToolsError::RegexError(error)
    }
}
impl From<rayon::ThreadPoolBuildError> for BattleToolsError {
    fn from(error: rayon::ThreadPoolBuildError) -> Self {
        BattleToolsError::ThreadPoolError(error)
    }
}

fn main() -> Result<(), BattleToolsError> {
    let options = Options::from_args();

    if let Some(threads) = options.threads {
        // see https://stackoverflow.com/questions/59205184/how-can-i-change-the-number-of-threads-rayon-uses
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()?;
    }

    match options.command {
        Subcommand::Statistics {
            directories,
            csv_path,
            human_readable_path,
            minimum_elo,
        } => {
            let mut parser = StatisticsDirectoryParser::new(minimum_elo);
            parser.handle_directories(directories, options.exclude)?;

            let mut produced_output = false;
            if let Some(csv_path) = csv_path {
                fs::write(csv_path, parser.to_csv())?;
                produced_output = true;
            }
            if let Some(human_readable_path) = human_readable_path {
                fs::write(human_readable_path, parser.to_human_readable())?;
                produced_output = true;
            }

            // If we haven't written to any output files, print the results as a pretty-print to stdout
            if !produced_output {
                println!("{}", parser.to_human_readable());
            }
        }
        Subcommand::Search {
            username,
            directories,
            wins_only,
            forfeits_only,
        } => {
            let mut parser = BattleSearcher::new(&username, wins_only, forfeits_only);
            parser.handle_directories(directories, options.exclude)?;
        }
        Subcommand::Anonymize {
            directories,
            output_dir,
        } => {
            // TODO: Add --safe option
            // create dir if needed
            fs::create_dir_all(&output_dir)?;
            let mut anonymizer = AnonymizingDirectoryParser::new(false, output_dir);
            anonymizer.handle_directories(directories, options.exclude)?;
        }
    }

    Ok(())
}
