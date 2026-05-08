//unPack ver 0.0.1

use colored::Colorize;
use core::clone::Clone;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display};
use std::fs::{self, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use time::{OffsetDateTime, PrimitiveDateTime};

#[derive(Serialize, Deserialize, PartialEq)]
pub struct UninPackage {
    pub name: String,
    pub paths: Vec<PathBuf>,
    pub change_date: String,
    pub updated: bool,
}

impl Display for UninPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\tName: {}\n\tPaths: {:?}\n\tChange Date: {}\n",
            self.name, self.paths, self.change_date
        )
    }
}
impl Clone for UninPackage {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            paths: self.paths.clone(),
            change_date: self.change_date.clone(),
            updated: self.updated,
        }
    }
}
impl UninPackage {
    pub fn new(name: String) -> Self {
        UninPackage {
            name,
            paths: Vec::new(),
            change_date: "1970-01-01 00:00:00.0".to_string(),
            updated: false,
        }
    }
}
pub struct DebuggableOptionUninPackage(pub Option<UninPackage>);

impl Debug for DebuggableOptionUninPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(p) => write!(f, "{}", p),
            None => write!(f, "None"),
        }
    }
}

pub fn get_registry() {
    if !registry_exists() {
        println!("{}", "Registry couldn't be found nor created. Consider creating it manually at ~/.unin/registry/registry.json".red());
    }
    let home = std::env::var("HOME").unwrap();
    let registry_path = PathBuf::from(format!("{}/.unin/registry/", home));
    let registry_file = PathBuf::from(format!("{}/registry.json", registry_path.to_str().unwrap()));
    let data = fs::read_to_string(registry_file).unwrap();
    let packages: Vec<UninPackage> =
        serde_json::from_str(&data).unwrap_or_else(|e| panic!("Shit happened: {:?}", e));
    for p in packages {
        println!("\n{}", p);
    }
}
pub fn time_create() -> String {
    let now_utc = OffsetDateTime::now_utc();
    let primitive_now_utc = PrimitiveDateTime::new(
        now_utc.date(),
        now_utc
            .time()
            .truncate_to_second()
            .replace_nanosecond(0)
            .unwrap(),
    );
    PrimitiveDateTime::to_string(&primitive_now_utc)
}

pub fn time_read() -> PrimitiveDateTime {
    let registry_path = PathBuf::from(format!(
        "{}/.unin/registry/",
        std::env::var("HOME").unwrap_or_else(|_| "/root/".to_string())
    ));
    let registry_file = PathBuf::from(format!("{}/registry.json", registry_path.to_str().unwrap()));
    let data = fs::read_to_string(registry_file).unwrap();
    let p: UninPackage =
        serde_json::from_str(&data).unwrap_or_else(|e| panic!("Shit happened: {:?}", e));
    let time_string: String = p.change_date.clone();
    let x: PrimitiveDateTime =
        PrimitiveDateTime::parse(&time_string, &time::format_description::well_known::Rfc3339)
            .unwrap();
    println!("{}", x);
    x
}

pub fn registry_write(package: &UninPackage, print_console: bool) {
    let registry_path = format!(
        "{}/.unin/registry/registry.json",
        std::env::var("HOME").unwrap()
    );
    let registry_dir = Path::new(&registry_path).parent().unwrap();

    let _ = create_dir_all(registry_dir);

    let existing_content = fs::read_to_string(&registry_path).unwrap_or_else(|_| String::new());
    let mut is_new = true;
    for line in existing_content.lines() {
        if line.contains(package.name.trim()) {
            is_new = false;
            break;
        }
    }

    let mut packages: Vec<UninPackage> =
        serde_json::from_str(&existing_content).unwrap_or_else(|_| Vec::new());

    let package_name = package.name.clone();

    if let Some(pos) = packages.iter().position(|p| p.name == package.name) {
        packages[pos].updated = true;
        packages[pos].change_date = time_create();
        packages[pos].paths = package.paths.clone();
    } else {
        packages.push((*package).clone())
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&registry_path)
        .unwrap();

    serde_json::to_writer(&mut file, &packages).unwrap();
    file.flush().unwrap();

    if is_new && print_console {
        println!(
            "Registry for entry {} created successfully!",
            package_name.green().underline()
        );
    } else if !is_new && print_console {
        println!(
            "Registry for entry {} updated successfully!",
            package_name.green().underline()
        );
    }
}
pub fn registry_get_package(package_name: String) -> Option<UninPackage> {
    if !registry_exists() {
        println!("{}", "Registry couldn't be found nor created. Consider creating it manually at ~/.unin/registry/registry.json".red());
    }
    let home = std::env::var("HOME").unwrap();
    let registry_path = PathBuf::from(format!("{}/.unin/registry/", home));
    let registry_file = PathBuf::from(format!("{}/registry.json", registry_path.to_str().unwrap()));
    let data = fs::read_to_string(registry_file).unwrap();
    let packages: Vec<UninPackage> =
        serde_json::from_str(&data).unwrap_or_else(|e| panic!("Shit happened: {:?}", e));

    packages.into_iter().find(|p| p.name == package_name)
}
pub fn registry_exists() -> bool {
    let registry_path = PathBuf::from(format!(
        "{}/.unin/registry/",
        std::env::var("HOME").unwrap()
    ));
    let registry_file = PathBuf::from(format!("{}/registry.json", registry_path.to_str().unwrap()));
    if !registry_file.exists() {
        if !create_dir_all(&registry_path).is_ok() {
            // Create the DIRECTORY, not the file path
            println!("Failed to create registry for unPack ver 0.0.1");
            false //the directory didn't get created, so it doesn't exist.
        } else {
            // Create the empty registry file
            if let Ok(mut file) = fs::File::create(&registry_file) {
                let _ = file.write_all(b"[]"); // Write empty JSON array
                true //it exists now
            } else {
                println!("Failed to create registry file");
                false //It didn't get created, so it doesn't exist.
            }
        }
    } else {
        true //It existed and it will exist forever.
    }
}
pub fn registry_uninstall(package_name: String) {
    let home = std::env::var("HOME").unwrap();
    let registry_path: PathBuf = PathBuf::from(format!("{}/.unin/registry/registry.json", home));
    let registry_json: Value =
        serde_json::from_str(&fs::read_to_string(registry_path.clone()).unwrap()).unwrap();
    let mut packages: Vec<UninPackage> = serde_json::from_value(registry_json).unwrap();

    let pkg = match packages.iter().find(|p| p.name == package_name) {
        Some(p) => p,
        None => {
            eprintln!("No package named '{}' in registry", package_name);
            return;
        }
    };

    println!("\n{}", pkg);

    let confirmation = dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to delete this application?")
        .interact()
        .unwrap();
    if !confirmation {
        return;
    }

    // Work on a mutable copy for removal
    let mut remaining = packages.clone();

    let path = match pkg.paths.first() {
        Some(p) => p,
        None => {
            eprintln!("Package '{}' has no installed paths", pkg.name);
            let ask = dialoguer::Confirm::new()
                .with_prompt("Do you want to delete the entry anyway?")
                .interact()
                .unwrap();
            if ask {
                remaining.retain(|p| p.name != pkg.name);
                let _ = fs::write(registry_path, serde_json::to_string(&remaining).unwrap());
                println!("Registry entry for {} deleted", package_name);
            }
            return;
        }
    };

    let path_str = path.to_str().unwrap_or_else(|| {
        eprintln!("Path is not valid UTF-8");
        std::process::exit(1);
    });

    let delete_status = std::process::Command::new("sudo")
        .arg("rm")
        .arg("-f")
        .arg(path_str)
        .output()
        .unwrap_or_else(|e| panic!("Failed to delete the file: {}", e))
        .status;

    if !delete_status.success() {
        println!("Failed to delete the file for {}", pkg.name);
        let confirmation = dialoguer::Confirm::new()
            .with_prompt("Do you want to delete the registry entry anyway?")
            .interact()
            .unwrap();
        if confirmation {
            remaining.retain(|p| p.name != package_name);
            let _ = fs::write(registry_path, serde_json::to_string(&remaining).unwrap());
            println!("Registry entry for {} deleted", package_name);
        } else {
            println!("Aborting");
        }
    } else {
        packages.retain(|p| p.name != package_name);
        let _ = fs::write(registry_path, serde_json::to_string(&packages).unwrap());
        println!("Registry entry for {} deleted", package_name);
    }
}

pub fn temp_test() {
    let _x: UninPackage = UninPackage {
        name: String::from("dev"),
        paths: vec![PathBuf::from("/root/ad"), PathBuf::from("/root/da")],
        change_date: time_create(),
        updated: false,
    };
    get_registry();
}
pub fn return_registry_path() -> PathBuf {
    PathBuf::from(format!(
        "{}/.unin/registry/registry.json",
        std::env::var("HOME").unwrap()
    ))
}

pub fn update_check_registry() {
    let registry_path = PathBuf::from(format!(
        "{}/.unin/registry/registry.json",
        std::env::var("HOME").unwrap()
    ));
    if !registry_exists() {
        println!("{}", "Registry couldn't be found nor created. Consider creating it manually at ~/.unin/registry/registry.json".red());
    }
    let contents =
        fs::read_to_string(registry_path).expect("Something went wrong reading the file");
    let packages: Vec<UninPackage> = serde_json::from_str(&contents).unwrap();
    for package in packages {
        let mut new_paths: Vec<String> = Vec::new();
        package.paths.iter().for_each(|path| {
            if path.exists() {
                new_paths.push(path.to_str().unwrap().to_string());
            } else {
                println!(
                    "{}",
                    "The binaries of this registry entry are missing. Do you want to remove it?"
                        .red()
                );
                registry_uninstall(package.name.clone())
            }
        });

        if package.paths.is_empty() {
            let confirmation = dialoguer::Confirm::new()
                .with_prompt(format!(
                    "{}",
                    "The binaries of this registry entry are missing. Do you want to remove it?"
                        .red()
                ))
                .interact()
                .unwrap();

            if !confirmation {
                exit(0);
                unreachable!()
            }
            //on case of confirmation also unreachable!()
            registry_uninstall(package.name.clone());
        }

        let new_package = UninPackage {
            name: package.name.clone(),
            paths: new_paths.iter().map(PathBuf::from).collect(),
            change_date: package.change_date.clone(),
            updated: package.updated,
        };
        registry_write(&new_package, false);
    }
}
