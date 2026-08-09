use serde;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Serialize, Deserialize)]
pub struct Uniconf {
    pub bin_name: String,
    pub lang: String,
    pub compile_instruct: Vec<String>,
    pub bin_location: Vec<String>, //. for path root
    pub install_command: Vec<String>,
    pub clean_command: Vec<String>,
    pub license: String,
    pub author: String,
}

pub fn parse_conf(config_path: &PathBuf) -> Uniconf {
    if !config_path.is_file()
        && !config_path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap()
            .to_string()
            .trim()
            .ends_with(".json")
    {
        exit(1)
    }

    let string = fs::read_to_string(config_path).unwrap_or_else(|_| panic!("Failed to read to string"));

    serde_json::from_str(string.as_str()).unwrap()
    
}
pub fn create_conf() {
    todo!()
}
pub(crate) fn test() {
    let conf: Uniconf = Uniconf {
        bin_name: "test".to_string(),
        lang: "Rust".to_string(),
        compile_instruct: vec!["echo hi".to_string()],
        bin_location: vec!["./out".to_string()],
        install_command: vec!["echo hi".to_string()],
        clean_command: vec!["echo hi".to_string()],
        license: "MIT".to_string(),
        author: "notchapplez".to_string(),
    };
    let json_string = serde_json::to_string(&conf);
    println!("{}", json_string.unwrap());
}
