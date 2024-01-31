use std::{ffi::OsStr, fs, path::PathBuf};

use glob::{glob, glob_with, MatchOptions};

macro_rules! get_test_content {
    () => {};
}

#[test]
fn feature() {
    test_main_less_feature(|path, content| {
        dbg!(path);
        dbg!(content);
    })
}

pub fn read_test_file(sub_path: &str) -> String {
    dbg!();
    let root_path = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(root_path);
    path.push("test-data/less");
    path.push(sub_path);
    fs::read_to_string(path).unwrap()
}

pub fn test_main_less_feature<F>(cb: F)
where
    F: Fn(&String, &String) -> Result<(), ()>,
{
    let root_path = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(root_path);
    path.push("test-data/less/_main/*.less");

    for entry in glob(path.to_str().unwrap()).unwrap() {
        match entry {
            Ok(path) => {
                let path_name = path.to_str().unwrap().to_string();
                let content = &fs::read_to_string(path).unwrap();
                if cb(&path_name, content).is_err() {
                    break;
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
