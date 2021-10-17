// Code for handling directories

use rayon::prelude::*;
use std::{
    fs,
    marker::{Send, Sync},
    path::PathBuf,
};

use crate::BattleToolsError;

/// Anything that wants to parse logs should implement this
trait LogParser<R> {
    /// Parses an individual log file's JSON
    fn handle_log_file(&self, raw_json: String) -> Result<R, BattleToolsError>;
    /// Parses the results from an entire directory.
    /// May be called multiple times (once with the results of each recursive directory)
    ///
    /// TODO: is it better to make this accept a FilterMap<R>?
    fn handle_results(&self, results: Vec<R>) -> Result<(), BattleToolsError>;
}

/// Iterates over directories and executes code on each log file in parallel.
trait ParallelDirectoryParser<R> {
    fn handle_directory(&self, dir: &PathBuf) -> Result<(), BattleToolsError>;
}

impl<T, R> ParallelDirectoryParser<R> for T
where
    T: LogParser<R> + Sync,
    R: Send,
{
    fn handle_directory(&self, path: &PathBuf) -> Result<(), BattleToolsError> {
        let results = fs::read_dir(path)?
            .collect::<Vec<_>>()
            .par_iter()
            .filter_map(|file| {
                if let Ok(entry) = file.as_ref() {
                    if entry.file_type().ok()?.is_dir() {
                        self.handle_directory(&entry.path()).ok()?;
                    }
                    match self.handle_log_file(fs::read_to_string(entry.path()).ok()?) {
                        Ok(r) => Some(r),
                        Err(e) => {
                            eprintln!("{:?}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<R>>();
        self.handle_results(results)
    }
}
