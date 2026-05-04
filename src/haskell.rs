use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Command;

pub fn compile_haskell(directory: PathBuf, noinstall: bool) {
	let config_file_path = PathBuf::from(format!("{}/project.cabal", directory.clone().to_str().unwrap()));
	let updater = Command::new("cabal")
		.arg("update")
		.current_dir(directory.clone())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn();

	let stdout = updater.unwrap().stdout.take().unwrap();
	let reader = BufReader::new(stdout);
	reader.lines().for_each(|line| {
		println!("{}", line.unwrap_or("".to_string()));
	});
	//i want stdout out of scope
	//and the reader as well
	//let's shadow them.

	let precompile_deps = Command::new("cabal")
		.args(["build", "--only-dependencies"])
		.current_dir(directory.clone())
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn();

	let stdout = precompile_deps.unwrap().stdout.take().unwrap();
	let reader = BufReader::new(stdout);

	for line in reader.lines().flatten() {
		if line.contains("Starting")
			|| line.contains("Building")
			|| line.contains("Configuring") {
			print!("\r\x1B[K{}", line);
		}
	}
}
