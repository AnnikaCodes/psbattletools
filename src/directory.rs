// Code for handling directories

use rayon::prelude::*;
use std::{
    fs,
    marker::{Send, Sync},
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::BattleToolsError;

/// Anything that wants to parse logs should implement this
pub trait LogParser<R> {
    /// Parses an individual log file's JSON
    fn handle_log_file(&self, raw_json: String, file_path: &Path) -> Result<R, BattleToolsError>;
    /// Parses the results from an entire directory.
    /// Guaranteed to be only called once per invocation of ParallelDirectoryParser::handle_directory;
    /// if subdirectories are found, their parse results will be combined together and passed to handle_results.
    ///
    /// TODO: optimizations for when there is no need to handle results at all?
    ///
    /// TODO: is it better to make this accept a FilterMap<R>?
    fn handle_results(&mut self, results: Vec<R>) -> Result<(), BattleToolsError>;
}

/// Iterates over directories and executes code on each log file in parallel.
pub trait ParallelDirectoryParser<R> {
    /// `exclusion` will exclude any directory or file that matches the exclusion
    fn handle_directories(&mut self, dirs: Vec<PathBuf>, exclusion: Option<String>) -> Result<(), BattleToolsError>;
}

impl<T, R> ParallelDirectoryParser<R> for T
where
    T: LogParser<R> + Sync + Send,
    R: Send,
{
    fn handle_directories(&mut self, dirs: Vec<PathBuf>, exclusion: Option<String>) -> Result<(), BattleToolsError> {
        // We don't know if we'll get a directory with lots of subdirectories or one with lots of JSON files,
        // so we always use parallel iteration.

        // We have a Mutex<Vec<PathBuf>> to store directories we need to check. We start with the provided directory,
        // and add to this mutex whenever we find a subdirectory. This means that we can collect ALL the results into
        // one Vec, and guarantee that handle_results is only called once. It's not too much of a performance to use a mutex,
        // since in PS log structures, there are not generally JSON logs and subdirectories in the same directory.
        let mut results: Vec<R> = vec![];

        let subdirectories_mutex = Mutex::new(dirs);
        let handle_specific_dir = |p: &PathBuf| -> Result<Vec<R>, BattleToolsError> {
            eprintln!("Parsing {}...", p.display());
            let result_vec = fs::read_dir(p)?
                .collect::<Vec<_>>()
                .par_iter()
                .filter_map(|file| {
                    if let Ok(entry) = file.as_ref() {
                        if let Some(exclude) = &exclusion {
                            if entry.file_name().to_string_lossy().contains(exclude) {
                                return None;
                            }
                        }

                        if entry.file_type().ok()?.is_dir() {
                            // We found a subdirectory! Add it to the list of directories to process,
                            // then return None (there's no parsed data for a subdirectory!)
                            match subdirectories_mutex.lock() {
                                Ok(mut s) => s.push(entry.path()),
                                Err(e) => {
                                    // We can't just propagate this error, since we're in a filter_map
                                    // If the mutex is poisoned (which shouldn't ever happen?) we can probably just panic
                                    // TODO: make sure the mutex can't ever be poisoned
                                    panic!(
                                        "Mutex for list of directories to process poisoned: {}",
                                        e
                                    );
                                }
                            };
                            return None;
                        }

                        let path = entry.path();
                        let raw_json = match fs::read_to_string(entry.path()) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("Error reading file '{:?}': {:?}", path, e);
                                return None;
                            }
                        };
                        match self.handle_log_file(raw_json, &path) {
                            Ok(res) => Some(res),
                            Err(e) => {
                                eprintln!("Error parsing file '{:?}': {:?}", path, e);
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

        // get_next_dir is a closure so that the mutex is unlocked before handle_specific_dir is called.
        // Otherwise, the Rayon filter_map threads block infinitely when trying to lock the mutex (to add a subdirectory)
        let get_next_dir = || subdirectories_mutex.lock().unwrap().pop();
        while let Some(dir) = get_next_dir() {
            if let Ok(r) = handle_specific_dir(&dir) {
                results.extend(r);
            }
        }
        self.handle_results(results)
    }
}
