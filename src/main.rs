// Command-line program to manage PS battle logs.

mod directory_parse;

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
}
