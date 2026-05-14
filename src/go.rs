use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use duct::cmd;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::exit;
use std::thread;
use std::{fs, io};

pub fn compile_go(directory: PathBuf, noinstall: bool) {
    let cmdchild = cmd!("go", "build", "-o", "unin_built_temp/")
        .dir(&directory)
        .stderr_to_stdout()
        .unchecked();

    let merged_out = cmdchild.reader().unwrap();
    let reader = BufReader::new(merged_out);
    let mut has_error = false;
    let mut merged_out = String::new();
    for mut line in reader.lines() {
        if !line.as_mut().unwrap().contains("error") && !line.as_mut().unwrap().contains("failed") {
            print!("\r\x1B[K{}", line.as_mut().unwrap().purple());
            has_error = true;
            merged_out.push_str(&line.unwrap());
            merged_out.push('\n');
            io::stdout().flush().unwrap();
        } else {
            has_error = true;
            merged_out.push_str(line.as_mut().unwrap_or_else(|e| panic!("panic: {}", e)));
            merged_out.push('\n');
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    if has_error {
        print!("\r\x1B[K");
        println!(
            "{}",
            "The go build process yielded an error. The full output will shown below.".yellow()
        );
        println!("{}", merged_out.as_str().red());
        exit(1)
    }

    let test = PathBuf::from(format!("{}/unin_built_temp/", directory.to_str().unwrap()));
    if noinstall {
        println!(
            "{}",
            "Skipping installation. Binaries can be found in unin_built_temp/"
                .yellow()
                .underline()
        );
        println!();
        println!("I found some files, here they are:");
        find_files_because_the_user_is_too_lazy(test)
            .iter()
            .for_each(|x| println!("{}", x.to_str().unwrap()));
        exit(0)
    }

    let installer = install_to_bin(find_files_because_the_user_is_too_lazy(test));
    if installer.is_err() {
        println!("Failed to install binaries");
    }
}
pub fn clean(directory: PathBuf) {
    let build_dir = PathBuf::from(format!("{}/unin_built_temp/", directory.to_str().unwrap()));
    if !build_dir.exists() {
        println!("Nothing to do");
        exit(0);
    }
    fs::remove_dir_all(build_dir).expect("The cleaner fucked up. i don't know why.")
}
