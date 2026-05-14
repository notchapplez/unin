use std::path::PathBuf;
use duct::cmd;

pub fn compile_gleam(path: PathBuf, noinstall: bool) {
	let builder = cmd!("gleam", "export", "erlang-shipment")
		.unchecked()
		.dir(path.clone())
		.stderr_to_stdout();

	let shipment = format!("{}/build/erlang-shipment", path.to_str().unwrap());
	//proof of concept: move the shipment to /usr/local/bin/.unin_gleam/, then create the runner in /usr/local/bin/<file> with the binary name: example content:
	// #!/usr/bin/bash
	// /usr/local/bin/.unin_gleam/gleamy_test/entrypoint.sh
	//rn working with 'runner' command
	todo!()
}
pub fn clean(path: PathBuf) {
	todo!()
}