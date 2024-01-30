use std::{fs, path::PathBuf};

macro_rules! get_test_content {
    () => {
        
    };
}


#[test]
fn feature() {
    let content = read_test_file("_main/calc.less");
    dbg!(content);
}

pub fn read_test_file(sub_path: &str) -> String {
    dbg!();
    let root_path = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(root_path);
    path.push("test-data/less");
    path.push(sub_path);
    fs::read_to_string(path).unwrap()
}