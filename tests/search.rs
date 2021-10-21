#![feature(test)]
use std::path::PathBuf;
use std::process::Command;

use tests::*;

extern crate test;
use test::*;

#[test]
fn test_search() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");

    let output = Command::new(path)
        .arg("search")
        .arg("AnniKa")
        .arg(&*TEST_ROOT_DIR)
        .output()
        .expect("Failed to execute command");

    let output_str = std::str::from_utf8(&output.stdout).unwrap();
    assert!(output_str.contains("annika vs. rusthaters (annika won normally)"));
    assert!(output_str.split('\n').collect::<Vec<_>>().len() >= 1000);
}

#[test]
fn test_search_forfeit() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");

    for forfeit_arg in ["-f", "--forfeits-only"] {
        let output = Command::new(&path)
            .arg("search")
            .arg("AnniKa")
            .arg(&*TEST_ROOT_DIR)
            .arg(forfeit_arg)
            .output()
            .expect("Failed to execute command");

        let output_str = std::str::from_utf8(&output.stdout).unwrap();
        assert!(!output_str.contains("annika"));
        assert!(!output_str.contains("Annika"));
        assert!(output_str.split('\n').collect::<Vec<_>>().len() <= 5); // 5 lines permitted for "parsing..." etc
    }
}

#[test]
fn test_search_wins_only() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");

    for win_arg in ["-w", "--wins-only"] {
        // Annika always wins in the test logs, so when searching for _her_ wins, we should get 1000 results
        let annika_output = Command::new(&path)
            .arg("search")
            .arg("AnniKa")
            .arg(&*TEST_ROOT_DIR)
            .arg(win_arg)
            .output()
            .expect("Failed to execute command");

        let annika_output_str = std::str::from_utf8(&annika_output.stdout).unwrap();
        assert!(annika_output_str.contains("annika vs. rusthaters (annika won normally)"));
        assert!(annika_output_str.split('\n').collect::<Vec<_>>().len() >= 1000);

        // Rust Haters always loses in the test logs, so when searching for _their_ wins, we should get 0 results
        let rust_haters_output = Command::new(&path)
            .arg("search")
            .arg("rusthaters")
            .arg(&*TEST_ROOT_DIR)
            .arg(win_arg)
            .output()
            .expect("Failed to execute command");

        let rust_haters_output_str = std::str::from_utf8(&rust_haters_output.stdout).unwrap();
        assert!(!rust_haters_output_str.contains("annika"));
        assert!(!rust_haters_output_str.contains("Annika"));
        assert!(rust_haters_output_str.split('\n').collect::<Vec<_>>().len() <= 5);
    }
}

// Benchmarks
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
