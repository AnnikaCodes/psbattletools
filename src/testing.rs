// Helper functions for testing
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref TEST_ROOT_DIR: PathBuf = PathBuf::from("test-scratch");
    pub static ref TEST_ROOT_DIR_DAY1: PathBuf = PathBuf::from("test-scratch/day1");
    pub static ref TEST_ROOT_DIR_DAY2: PathBuf = PathBuf::from("test-scratch/day2");
}

pub fn build_test_dir(num_files: u32) -> std::io::Result<()> {
    let src_file = &PathBuf::from("src/benchmark-data.json");

    for dir in [&*TEST_ROOT_DIR_DAY1, &*TEST_ROOT_DIR_DAY2] {
        std::fs::create_dir_all(dir)?;
    }

    let pivot = num_files / 2;
    for i in 0..num_files {
        let mut file = if i > pivot {
            TEST_ROOT_DIR_DAY1.clone()
        } else {
            TEST_ROOT_DIR_DAY2.clone()
        };

        file.push(format!("{}.json", i));
        std::fs::copy(src_file, &file)
            .expect(&format!("error copying from {:?} to {:?}", src_file, file));
    }
    Ok(())
}
