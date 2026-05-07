use std::fs;
use std::path::PathBuf;

pub fn log_to_file(directory: PathBuf, name: String, content: String) -> String {
    let file_path = format!("{}/latest-{}.log", directory.to_str().unwrap(), name);
    let mut path = PathBuf::from(&file_path);
    if path.exists() {
        fs::remove_file(path.clone()).expect("fuck, something went wrong    ")
    }
    let _ = fs::write(&file_path, content);
    if path.clone().exists() {
        path = path.clone().canonicalize().unwrap_or(path);
        String::from(path.to_str().unwrap())
    } else {
        String::from("<File could not be created>")
    }
}
