// Code for handling directories

use rayon::prelude::*;
use std::{
    borrow::Borrow,
    fs,
    marker::{Send, Sync},
    path::PathBuf,
    sync::Mutex,
};

use crate::BattleToolsError;

/// Anything that wants to parse logs should implement this
pub trait LogParser<R> {
    /// Parses an individual log file's JSON
    fn handle_log_file(&self, raw_json: String) -> Result<R, BattleToolsError>;
    /// Parses the results from an entire directory.
    /// May be called multiple times (once with the results of each recursive directory)
    ///
    /// TODO: is it better to make this accept a FilterMap<R>?
    fn handle_results(&mut self, results: Vec<R>) -> Result<(), BattleToolsError>;
}

/// Iterates over directories and executes code on each log file in parallel.
pub trait ParallelDirectoryParser<R> {
    fn handle_directory(&mut self, dir: PathBuf) -> Result<(), BattleToolsError>;
}

impl<T, R> ParallelDirectoryParser<R> for T
where
    T: LogParser<R> + Sync + Send,
    R: Send,
{
    fn handle_directory(&mut self, path: PathBuf) -> Result<(), BattleToolsError> {
        // We don't know if we'll get a directory with lots of subdirectories or one with lots of JSON files,
        // so we always use parallel iteration.
        // However, it's not too much of a performance to use a mutex, since in PS log structures, there are
        // not generally JSON logs and subdirectories in the same directory.
        let mut sub_directories = vec![path];
        let mut results: Vec<R> = vec![];

        let subdirectories_mutex = Mutex::new(sub_directories);
        let handle_specific_dir = |p: &PathBuf| -> Result<Vec<R>, BattleToolsError> {
            let result_vec = fs::read_dir(p)?
                .collect::<Vec<_>>()
                .par_iter()
                .filter_map(|file| {
                    if let Ok(entry) = file.as_ref() {
                        if entry.file_type().ok()?.is_dir() {
                            subdirectories_mutex.lock().ok()?.push(entry.path());
                            return None;
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
                .collect();
            Ok(result_vec)
        };

        // We start at 0, which is the provided method argument `path`.
        // No values are ever removed from the vector, so we can just keep going until we reach the end;
        // since `handle_specific_dir` blocks, when a call to `handle_specific_dir` adds directories to the vector,
        // we'll handle them in the next iterations of the while loop.
        // TODO: see if this actually works
        let mut last_handled_dir_idx = 0;
        while let Some(dir) = subdirectories_mutex
            .lock()
            .unwrap()
            .get(last_handled_dir_idx)
        {
            if let Ok(r) = handle_specific_dir(dir) {
                results.extend(r);
            }
            last_handled_dir_idx += 1;
        }
        self.handle_results(results)
    }
}
