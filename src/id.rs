// Code for converting to Pokémon Showdown's string IDs.
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ID_REGEX: Regex = Regex::new(r"[^A-Za-z0-9]").unwrap();
}

// See https://github.com/smogon/pokemon-showdown/blob/ddb6010bb92963fb50771aaf5a052fff29a82135/sim/dex-data.ts#L9-L35
// for PS's implementation.
pub fn to_id(str: &str) -> String {
    (*ID_REGEX.replace_all(str, "")).to_lowercase()
}
