use colored::Colorize;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;
use std::sync::mpsc::{Receiver, Sender};

pub fn compile_haskell(directory: PathBuf, noinstall: bool) {
    let config_file_path = PathBuf::from(format!(
        "{}/project.cabal",
        directory.clone().to_str().unwrap()
    ));
    let mut updater = Command::new("cabal")
        .arg("update")
        .current_dir(directory.clone())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let waiter = updater.unwrap().wait_with_output().unwrap();
    if !waiter.status.success() {
        println!("cabal update failed.")
    }

    let mut precompile_deps = Command::new("cabal")
        .args(["build", "--only-dependencies"])
        .current_dir(directory.clone())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let (transmitter, receiver): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();

    let stdout = precompile_deps.as_mut().unwrap().stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines().map_while(Result::ok) {
        if line.contains("Starting")
            || line.contains("Building")
            || line.contains("Configuring")
            || line.contains("Downloaded")
            || line.contains("Downloading")
            || line.contains("Resolving")
            || line.contains("Completed")
        {
            print!("\r\x1B[K{}", line);
            io::stdout().flush().unwrap();
            transmitter.send(line.to_string()).unwrap();
            continue;
        }
        print!("\r\x1B[K{}", "Debug".underline());
        io::stdout().flush().unwrap();
        transmitter.send(line.to_string()).unwrap();
    }

    let stderr = precompile_deps.as_mut().unwrap().stderr.take().unwrap();
    let stderr_reader = BufReader::new(stderr);
    let mut stderr_has_error = false;

    for line in stderr_reader.lines().map_while(Result::ok) {
        if line.contains("error") {
            println!(
                "{}",
                "An error occurred. The full output will be shown below".red()
            );
            stderr_has_error = true;
        }
    }

    if stderr_has_error {
        println!("Full error output:");
        receiver.iter().for_each(|line| println!("{}", line))
    }
    exit(0)
}
