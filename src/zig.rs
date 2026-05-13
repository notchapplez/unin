use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use std::{io, io::BufRead, path::PathBuf, process::{Command, exit}};
use std::io::{Read, Write};
use duct::cmd;
use crate::logging::log_to_file;

pub fn build_zig(directory: PathBuf, noinstall: bool)   {
    let mut zig_build_process = cmd!("zig", "build", "-Doptimize=ReleaseFast")
        .dir(&directory)
        .stdout_to_stderr() // Move any rare stdout into the same stream
        .reader()
        .unwrap();

    let mut buf = [0u8; 8 * 1024];
    let stdout_handle = io::stdout();
    let mut terminal = stdout_handle.lock();

    loop {
        let n = zig_build_process.read(&mut buf).unwrap_or(0);
        if n == 0 { break }
        terminal.write_all(&buf[..n]).unwrap();
        terminal.flush().unwrap();

    }
    terminal.flush().unwrap();

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
