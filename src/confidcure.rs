use crate::logging::log_to_file;
use crate::tools::find_files_because_the_user_is_too_lazy;
use crate::tools::only_unique;
use colored::Colorize;
use dialoguer::console::strip_ansi_codes;
use std::io;
use std::io::{stdout, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::exit;
use duct::cmd;
use unicode_truncate::UnicodeTruncateStr;
use unin_bin::{UninPackage, registry_write, return_registry_path, time_create};

pub(crate) fn init_build(path: PathBuf, noinstall: bool) {
    confihgure(path.clone()); //configure
    make(path.clone());
    install(path.clone(), noinstall);
    exit(0)
}

fn confihgure(path: PathBuf) {
    let configure_path = PathBuf::from(format!("{}/configure", path.to_str().unwrap()));

    let output = cmd!(&configure_path, "--prefix=/usr/local")
        .dir(&path)
        .stderr_to_stdout()
        .read()
        .expect("Failed to read stdout");


    let mut full_stdout = String::new();
    let mut stdout_has_error = false;
    for line in output.lines() {
        if line.contains("configure: error:") || line.contains("error:") || line.contains("failed") {
            stdout_has_error = true;
            full_stdout.push_str(&line);
            full_stdout.push('\n');
            continue;
        } else {
            full_stdout.push_str(&line);
            full_stdout.push('\n');

            println!("{}", line);
            io::stdout().flush().unwrap();
        }
    }
    print!("\r\x1B[K");
    stdout().flush().unwrap();
    log_to_file(path, "configure".to_string(), full_stdout.clone());
    println!();
    println!(
        "{}",
        "The configuration process has finished. The full output is available in ./latest-configure.txt.".yellow()
            .underline()
	);

    if stdout_has_error {
        println!("\nThe configuration process yielded an error. The full output will shown below");
        println!("{}", full_stdout.as_str().red());
    }
}

fn make(directory: PathBuf) {
    let cols = terminal_size::terminal_size()
        .map(|(width, _)| width.0 as usize - 10)
        .unwrap_or(80); //important ! columns

    let num_cpus = num_cpus::get();

    let make_process = cmd!("make", "-j", &num_cpus.to_string())
        .dir(&directory)
        .stderr_to_stdout();

    let stdout = make_process.reader().unwrap();

    let mut full_stdout = String::new();
    let mut stdout_has_error = false;

    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        if !line.contains("error:") && !line.contains("failed") {
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
    print!("\r\x1B[K");
    io::stdout().flush().unwrap();

    if stdout_has_error {
        println!("The make process yielded an error. The full output will shown below");
        println!("{}", full_stdout.as_str().red());
    }

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
        println!(
            "{}{} Do it yourself you lazy sloth!",
            "Skipping installation".yellow().underline(),
            " because the Makefile does not provide an install target.".yellow()
        );
        exit(0)
    }

    let registry_path = return_registry_path();
    let before_install: Vec<PathBuf> =
        find_files_because_the_user_is_too_lazy(registry_path.clone());

    let make_installer = cmd!("sudo", "make", "install")
        .stderr_to_stdout()
        .dir(&path);

    let stdout = make_installer.reader().unwrap();

    let mut full_stdout = String::new();
    let mut stdout_has_error = false;
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        if !line.contains("error:") && !line.contains("failed") {
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

    if stdout_has_error{
        println!("The make install process yielded an error. The full output will shown below");
        println!("{}", full_stdout.as_str().red());
        exit(0)
    }
    let after_install: Vec<PathBuf> = find_files_because_the_user_is_too_lazy(registry_path.clone());
    let binaries_installed: Vec<PathBuf> = only_unique(&before_install, &after_install);

    println!();
    println!(
        "{}",
        "Installation finished. The following binaries were installed, and will be written to the registry:".yellow().underline()
    );
    binaries_installed
        .iter()
        .for_each(|b| println!("{}", b.to_str().unwrap()));
    for package in binaries_installed.iter() {
        let package = UninPackage {
            name: package
                .to_str()
                .unwrap_or("")
                .split("/")
                .last()
                .unwrap_or("placeholder")
                .to_string(),
            paths: vec![package.to_owned()],
            change_date: time_create(),
            updated: false,
        };
        registry_write(&package, true)
    }
}
