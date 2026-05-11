use crate::{
    cmake::compile_cmake, confidcure::init_build, make::build_make, meson::start_meson,
    rust::compile_rust, zig::build_zig,
};

use crate::go::compile_go;
use crate::haskell::compile_haskell;
use colored::Colorize;
use path_absolutize::Absolutize;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::{env, fs, io, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
use unin_bin::{UninPackage, registry_write, time_create};

type UniversalResult<T> = Result<T, Box<dyn std::error::Error>>; //define UniversalResult

#[derive(Debug)]
struct CustomError(Vec<String>);
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Errors: {:?}", self.0)
    }
}
impl std::error::Error for CustomError {}

pub fn absolutize_path(path: String) -> PathBuf {
    let path = PathBuf::from(path);
    let absolutized_path = path.absolutize().unwrap();
    absolutized_path.to_path_buf()
}

pub fn detect(path: String, noinstall: bool) {
    let mut new_path: PathBuf = PathBuf::new();
    if path.is_empty() {
        println!("Path is empty");
        new_path = env::current_dir().unwrap()
    } else {
        new_path.clear();
        new_path.push(&path);
    }
    let entries: HashSet<String> = fs::read_dir(&new_path)
        .unwrap()
        .filter_map(|res| res.ok())
        .map(|e| e.file_name().into_string().unwrap_or_default())
        .collect();

    let priorities = [
        "configure",
        "CMakeLists.txt",
        "meson.build",
        "Cargo.toml",
        "build.zig",
        "go.mod",
        "Makefile",
        "cabal.project",
    ];

    let detected_file = priorities.iter().find(|&&f| entries.contains(f));

    if let Some(filename) = detected_file {
        match *filename {
            "configure" => init_build(PathBuf::from(&path), noinstall),
            "CMakeLists.txt" => compile_cmake(PathBuf::from(&path), noinstall),
            "Cargo.toml" => compile_rust(PathBuf::from(&path), noinstall),
            "Makefile" => build_make(PathBuf::from(&path), noinstall),
            "build.zig" => build_zig(PathBuf::from(&path), noinstall),
            "meson.build" => start_meson(PathBuf::from(&path), noinstall),
            "go.mod" => compile_go(PathBuf::from(&path), noinstall),
            "cabal.project" => compile_haskell(PathBuf::from(&path), noinstall),
            _ => unreachable!(),
        }
    } else {
        println!("No recognized build system found.");
    }
}
pub fn detect_clean(directory: String) {
    let mut path = PathBuf::new();
    if directory.is_empty() {
        println!("Path is empty, nothing to clean");
        path = env::current_dir().unwrap()
    } else {
        path.clear();
        path.push(&directory);
    }
    for got_file in fs::read_dir(&path).unwrap() {
        let entry = got_file.unwrap();
        let os_filename = entry.file_name();
        let filename = os_filename.into_string().unwrap();
        match filename.as_str() {
            "Cargo.toml" => crate::rust::clean(path.clone()),
            "CMakeLists.txt" => crate::cmake::clean(path.clone()),
            "build.zig" => crate::zig::clean(path.clone()),
            "Makefile" => crate::make::clean(path.clone()),
            "build.meson" => crate::meson::clean(path.clone()),
            "go.mod" => crate::go::clean(path.clone()),
            _ => {
                continue;
            }
        }
    }
}
///universal finder
pub fn find_files_because_the_user_is_too_lazy(directory: PathBuf) -> Vec<PathBuf> {
    let temp = directory.canonicalize().unwrap();

    let all_entries: Vec<PathBuf> = fs::read_dir(temp)
        .unwrap()
        .filter_map(|res| res.ok().map(|e| e.path()))
        .collect();

    find_executable_files(all_entries)
}

pub fn install_to_bin(executables: Vec<PathBuf>) -> UniversalResult<()> {
    let mut errors: Vec<String> = Vec::new();
    for binary in executables {
        let filename = binary.file_name().unwrap().to_str().unwrap().to_owned();
        let destination = format!("/usr/local/bin/{}", filename);

        let _clean = Command::new("sudo")
            .arg("rm")
            .arg(&destination)
            .arg("-f")
            .output()?
            .status;

        let status = Command::new("sudo")
            .arg("cp")
            .arg(&binary)
            .arg(&destination)
            .status()
            .expect("failed to execute process");

        if !status.success() {
            println!("Failed to copy binaries ");
            errors.push(binary.to_str().unwrap().to_string());
            continue;
        } else {
            if Command::new("sudo")
                .args(["chmod", "+x", &destination])
                .status()
                .is_ok()
            {
                println!(
                    "Copied {} to {}",
                    binary.to_str().unwrap(),
                    destination.green()
                );
            } else {
                println!(
                    "Failed to copy {} {}",
                    binary.to_str().unwrap(),
                    destination.green()
                );
                errors.push(binary.to_str().unwrap().to_string());
                continue;
            }
        }
        let last_item_binary = binary
            .to_str()
            .unwrap()
            .split("/")
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        let installed_absolute_path = format!("/usr/local/bin/{}", last_item_binary);
        let temp_binary: UninPackage = UninPackage {
            name: binary
                .to_str()
                .unwrap()
                .split('/')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string(),
            paths: vec![PathBuf::from(installed_absolute_path)],
            change_date: time_create(),
            updated: false,
        };
        registry_write(&temp_binary, true);
        println!("\n{}", temp_binary);
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Box::new(CustomError(errors)))
    }
}
pub fn sudo() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn find_executable_files(files: Vec<PathBuf>) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|file| {
            if file.is_dir() {
                return false;
            }

            if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
                let blacklist = ["so", "d", "rlib", "lock", "txt", "fingerprint"];
                if blacklist.contains(&ext) {
                    return false;
                }
            }
            if let Some(content) = file.file_name()
                && content.to_str().unwrap().contains(".so")
            {
                return false;
            }

            if matches!(file.file_name().and_then(|n| n.to_str()), Some(n) if n.starts_with('.')) {
                return false;
            }

            if is_elf_quiet(&file.to_string_lossy()) {
                let metadata = fs::metadata(file).unwrap();
                return metadata.permissions().mode() & 0o111 != 0;
            }
            false
        })
        .collect()
}

fn is_elf(path: &str) -> io::Result<bool> {
    let mut f = File::open(path)?;
    let mut buf = [0u8; 4];
    let n = f.read(&mut buf)?;
    Ok(n == 4 && buf == [0x7f, b'E', b'L', b'F'])
}
fn is_elf_quiet(path: &str) -> bool {
    is_elf(path).unwrap_or(false)
}
pub fn only_unique<T: Eq + Hash + Clone>(first: &[T], second: &[T]) -> Vec<T> {
    let first_set: HashSet<_> = first.iter().collect();
    second
        .iter()
        .filter(|item| !first_set.contains(item))
        .cloned()
        .collect()
}
