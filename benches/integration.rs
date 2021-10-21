#![feature(test)]
// Integration benchmarks testing an entire binary.

extern crate test;
use std::{path::PathBuf, process::Command};

use test::*;
use tests::*;

// Benchmarks
#[bench]
fn bench_stats_1k(b: &mut Bencher) {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target");
    path.push("release");
    if !path.exists() {
        // probably not benchmarking - `cargo test` was invoked instead
        eprintln!("{:?} doesn't exist - ignoring benchmark", path);
        return;
    }
    path.push("psbattletools");

    let mut cmd = Command::new(&path);
    cmd
        .arg("statistics")
        .arg(path)
        .arg(&*TEST_ROOT_DIR)
        .arg("--csv")
        .arg("test-scratch/new.csv");

    b.iter(|| cmd.output().expect("Failed to execute command"));
}

// Benchmark against the old `randbats-winrates` program
#[cfg_attr(not(feature = "bench_old_winrates"), ignore)]
#[bench]
fn bench_old_winrates_1k(b: &mut Bencher) {
    build_test_dir(1_000).unwrap();

    let mut cmd = Command::new("randbats-winrates");
    cmd
        .arg("--input").arg(&*TEST_ROOT_DIR)
        .arg("--minimum-elo").arg("0")
        .arg("--csv-output")
        .arg("test-scratch/old.csv");

    b.iter(|| cmd.output().expect("Failed to execute command"));
}

#[bench]
fn bench_search_1k(b: &mut Bencher) {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target");
    path.push("release");
    if !path.exists() {
        // probably not benchmarking - `cargo test` was invoked instead
        eprintln!("{:?} doesn't exist - ignoring benchmark", path);
        return;
    }
    path.push("psbattletools");

    let mut cmd = Command::new(&path);
    cmd.arg("search").arg("AnniKa").arg(&*TEST_ROOT_DIR);

    b.iter(|| cmd.output().expect("Failed to execute command"));
}

// Benchmark against the old `battlesearch` program
#[cfg_attr(not(feature = "bench_old_battlesearch"), ignore)]
#[bench]
fn bench_old_battlesearch_1k(b: &mut Bencher) {
    build_test_dir(1_000).unwrap();

    let mut cmd = Command::new("battlesearch");
    cmd.arg("AnniKa").arg(&*TEST_ROOT_DIR);

    b.iter(|| cmd.output().expect("Failed to execute command"));
}
