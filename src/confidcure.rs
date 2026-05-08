use crate::logging::log_to_file;
use colored::Colorize;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use dialoguer::console::strip_ansi_codes;
use unicode_truncate::UnicodeTruncateStr;
use unin_bin::{registry_write, return_registry_path, time_create, UninPackage};
use crate::tools::find_files_because_the_user_is_too_lazy;
use crate::tools::only_unique;

pub(crate) fn init_build(path: PathBuf, noinstall: bool) {
    confihgure(path.clone()); //configure
    make(path.clone());
    install(path.clone(), noinstall);
    exit(0)
}

fn confihgure(path: PathBuf) {
    let mut configure_process = Command::new("sh")
        .args(["./configure", "--prefix=/usr/local"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(&path)
        .spawn();

    let stdout = configure_process.as_mut().unwrap().stdout.take().unwrap();
    let mut full_stdout = String::new();
    let mut stdout_has_error: bool = false;
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        if !line.contains("configure: error:") || line.contains("error:") || line.contains("failed")
        {
            print!("\r\x1B[K{}", line.clone().purple());
            io::stdout().flush().unwrap();
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
            //append to full_stdout
        } else {
            stdout_has_error = true;
            full_stdout.push_str(&line.clone()); //appe
            full_stdout.push('\n');
            //append to full_stdout
            continue;
        }
    }

    let stderr = configure_process.as_mut().unwrap().stderr.take().unwrap();
    let mut stderr_has_error: bool = false;
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines().map_while(Result::ok) {
        if !line.contains("configure: error:") || line.contains("error:") || line.contains("failed")
        {
            print!("\r\x1B[K{}", line.clone().yellow());
            io::stdout().flush().unwrap();
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
        } else {
            stderr_has_error = true;
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
            continue;
        }
    }
    log_to_file(path.clone(), "configure".to_string(), full_stdout.clone());
    println!();
    println!(
        "{}",
        "The configuration process has finished. The full output is available in ./latest-configure.txt.".yellow()
            .underline()
	);

    if stdout_has_error || stderr_has_error {
        println!("The configuration process yielded an error. The full output will shown below");
        println!("{}", full_stdout.as_str().red());
    }

    let _waiter = configure_process.as_mut().unwrap().wait().unwrap();
}

fn make(directory: PathBuf) {

    let cols = terminal_size::terminal_size().map(|(width, _)| width.0 as usize - 10).unwrap_or(80); //important ! columns

    let num_cpus = num_cpus::get();
    let mut make_process = Command::new("make")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .args(["-j", &num_cpus.to_string()])
        .current_dir(&directory)
        .spawn();

    let stdout = make_process.as_mut().unwrap().stdout.take().unwrap();
    let mut full_stdout = String::new();
    let mut stdout_has_error = false;

    let stdout_reader = BufReader::new(stdout);
	println!("BufReader init");
    for line in stdout_reader.lines().map_while(Result::ok) {
        if !line.contains("error:") || line.contains("failed") {
            let line = strip_ansi_codes(&line);
            let truncated = line.unicode_truncate(cols).0;
            print!("\r\x1B[K{}", truncated.purple());
			io::stdout().flush().unwrap();
            full_stdout.push_str(&line.clone());
			full_stdout.push('\n');
        } else {
            stdout_has_error = true;
            full_stdout.push_str(&line.clone());
			full_stdout.push('\n');
            continue;
        }
    }
	println!("BufReader done");

    let stderr = make_process.as_mut().unwrap().stderr.take().unwrap();
    let mut stderr_has_error = false;
    let stderr_reader = BufReader::new(stderr);

    for line in stderr_reader.lines().map_while(Result::ok) {
        if line.contains("error:") || line.contains("failed") {
            stderr_has_error = true;
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
            continue;
        } else {
            print!("\r\x1B[K{}", line);
			io::stdout().flush().unwrap();
			full_stdout.push_str(&line.clone());
			full_stdout.push('\n');
        }
    }
    if stdout_has_error || stderr_has_error {
        println!("The make process yielded an error. The full output will shown below");
        println!("{}", full_stdout.as_str().red());
    }

    let _waiter = make_process.as_mut().unwrap().wait().unwrap();
}

fn install(path: PathBuf, noinstall: bool) {
    if noinstall {
        println!(
            "{}{}, you can find the binaries somewhere idk",
            "Skipping installation".yellow().underline(),
            " because of the --noinstall flag.".yellow()
        );
        exit(0)
    } /* else */
    let provides_install: fn(PathBuf) -> bool = move |path: PathBuf| {
        let read_content = std::fs::read_to_string(path).unwrap();

        read_content.contains("install:") //already returns for me!!!
    };
    if !provides_install(path.clone()) {
        println!("{}{} Do it yourself you lazy sloth!", "Skipping installation".yellow().underline(), " because the Makefile does not provide an install target.".yellow());
        exit(0)
    } //todo!()

    let registry_path = return_registry_path();
    let before_install: Vec<PathBuf> = find_files_because_the_user_is_too_lazy(registry_path.clone());

    let mut make_install_process = Command::new("sudo")
        .args(["make", "install"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(&path)
        .spawn();

    let stdout = make_install_process.as_mut().unwrap().stdout.take().unwrap();
    let mut full_stdout = String::new();
    let mut stdout_has_error = false;
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        if !line.contains("error:") || line.contains("failed") {
            print!("\r\x1B[K{}", line.clone().purple());
            io::stdout().flush().unwrap(); //important: flush stdout as you write to it, obviously
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
        } else {
            stdout_has_error = true;
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
            continue;
        }
    }
    let stderr = make_install_process.as_mut().unwrap().stderr.take().unwrap();
    let stderr_reader = BufReader::new(stderr);
    let mut stderr_has_error = false;
    for line in stderr_reader.lines().map_while(Result::ok) {
        if !line.contains("error:") || line.contains("fuck") {
            print!("\r\x1B[K{}", line.clone().yellow());
            io::stdout().flush().unwrap();
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
        } else {
            stderr_has_error = true;
            full_stdout.push_str(&line.clone());
            full_stdout.push('\n');
            continue;
        }
    }
    make_install_process.as_mut().unwrap().wait().unwrap();

    if stdout_has_error || stderr_has_error {
        println!("The make install process yielded an error. The full output will shown below");
        println!("{}", full_stdout.as_str().red());
        exit(0)
    }
    let after_install: Vec<PathBuf> = find_files_because_the_user_is_too_lazy(registry_path.clone());
    let binaries_installed: Vec<PathBuf> = only_unique(&before_install, &after_install);

    println!();
    println!(
        "{}",
        "Installation finished. The following binaries were installed:".yellow().underline()
    );
    binaries_installed.iter().for_each(|b| println!("{}", b.to_str().unwrap()));
    for package in binaries_installed.iter() {
        let package = UninPackage {
            name: package.to_str().unwrap_or("").split("/").last().unwrap_or("placeholder").to_string(),
            paths: vec![package.to_owned()],
            change_date: time_create(),
            updated: false,
        };
        registry_write(&package, true)
    }

}

