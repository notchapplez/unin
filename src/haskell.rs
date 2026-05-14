use crate::logging::log_to_file;
use colored::Colorize;
use duct::cmd;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;

pub fn compile_haskell(directory: PathBuf, _noinstall: bool) {
    let updater = Command::new("cabal")
        .arg("update")
        .current_dir(directory.clone())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let waiter = updater.unwrap().wait_with_output().unwrap();
    if !waiter.status.success() {
        println!("cabal update failed.")
    }

    let mut precompile = cmd!("cabal", "build")
        .dir(directory.clone())
        .stderr_to_stdout()
        .unchecked();

    let stdout = precompile.reader().unwrap();
    let reader = BufReader::new(stdout);
    let mut content = String::new();
    let mut has_error: bool = false;

    for line in reader.lines().map_while(Result::ok) {
        if line.contains("Starting")
            || line.contains("Building")
            || line.contains("Configuring")
            || line.contains("Downloaded")
            || line.contains("Downloading")
            || line.contains("Resolving")
            || line.contains("Completed")
            || line.contains("Installing")
        {
            print!("\r\x1B[K{}", line);
            io::stdout().flush().unwrap();
            content.push_str(format!("{}\n", line).as_str());
            continue;
        } else if line.contains("Error:") || line.contains("error:") || line.contains("Failed") {
            has_error = true;
            content.push_str(format!("{}\n", line).as_str());
            continue;
        }
        content.push_str(format!("{}\n", line).as_str());
        continue;
    }
    if has_error {
        println!(
            "{}",
            "Compilation failed. Output will be shown below.".yellow()
        );
        println!("{}", content.red());
    }

    let _logger = log_to_file(directory.clone(), "build".to_string(), content);

    exit(0)
}
pub fn clean(directory: PathBuf) {
    let process = cmd!("cabal", "clean")
        .dir(directory.clone())
        .stderr_to_stdout()
        .run();

    if process.is_err() {
        println!("cabal clean failed.");
        exit(1)
    } else {
        println!("cabal clean successful.");
        exit(0)
    }
}
