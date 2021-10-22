#![feature(test)]
use std::path::PathBuf;
use std::process::Command;

use lazy_static::lazy_static;
use tests::*;

extern crate test;

///
/*
       assert_ne!(json, *SAMPLE_JSON);

       for term in ["00:00:01", "Annika", "annika", "Rust Haters", "rusthaters"] {
           assert!(
               !json.contains(term),
               "Identifying information in anonymized JSON ('{}' in '{}')",
               term,
               json
           );
       }

       for property in ["p1rating", "p2rating", "roomid"] {
           let value = gjson::get(&json, property);
           assert!(
               !value.exists() || value.kind() == gjson::Kind::Null,
               "Anonymized JSON includes potentially-identifying property '{}' (full JSON: '{}')",
               property,
               json
           );
       }
*/
#[test]
fn test_no_terms_and_identicality() {
    build_test_dir(1_000).unwrap();
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("target/debug/psbattletools");
    let out_dir = PathBuf::from("test-scratch/anonymized");
    Command::new(&path)
        .arg("anonymize")
        .arg(&*TEST_ROOT_DIR)
        .arg("--o")
        .arg(&out_dir)
        .output()
        .expect("Failed to execute command");

    let mut out_file_1 = out_dir.clone();
    out_file_1.push("1.log.json");
    let output_1 = std::fs::read_to_string(&out_file_1)
        .expect(&format!("Couldn't read output file {:?}", out_file_1));

    let mut out_file_999 = out_dir.clone();
    out_file_999.push("999.log.json");
    let output_999 = std::fs::read_to_string(out_file_999).expect("Couldn't read output file");

    for term in ["00:00:01", "Annika", "annika", "Rust Haters", "rusthaters"] {
        assert!(
            !output_1.contains(term),
            "Identifying information in anonymized JSON ('{}' in '{}')",
            term,
            output_1
        );

        assert!(
            !output_999.contains(term),
            "Identifying information in anonymized JSON ('{}' in '{}')",
            term,
            output_999
        );
    }

    assert_eq!(output_1, output_999);
}
