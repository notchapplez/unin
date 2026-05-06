use crate::logging::log_to_file;
use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use rand::RngExt;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};
use unin_bin::{UninPackage, registry_write, time_create};

pub fn start_meson(directory: PathBuf, noinstall: bool) {
    let mut setup = Command::new("meson")
        .args(&["setup", "build"]) //build is the path to the build directory!
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(directory.clone())
        .spawn();

    let stdout = setup.as_mut().unwrap().stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut full_content = String::new();
    let mut has_error = false;
    for line in reader.lines() {
        if let Ok(content) = line {
            let raw_content = content.clone();
            let shc = content.clone();
            if shc.contains("ERROR") {
                has_error = true;
                print!("\r\x1B[K{}", shc.red().underline().bold());
                std::io::stdout().flush().unwrap();
                full_content.push_str(format!("{}\n", &raw_content.clone()).as_str());
                continue;
            }
            print!("\r\x1B[K{}", content.purple().bold());
            std::io::stdout().flush().unwrap();
            let mut rng = rand::rng();
            let random_delay: u64 = rng.random_range(30..=60);
            std::thread::sleep(std::time::Duration::from_millis(random_delay));
            full_content.push_str(format!("{}\n", &raw_content.clone()).as_str());
        }
    }
    let waiter = setup.unwrap().wait().unwrap();

    if !waiter.success() {
        println!("\nConfiguration failed.");
    }
    if has_error {
        println!(
            "{} The full error will be printed here.\n",
            "\nAn error occurred while configuring the project."
                .red()
                .bold()
        );
        println!(
            "{}",
            full_content.trim_end().replace("error:", &"error:".red())
        );
        exit(1);
    }
    let write_log = log_to_file(directory.clone(), "meson".to_string(), full_content);
    println!(
        "\nLog for the unin install step \"meson\" can be found here: {}",
        write_log
    );
    drop(write_log);

    let cpu_cores = num_cpus::get();
    let mut child = Command::new("ninja")
        .args(&["-C", "build", "-j", &cpu_cores.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(directory.clone())
        .spawn();

    let childed = child.as_mut().unwrap().stdout.take().unwrap();

    let stdout = childed;
    let reader = BufReader::new(stdout);
    let mut content_full = String::new();
    let mut has_error = false;
    for line in reader.lines() {
        if let Ok(content) = line {
            print!("\r\x1B[K{}", content.purple().bold());
            std::io::stdout().flush().unwrap();
            if content.contains("error:") {
                has_error = true;
                content_full.push_str(content.as_str());
                continue;
            }
            content_full.push_str(content.as_str());
            continue;
        }
    }
    let _waiter = child.unwrap().wait().unwrap();
    if has_error {
        println!(
            "Build using ninja failed. If you want, I can show you the output of the build process. I don't care if you want to, here it is:"
        );
        content_full
            .split("\n")
            .for_each(|line| println!("{}", line));
    }
    let log = log_to_file(directory.clone(), "ninja".to_string(), content_full);
    println!(
        "\nLog for unin step \"ninja\" build can be found here: {}",
        log
    );
    drop(log);

    if noinstall {
        println!("Build finished successfully.");
        println!("I have no idea where the binaries are. They are somewhere, go find them")
    }
    let file_path: String = String::from(
        format!(
            "{}/build/meson-private/install.dat",
            directory.to_str().unwrap()
        )
        .as_str(),
    );
    let exists = fs::metadata(file_path).is_ok();
    if !exists {
        println!(
            "\nThe project does not provide an \"install\" rule. This means that I cannot install the binaries. You can still find them somewhere in build/"
        );
        exit(1);
    }

    let mut installer = Command::new("sudo")
        .args(["ninja", "-C", "build", "install"])
        .current_dir(directory.clone())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let stdout = installer.as_mut().unwrap().stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut full_content = String::new();
    let mut has_error = false;
    for line in reader.lines() {
        if let Ok(content) = line {
            let coc = content.clone();
            if content.contains("error:")
                || content.contains("fatal error")
                || content.contains("failed")
            {
                full_content.push_str(format!("{}\n", &coc).as_str());
                has_error = true
            } else {
                print!("\r\x1B[K{}", content.purple().bold());
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(10));
                full_content.push_str(format!("{}\n", &coc).as_str());
            }
        }
    }

    let log = log_to_file(
        directory.clone(),
        "install".to_string(),
        full_content.clone(),
    );
    println!("\nLog for unin step \"install\" can be found here: {}", log);
    drop(log);

    if has_error {
        println!("Installation failed. Here is the full output:");
        println!("{}", full_content);
        exit(1);
    }
    println!("Installation finished successfully.");
    let output_dir = format!("{}/build/", directory.to_str().unwrap());
    let files = find_files_because_the_user_is_too_lazy(PathBuf::from(output_dir.clone()));
    for binary in files {
        let last_item_binary = binary
            .to_str()
            .unwrap()
            .split("/")
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        let installed_absolute_path = format!("/usr/local/bin/{}", last_item_binary);
        let temp_binary: UninPackage = UninPackage {
            name: binary
                .to_str()
                .unwrap()
                .split('/')
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string(),
            paths: vec![PathBuf::from(installed_absolute_path)],
            change_date: time_create(),
            updated: false,
        };
        registry_write(&temp_binary, true);
        println!("\n{}", temp_binary);
    }
    println!("B")
}
pub fn clean(directory: PathBuf) {
    if !directory.exists() {
        println!("Nothing to do.");
        exit(0)
    }
    let cleaner = fs::remove_dir_all(format!("{}/build", directory.to_str().unwrap()));
    if cleaner.is_err() {
        println!("Cleaning failed.");
        exit(1);
    }
    println!("Cleaned build directory.");
}
