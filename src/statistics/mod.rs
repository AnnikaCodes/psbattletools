// Code for the `psbattletools statistics` subcommand.
mod stats;

use crate::{directory::*, BattleToolsError};
use stats::{GameResult, Stats, StatsOutput};

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
        Stats::process_json(self.min_elo, &raw_json)
    }

    fn handle_results(&mut self, results: Vec<Vec<GameResult>>) -> Result<(), BattleToolsError> {
        for result in results {
            self.stats.add_game_results(result);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use lazy_static::lazy_static;
    use std::{fs, path::PathBuf};
    use test::Bencher;

    lazy_static! {
        static ref TEST_DIR: PathBuf = PathBuf::from("test-scratch/day1");
    }

    fn build_test_dir(num_files: u32) -> std::io::Result<()> {
        let src_file = &PathBuf::from("src/benchmark-data.json");
        fs::create_dir_all(&TEST_DIR.clone())?;
        for i in 0..num_files {
            let mut file = TEST_DIR.clone();
            file.push(format!("{}.json", i));
            fs::copy(src_file, file)?;
        }
        Ok(())
    }

    #[bench]
    fn bench_handle_directory_1k(b: &mut Bencher) {
        build_test_dir(1_000).unwrap();

        let format_dir = TEST_DIR.parent().unwrap().to_owned();
        let mut parser = StatisticsDirectoryParser::new(None);
        b.iter(|| parser.handle_directory(format_dir.clone()).unwrap());
    }

    #[test]
    fn test_handle_directory_1k() {
        build_test_dir(1_000).unwrap();
        let format_dir = TEST_DIR.parent().unwrap().to_owned();
        let mut parser = StatisticsDirectoryParser::new(None);
        parser.handle_directory(format_dir).unwrap();
        let mut stats = parser.stats;

        assert_eq!(
            stats.to_csv(),
            "Rotom-Fan,1000,1000,100,31.622776
Regirock,1000,1000,100,31.622776
Conkeldurr,1000,1000,100,31.622776
Reuniclus,1000,1000,100,31.622776
Incineroar,1000,1000,100,31.622776
Miltank,1000,1000,100,31.622776
Drednaw,1000,0,0,-31.622776
Pinsir,1000,0,0,-31.622776
Pikachu,1000,0,0,-31.622776
Latios,1000,0,0,-31.622776
Entei,1000,0,0,-31.622776
Exeggutor-Alola,1000,0,0,-31.622776"
        );
        assert_eq!(
            stats.to_human_readable(),
            "+------+-----------------+------------+---------+-------+------+
| Rank | Pokemon         | Deviations | Winrate | Games | Wins |
+------+-----------------+------------+---------+-------+------+
| 1    | Rotom-Fan       | 31.622776  | 100%    | 1000  | 1000 |
+------+-----------------+------------+---------+-------+------+
| 2    | Regirock        | 31.622776  | 100%    | 1000  | 1000 |
+------+-----------------+------------+---------+-------+------+
| 3    | Conkeldurr      | 31.622776  | 100%    | 1000  | 1000 |
+------+-----------------+------------+---------+-------+------+
| 4    | Reuniclus       | 31.622776  | 100%    | 1000  | 1000 |
+------+-----------------+------------+---------+-------+------+
| 5    | Incineroar      | 31.622776  | 100%    | 1000  | 1000 |
+------+-----------------+------------+---------+-------+------+
| 6    | Miltank         | 31.622776  | 100%    | 1000  | 1000 |
+------+-----------------+------------+---------+-------+------+
| 7    | Drednaw         | -31.622776 | 0%      | 1000  | 0    |
+------+-----------------+------------+---------+-------+------+
| 8    | Pinsir          | -31.622776 | 0%      | 1000  | 0    |
+------+-----------------+------------+---------+-------+------+
| 9    | Pikachu         | -31.622776 | 0%      | 1000  | 0    |
+------+-----------------+------------+---------+-------+------+
| 10   | Latios          | -31.622776 | 0%      | 1000  | 0    |
+------+-----------------+------------+---------+-------+------+
| 11   | Entei           | -31.622776 | 0%      | 1000  | 0    |
+------+-----------------+------------+---------+-------+------+
| 12   | Exeggutor-Alola | -31.622776 | 0%      | 1000  | 0    |
+------+-----------------+------------+---------+-------+------+
"
        )
    }
}
