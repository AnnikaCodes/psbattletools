#![feature(iter_intersperse, test)]
// Command-line program to manage PS battle logs.

mod directory;
mod statistics;
mod testing;

use directory::ParallelDirectoryParser;

use crate::statistics::StatisticsDirectoryParser;

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
    println!("Hello, world!");
    let format_dir = std::path::PathBuf::from("test-scratch");
    let mut parser = StatisticsDirectoryParser::new(None);
    parser.handle_directory(format_dir).unwrap();
}
