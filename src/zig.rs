use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use std::{
    io::BufRead,
    path::PathBuf,
    process::{Command, exit},
};

pub fn build_zig(directory: PathBuf, noinstall: bool) {
    let mut zig_build_process = Command::new("zig")
        .current_dir(&directory)
        .arg("build")
        .arg("-Doptimize=ReleaseFast")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Couldn't start the zig build process.");

    let stderr = zig_build_process.stderr.take().unwrap();
    let reader = std::io::BufReader::new(stderr);
    for line in reader.lines() {
        print!("\r\x1B[K{}", line.unwrap_or("".to_string()).bold().purple());
    }

    let _waiter = &zig_build_process
        .wait()
        .expect("Couldn't wait for the zig build process.");

    let out_dir = PathBuf::from(format!("{}/zig-out/bin", directory.to_str().unwrap()));
    if noinstall {
        println!("{}", "Skipping installation of zig binaries".yellow());
        println!(
            "You can find the binaries in {}",
            out_dir.to_str().unwrap().yellow()
        );
        exit(0)
    }
    println!("Installing zig binaries");
    let executables = find_files_because_the_user_is_too_lazy(out_dir.clone());
    let _ = install_to_bin(executables);

    exit(0)
}
pub fn clean(directory: PathBuf) {
    let target_dir = PathBuf::from(format!("{}/zig-out", directory.to_str().unwrap()));
    let cleaning = std::fs::remove_dir_all(target_dir);
    if cleaning.is_err() {
        println!("Couldn't clean the zig build directory.");
    } else {
        println!("Zig build directory cleaned.");
    }
}
