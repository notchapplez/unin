use crate::logging::log_to_file;
use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use dialoguer::console::strip_ansi_codes;
use duct::cmd;
use path_absolutize::Absolutize;
use std::io::BufReader;
use std::{
    fs,
    io::{BufRead, Write},
    path::PathBuf,
    process::exit,
};
use terminal_size::terminal_size;
use unicode_truncate::UnicodeTruncateStr;
use unin_bin::return_registry_path;

pub fn build_make(directory: PathBuf, noinstall: bool) {
    let cols = terminal_size()
        .map(|(width, _)| width.0 as usize - 10)
        .unwrap_or(80usize);

    let dir: PathBuf = PathBuf::from("/usr/local/bin");
    let before_install_files = find_files_because_the_user_is_too_lazy(dir);

    let absolute_path = directory.absolutize().unwrap();
    let makefile_path = PathBuf::from(format!("{}/Makefile", absolute_path.to_str().unwrap()));
    println!(
        "Makefile path: {}",
        makefile_path.to_str().unwrap().yellow()
    );

    let num_cpus = num_cpus::get();

    let make_replacement = cmd!("make", "-j", &num_cpus.to_string())
        .dir(&directory)
        .stderr_to_stdout()
        .unchecked();

    let read = make_replacement.reader();
    let reader = BufReader::new(read.unwrap());
    let mut full_content = String::new();
    let mut has_error = false;

    for line in reader.lines().map_while(Result::ok) {
        let mut content = line;
        let raw_content = content.clone();
        if content.contains("CC")
            || content.contains("LD")
            || content.contains("LINK")
            || content.contains("AR")
            || content.contains("checking")
            || content.contains("CCLD")
            || content.contains("LDSHARED")
            || content.contains("RANLIB")
            || content.contains("Building")
            || content.contains("Built")
        {
            content = strip_ansi_codes(&content.to_string().to_owned()).to_string();
            content = content.trim().to_string();
            let mut content_vec = content //some weird processing
                .split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>();

            content_vec[0] = content_vec[0].blue().bold().to_string();
            content_vec[1..].iter_mut().for_each(|s| {
                *s = s.purple().bold().to_string();
            });

            content = content_vec.join(" ");
            content = content.unicode_truncate(cols).0.to_string();

            print!("\r\x1B[K{}", content.trim_end()); //do beautiful stuff
            std::io::stdout().flush().unwrap();

        } else if content.contains("error:")
            || content.contains("No targets.")
            || content.contains("make: ***")
        {
            has_error = true;
            full_content.push_str(&raw_content.clone());
            full_content.push('\n');
            continue;
        }
        full_content.push_str(&raw_content.clone());
        full_content.push('\n');
    }

    log_to_file(directory.clone(), "make".to_string(), full_content.clone());
    println!();
    println!("The make process has finished. The full output is available in ./latest-make.txt.");

    if has_error {
        println!("{}", "\nAn error occurred while compiling.".red().bold());
        println!("{}", "This might be due to a missing dependency or unset environment variables. Check README.md for exact building information.".red().bold());
        println!(
            "Full error: \n{}",
            full_content.trim_end().replace("error:", &"error".red())
        );
        exit(0)
    }

    if noinstall {
        println!();
        println!(
            "As make does not provide a reliable to expose binary locations, I can't tell you the paths to the binaries. Find them yourself, idk bro."
        );
        println!(
            "If you still see some file paths shown then that's a small scan, okay? Don't trust this completely."
        );
        let binaries = find_files_because_the_user_is_too_lazy(directory.clone());
        binaries
            .iter()
            .for_each(|b| println!("{}", b.to_str().unwrap()));
        exit(0) //process ends here, allocator frees memory
    }
    println!(
        "An installation using \"make install\" will be attempted. This will not work if the Makefile does not define an \"install\" rule."
    );

    let file_contents = fs::read_to_string(makefile_path.clone()).unwrap();
    let mut prefix_argument = String::new();
    let mut path_already_defined: bool = false;
    let mut has_install_rule: bool = false;

    for line in file_contents.lines() {
        if line.contains("/usr/local/") {
            path_already_defined = true;
            break;
        } else if line.contains(".PHONY : install") {
            has_install_rule = true;
            continue;
        } else {
            path_already_defined = false;
            continue;
        }
    }

    if !has_install_rule {
        println!("The Makefile does not define an \"install\" rule. Aborting.");
        exit(2);
    } else {
        println!("The Makefile defines an \"install\" rule. Continuing.");
    }

    if !path_already_defined {
        prefix_argument = "PREFIX=/usr/local".trim().to_string();
    } else {
        prefix_argument = prefix_argument.trim().to_string();
    }

    let registry_path = return_registry_path();
    let before_install = find_files_because_the_user_is_too_lazy(registry_path.clone());
    before_install
        .iter()
        .for_each(|b| println!("{}", b.to_str().unwrap()));

    let installation_process = cmd!("sudo", "make", prefix_argument, "install")
        .stderr_to_stdout()
        .dir(&directory)
        .unchecked();


    let output = installation_process.reader().unwrap();
    let reader = BufReader::new(output);
    let mut full_content = String::new();
    let mut has_error = false;
    for line in reader.lines() {
        match line {
            Ok(content) => {
                let raw_content = content.clone();
                let trimmed = content.trim();
                if trimmed.contains("error")
                    || trimmed.contains("make: ***")
                    || trimmed.contains("No targets.")
                {
                    has_error = true;
                    print!("\r\x1B[KError: {}", raw_content.red());
                    full_content.push_str(&raw_content);
                    full_content.push('\n');
                } else {
                    print!("\r\x1B[K{}", raw_content.purple());
                }
            }
            Err(e) => eprintln!("read error: {}", e),
        }
    }

    let after_install = find_files_because_the_user_is_too_lazy(registry_path.clone());
    let uniques = crate::tools::only_unique(&before_install_files, &after_install);

    install_to_bin(uniques).unwrap();

    if has_error {
        println!("{}", "\nAn error occurred while installing.".red().bold());
        println!(
            "Full error: \n{}",
            full_content.trim_end().replace("error:", &"error:".red())
        );
        exit(0)
    }
    println!("Installation finished successfully.");
    println!("You can now use the binaries in your PATH.");
}

pub fn clean(directory: PathBuf) {
    let cleaner = cmd!("make", "clean")
        .dir(&directory)
        .stderr_to_stdout()
        .run();
    let output = cleaner.as_ref().unwrap();
    println!("{}", String::from_utf8_lossy(&output.stdout));
    if cleaner.is_ok() {
        println!("Cleaned successfully.");
    } else {
        println!("Cleaning failed.");
        println!(
            "If you want, you can try to clean manually by running \"make clean\" in the project root."
        )
    }
}

