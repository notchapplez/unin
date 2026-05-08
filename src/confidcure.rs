use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;
use colored::Colorize;
use crate::logging::log_to_file;

fn confihgure(path: PathBuf, noinstall: bool) {
    //prefix=/usr/local
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
		if !line.contains("configure: error:") || line.contains("error:")  || line.contains("failed") {
			print!("\r\x1B[K{}", line.clone().purple());
			full_stdout.push_str(&line.clone()); //append to full_stdout
		} else {
			stdout_has_error = true;
			full_stdout.push_str(&line.clone()); //append to full_stdout
			continue
		}
	}

	let stderr = configure_process.as_mut().unwrap().stderr.take().unwrap();
	let mut stderr_has_error: bool = false;
	let stderr_reader = BufReader::new(stderr);
	for line in stderr_reader.lines().map_while(Result::ok) {
		if !line.contains("configure: error:") || line.contains("error:") || line.contains("failed") {
			print!("\r\x1B[K{}", line.clone().yellow());
			full_stdout.push_str(&line.clone());
		} else {
			stderr_has_error = true;
			full_stdout.push_str(&line.clone());
			continue
		}
	}
	log_to_file(path.clone(), "configure".to_string(), full_stdout.clone());
	println!(
        "{}",
        "The configuration process has finished. The full output is available in {}.".yellow()
            .underline()
	);
	if stdout_has_error || stderr_has_error {
		println!("The configuration process yielded an error. The full output will shown below");
		println!("{}", full_stdout.as_str().red());
	}

	let _waiter = configure_process.as_mut().unwrap().wait().unwrap();
}

pub(crate) fn init_build(path: PathBuf, noinstall: bool) {
	confihgure(path, noinstall); //configure
}
fn make(directory: PathBuf, noinstall: bool) {
	let mut make_process = Command::new("make")
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.current_dir(&directory)
		.spawn();
	let stdout = make_process.as_mut().unwrap().stdout.take().unwrap();
	let mut full_stdout = String::new();
	let mut stdout_has_error = false;

	let stdout_reader = BufReader::new(stdout);
	for line in stdout_reader.lines().map_while(Result::ok) {
		if !line.contains("error:") || line.contains("failed") {
			print!("\r\x1B[K{}", line.clone().purple());
			full_stdout.push_str(&line.clone());
		} else {
			stdout_has_error = true;
			full_stdout.push_str(&line.clone());
			continue
		}
	}

	let stderr = make_process.as_mut().unwrap().stderr.take().unwrap();
	let mut stderr_has_error = false;
	let stderr_reader = BufReader::new(stderr);

	for line in stderr_reader.lines().map_while(Result::ok) {
		if line.contains("error:") || line.contains("failed") {
			stderr_has_error = true;
			full_stdout.push_str(&line.clone());
			continue
		} else {
			print!("\r\x1B[K{}", line.clone().yellow());
		}
	}
	if stdout_has_error || stderr_has_error {
		println!("The make process yielded an error. The full output will shown below");
		println!("{}", full_stdout.as_str().red());
	}

	let _waiter = make_process.as_mut().unwrap().wait().unwrap();
}
