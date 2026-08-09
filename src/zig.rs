use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use duct::cmd;
use std::fs::File;
use std::io::{Read, Write};
use std::{io, path::PathBuf, process::exit};

pub fn build_zig(directory: PathBuf, noinstall: bool) {
    let child = cmd!("zig", "build", "-Doptimize=ReleaseFast")
        .dir(&directory)
        .stdout_to_stderr()
        .reader()
        .unwrap_or_else(|_| panic!("Couldn't create zig build process"));

    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut r = child;
        let mut buf = [0u8; 8 * 1024];
        loop {
            match r.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        //i have no idea why i used a channel here, but whatever
                        break;
                    }
                }

                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    break;
                }
            }
        }
    });
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut file = File::create("zig-out/zig-build-log.txt").unwrap();
    for chunk in rx {
        out.write_all(&chunk).unwrap();
        out.flush().unwrap();
        file.write_all(&chunk).unwrap();
    }

    file.flush().unwrap();
    let out_dir = PathBuf::from(format!("{}/zig-out/bin", directory.to_str().unwrap()));
    if noinstall {
        println!("{}", "Skipping installation of zig binaries".yellow());
        println!(
            "You can find the binaries in {}",
            out_dir.to_str().unwrap().yellow()
        );
        exit(0)
    }
    println!("Installing zig binariesoos");
    let executables = find_files_because_the_user_is_too_lazy(out_dir.clone());
    let _ = install_to_bin(executables);

    exit(0)
}
pub fn clean(directory: PathBuf) {
    let target_dir = PathBuf::from(format!("{}/.zig-cache", directory.to_str().unwrap()));
    if !target_dir.exists() {
        println!("Zig build directory doesn't exist, nothing to do.");
        exit(0)
    }

    let cleaning = std::fs::remove_dir_all(target_dir);

    if cleaning.is_err() {
        println!("Couldn't clean the zig build directory.");
    } else {
        println!("Zig build directory cleaned.");
    }
}
