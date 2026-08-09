use colored::Colorize;
use duct::cmd;
use smallvec::SmallVec;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::exit;
use std::string::String;
use unin_bin::{UninPackage, registry_write, time_create};

pub fn compile_gleam(path: PathBuf, noinstall: bool) {
    let builder = cmd!("gleam", "export", "erlang-shipment")
        .unchecked()
        .dir(path.clone())
        .stderr_to_stdout();

    let reader = builder.reader().unwrap();
    let bufreader = std::io::BufReader::new(reader);

    for line in bufreader.lines() {
        let line = line.unwrap_or("".to_string());
        let line = line.trim();
        if line.contains("Downloading")
            || line.contains("Downloaded")
            || line.contains("Compiling")
            || line.contains("Compiled")
            || line.contains("Exported")
        {
            let mut sauce = line.split_whitespace().collect::<Vec<&str>>();
            let first_token = sauce.remove(0); // removes and returns first
            let mut first_string = vec![first_token.to_string().purple().to_string()];
            let mut colored_vec: Vec<String> =
                sauce.iter().map(|s| s.green().to_string()).collect();
            first_string.append(&mut colored_vec);
            println!("{}", first_string.join(" "));
        } else if line.contains("Error") || line.contains("error") || line.contains("failed") {
            println!("{}", line.red());
        }
    }

    if noinstall {
        println!("Skipping install. The shipment is to be found in ./build/erlang-shipment");
        return;
    }

    let shipment = format!("{}/build/erlang-shipment", path.to_str().unwrap());
    let path_to_shipment = PathBuf::from(shipment);

    let path = path.canonicalize().unwrap();
    let package_name = &path.to_str().unwrap().split("/").last().unwrap();
    let path_to_shipment_dest =
        PathBuf::from(format!("/usr/local/bin/.unin_gleam/{}", package_name));

    let copier = cmd!(
        "sudo",
        "cp",
        "-r",
        &path_to_shipment,
        &path_to_shipment_dest
    )
    .stderr_to_stdout()
    .dir(path.clone())
    .run();

    if copier.is_err() {
        println!("{}", "Failed to copy the shipment".red());
        exit(1)
    }

    let script = format!(
        "#!/usr/bin/bash\n/usr/local/bin/.unin_gleam/{}/entrypoint.sh run",
        package_name
    );
    let script_path = PathBuf::from(format!("/usr/local/bin/{}", package_name));

    let write_cmd = cmd!("echo", "-e", script)
        .pipe(cmd!("sudo", "tee", script_path.clone()))
        .dir(path.clone())
        .stdout_null()
        .stderr_null()
        .run();

    if write_cmd.is_err() {
        println!("{}", "Failed to write the runner script".red());
        exit(1)
    }

    let chmod_cmd = cmd!("sudo", "chmod", "+x", script_path.clone())
        .dir(path.clone())
        .stderr_to_stdout()
        .run();

    if chmod_cmd.is_err() {
        println!(
            "{}",
            "Failed to change the permissions of the runner script".red()
        );
        exit(1)
    }

    let mut all_neccessary_paths = SmallVec::<[PathBuf; 2]>::new();
    all_neccessary_paths.push(path_to_shipment_dest);
    all_neccessary_paths.push(script_path);

    let package: UninPackage = UninPackage {
        name: package_name.to_string(),
        paths: all_neccessary_paths.to_vec(),
        change_date: time_create(),
        updated: false,
    };

    registry_write(&package, true);
}
pub fn clean(path: PathBuf) {
    let gleam_cmd = cmd!("gleam", "clean")
        .unchecked()
        .dir(path.clone())
        .stderr_to_stdout()
        .run();

    if gleam_cmd.is_err() {
        println!("Failed to clean gleam project");
        exit(1)
    }
}
