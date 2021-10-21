#![feature(test)]
use std::path::PathBuf;
use std::process::Command;

use lazy_static::lazy_static;
use tests::*;

extern crate test;

lazy_static! {
    static ref DESIRED_TABLE_OUTPUT: &'static str = "+------+-----------------+------------+---------+-------+------+
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
";
    static ref DESIRED_CSV_OUTPUT: &'static str = "Rotom-Fan,1000,1000,100,31.622776
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
Exeggutor-Alola,1000,0,0,-31.622776";
}

#[test]
fn test_table() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");
    for subcommand in ["stats", "winrates", "statistics"] {
        for table_arg in ["--human-readable", "--pretty"] {
            let out_file = ["test-scratch/pretty-", subcommand, table_arg, ".txt"].join("");
            Command::new(&path)
                .arg(subcommand)
                .arg("--pretty")
                .arg(&out_file)
                .arg(&*TEST_ROOT_DIR)
                .output()
                .expect("Failed to execute command");
            assert_eq!(std::fs::read_to_string(out_file).expect("Couldn't read output file"), *DESIRED_TABLE_OUTPUT);
        }
    }
}

#[test]
fn test_csv() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");
    for subcommand in ["stats", "winrates", "statistics"] {
        let out_file = ["test-scratch/csv-", subcommand, ".csv"].join("");
        Command::new(&path)
            .arg(subcommand)
            .arg("--csv")
            .arg(&out_file)
            .arg(&*TEST_ROOT_DIR)
            .output()
            .expect("Failed to execute command");

        assert_eq!(std::fs::read_to_string(out_file).expect("Couldn't read output file"), *DESIRED_CSV_OUTPUT);
    }
}

#[test]
fn test_default_output() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");
    for subcommand in ["stats", "winrates", "statistics"] {
        let output = Command::new(&path)
            .arg(subcommand)
            .arg(&*TEST_ROOT_DIR)
            .output()
            .expect("Failed to execute command");

        let output_str = std::str::from_utf8(&output.stdout).unwrap();
        assert_eq!(output_str.to_string().strip_suffix('\n').unwrap(), *DESIRED_TABLE_OUTPUT);
    }
}

#[test]
fn test_min_elo() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");
    for min_elo_arg in ["--elo", "--minimum-elo"] {
        // Should provide lots of output with ELO < 1130 and none above
        let normal_output = Command::new(&path)
            .arg("statistics")
            .arg(min_elo_arg)
            .arg("1129")
            .arg(&*TEST_ROOT_DIR)
            .output()
            .expect("Failed to execute command");

        let normal_output_str = std::str::from_utf8(&normal_output.stdout).unwrap();
        assert_eq!(normal_output_str.to_string().strip_suffix('\n').unwrap(), *DESIRED_TABLE_OUTPUT);

        let no_output = Command::new(&path)
            .arg("statistics")
            .arg(min_elo_arg)
            .arg("1131")
            .arg(&*TEST_ROOT_DIR)
            .output()
            .expect("Failed to execute command");

        let no_output_str = std::str::from_utf8(&no_output.stdout).unwrap();
        assert!(!no_output_str.contains('%'));
        assert!(!no_output_str.contains("Rotom"));
    }
}

#[test]
#[ignore] // TOOD: implement exclusions
fn test_exclusions() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");

    // Should provide normal output with an irrelevant exclusion...
    let normal_output = Command::new(&path)
        .arg("statistics")
        .arg("--exclude")
        .arg("something that isn't in filenames")
        .arg(&*TEST_ROOT_DIR)
        .output()
        .expect("Failed to execute command");

    let normal_output_str = std::str::from_utf8(&normal_output.stdout).unwrap();
    assert_eq!(normal_output_str.to_string().strip_suffix('\n').unwrap(), *DESIRED_TABLE_OUTPUT);

    // ...and less output when excluding many files
    let reduced_output = Command::new(&path)
        .arg("statistics")
        .arg("--exclude")
        .arg("6")
        .arg("1131")
        .arg(&*TEST_ROOT_DIR)
        .output()
        .expect("Failed to execute command");

    let reduced_output_str = std::str::from_utf8(&reduced_output.stdout).unwrap();
    assert!(reduced_output_str.len() < normal_output_str.len() / 2);
}
