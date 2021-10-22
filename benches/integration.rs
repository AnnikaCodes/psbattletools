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
    cmd.arg("statistics")
        .arg(path)
        .arg(&*TEST_ROOT_DIR)
        .arg("--csv")
        .arg("test-scratch/new.csv");

    b.iter(|| cmd.output().expect("Failed to run command"));
}

// Benchmark against the old `randbats-winrates` program
#[cfg_attr(not(feature = "bench_old_winrates"), ignore)]
#[bench]
fn bench_old_winrates_1k(b: &mut Bencher) {
    build_test_dir(1_000).unwrap();

    let mut cmd = Command::new("randbats-winrates");
    cmd.arg("--input")
        .arg(&*TEST_ROOT_DIR)
        .arg("--minimum-elo")
        .arg("0")
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

#[bench]
fn bench_anonymize_1k(b: &mut Bencher) {
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

    let mut output_path = TEST_ROOT_DIR
        .parent()
        .expect("Couldn't get parent")
        .to_path_buf();
    output_path.push("anon-bench-new");
    let mut cmd = Command::new(&path);
    cmd.arg("anonymize")
        .arg(&*TEST_ROOT_DIR)
        .arg("--output")
        .arg(output_path);

    b.iter(|| cmd.output().expect("Failed to execute command"));
}

// Benchmark against the old `anonbattle` program
#[cfg_attr(not(feature = "bench_old_anonbattle"), ignore)]
#[bench]
fn bench_old_anonymize_1k(b: &mut Bencher) {
    build_test_dir(1_000).unwrap();

    let mut output_path = TEST_ROOT_DIR
        .parent()
        .expect("Couldn't get parent")
        .to_path_buf();
    output_path.push("anon-bench-old");

    let mut cmd = Command::new("anonbattle");
    cmd.arg("-i")
        .arg(&*TEST_ROOT_DIR)
        .arg("-o")
        .arg(output_path)
        .arg("-f")
        .arg("day"); // use day as the format to avoid skipping

    b.iter(|| cmd.output().expect("Failed to execute command"));
}
