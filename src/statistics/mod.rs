// Code for the `psbattletools statistics` subcommand.
mod stats;

use crate::{directory::LogParser, BattleToolsError};
use stats::{GameResult, Stats};

pub type StatsMutexError<'a> =
    std::sync::PoisonError<std::sync::MutexGuard<'a, Vec<std::path::PathBuf>>>;

/// Parses a directory and computes winrates on the battles within.
pub struct StatisticsDirectoryParser {
    min_elo: u64,
    pub stats: Stats,
}

impl StatisticsDirectoryParser {
    pub fn new(min_elo: Option<u64>) -> Self {
        Self {
            min_elo: min_elo.unwrap_or(0),
            stats: Stats::new(),
        }
    }
}

impl LogParser<Vec<GameResult>> for StatisticsDirectoryParser {
    fn handle_log_file(&self, raw_json: String) -> Result<Vec<GameResult>, BattleToolsError> {
        // TODO: potentially optimize by not comparing min_elo if unnecessary
        Stats::process_json(self.min_elo, raw_json)
    }

    fn handle_results(&mut self, results: Vec<Vec<GameResult>>) -> Result<(), BattleToolsError> {
        for result in results {
            self.stats.add_game_results(result);
        }
        Ok(())
    }
}
