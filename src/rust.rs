use crate::tools::{
    //imports the functions from the tools module
    find_files_because_the_user_is_too_lazy,
    install_to_bin,
};
use colored::Colorize;
use dialoguer::console::strip_ansi_codes;
use path_absolutize::Absolutize;
use std::{
    env, fs,
    io::{BufRead, Write},
    path::PathBuf,
    process::{Command, Stdio, exit},
};

pub fn compile_rust(directory: PathBuf, noinstall: bool) {
    let mut full_path = String::new();
    //defines the function
    if directory == *"." {
        let directory = env::current_dir().unwrap();
        let _ = full_path; //to satisfy clippy
        full_path = String::from(directory.absolutize().unwrap().to_str().unwrap());
    } else {
        let _ = full_path; //to satisfy clippy
        full_path = String::from(directory.absolutize().unwrap().to_str().unwrap());
    }

    println!("Now compiling {}", full_path.yellow().underline()); //prints a start message

    let mut child = Command::new("cargo")
        .args(["build", "--release", "--color", "always"])
        .current_dir(&directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to compile");

    let mut full_out = String::new();
    let stderr = child.stderr.take().unwrap();
    let reader = std::io::BufReader::new(stderr);
    let mut has_error: bool = false;
    for line in reader.lines() {
        match line {
            Ok(content) => {
                let checking_content = strip_ansi_codes(content.to_owned().as_str()).to_string();
                //if the line is fine
                if checking_content.contains("Compiling") {
                    let appropriate_variable_name_here = checking_content.trim().to_string();
                    print!("\r\x1B[K{}", appropriate_variable_name_here.bold().purple());
                    std::io::stdout().flush().unwrap();
                } else if checking_content.contains("error:") {
                    has_error = true;
                }
                full_out.push_str(format!("{}\n", content).as_str());
            }
            Err(e) => println!("Error reading stdout: {}", e), //if there is an error, print the error
        }
    }
    if has_error {
        println!(
            "\r\x1B[K{}",
            "Compilation failed. Output will be shown below.".red()
        );
        println!("{}", full_out);
        exit(0)
    }

    println!();
    let releases_folder = PathBuf::from(format!("{}/target/release", directory.to_str().unwrap()));
    if !releases_folder.exists() {
        fs::create_dir_all(releases_folder.clone()).expect("Failed to create directory");
    }

    let binaries = find_files_because_the_user_is_too_lazy(releases_folder); //finds the binaries in the path

    if noinstall {
        //skips installation if user
        println!("Skipping the installation as --noinstall was given.");
        return;
    }
    let _ = install_to_bin(binaries.clone()); //installs the dropped binaries AND REGISTERS THEM
    println!(
        //prints an end message
        "{}",
        "The compilation and installation is finished. No error reported.".green()
    );
    exit(0)
}
pub fn clean(directory: PathBuf) {
    let clean_process_cargo = Command::new("cargo")
        .args(["clean"])
        .current_dir(&directory)
        .output();
    println!(
        "{}",
        String::from_utf8_lossy(&clean_process_cargo.unwrap().stderr).trim()
    );
}
