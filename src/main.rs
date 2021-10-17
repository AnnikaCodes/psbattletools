#![feature(iter_intersperse, test)]
// Command-line program to manage PS battle logs.

mod directory;
mod statistics;

use statistics::StatsMutexError;
use std::{any::Any, sync::MutexGuard};

#[derive(Debug)]
pub enum BattleToolsError<'a> {
    IOError(std::io::Error),
    StatsMutexError(StatsMutexError<'a>),
}
impl<'a> From<std::io::Error> for BattleToolsError<'a> {
    fn from(error: std::io::Error) -> Self {
        BattleToolsError::IOError(error)
    }
}
impl<'a> From<StatsMutexError<'a>> for BattleToolsError<'a> {
    fn from(error: StatsMutexError<'a>) -> Self {
        BattleToolsError::StatsMutexError(error)
    }
}

fn main() {
    println!("Hello, world!");
}
