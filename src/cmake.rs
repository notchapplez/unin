//these are all imports
use crate::logging::log_to_file;
use crate::tools::find_files_because_the_user_is_too_lazy;
use colored::Colorize;
use dialoguer::{Confirm, Input};
use duct::cmd;
use regex::Regex;
use std::{
    fs as filesystem,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{self as commands, exit},
    thread as sleeping,
    time::Duration,
};

use unicode_segmentation::UnicodeSegmentation;
use unin_bin::{UninPackage, registry_write, time_create};

pub fn compile_cmake(directory: PathBuf, noinstall: bool) {
    //defines the function
    let build_dir: PathBuf = PathBuf::from(format!("{}/build", directory.to_str().unwrap()));
    println!("Now configuring {}", directory.to_str().unwrap().yellow()); //prints the configuring message
    let cmake_lists_path = format!("{}/CMakeLists.txt", directory.to_str().unwrap()); //defines CMakeLists.txt path

    let cmake_lists: PathBuf = PathBuf::from(&cmake_lists_path); //defines cmake_lists as a PathBuf
    let opened_file = std::fs::read_to_string(cmake_lists).unwrap(); //defined the opened file
    println!(
        //prints help for the user
        "{} | {} | {}",
        "Option".bold().red(),
        "Description".bold().red(),
        "Default value".bold().red()
    );
    sleeping::sleep(Duration::from_millis(500)); //waits for the user to read the help
    let regex = Regex::new(r#""[^"]*"|\S+"#).unwrap(); //sets the regex pattern

    for line in opened_file.lines() {
        //"for" loop to read the file line by line
        if line.contains("option(") {
            //if the line contains option()
            line.split("("); //to do this, split the line by ()
            let linecontentfiltered = line.replace("option(", "").replace(")", "").to_string(); //some formatting stuff

            let result: Vec<&str> = regex
                .find_iter(linecontentfiltered.as_str())
                .map(|m| m.as_str())
                .collect(); //collects the result by matching against the regex

            let result_string = result.join(" "); //joins the result into a string
            println!("{}", result_string.bold().green()); //prints the string
            sleeping::sleep(Duration::from_millis(10)); //waits a bit
        }
    }
    let arguments_history: PathBuf = //defines the path to the .unin_arguments file
        PathBuf::from(format!("{}/.unin_arguments", directory.to_str().unwrap()));

    if arguments_history.exists() {
        //if the file exists
        let old_argument_read = filesystem::read_to_string(&arguments_history)
            .unwrap()
            .replace("-DCMAKE_INSTALL_PREFIX=/usr/local", "")
            .replace("-Wno-dev", "")
            .trim()
            .to_string(); //reads the file contents, obviously

        println!(
            "Following arguments were found as they were present in the .unin_arguments file: {}",
            old_argument_read.bold().yellow().underline()
        ); //notifies the user of the file

        let check_user_continue_old_args: bool =
            Confirm::new() //asks the user if they want to use the old arguments
                .with_prompt("Do you want to use the already used, cached arguments?")
                .interact()
                .unwrap();

        if check_user_continue_old_args {
            //if they do, use the old arguments and build
            let old_args = filesystem::read_to_string(&arguments_history).unwrap();

            configure(old_args.split(" ").collect(), &directory);

            make(directory, build_dir, noinstall);
        } else {
            //if not, ask them again
            let input: String = Input::new() //get their input to sell to companies without their consent /j
                .allow_empty(true)
                .with_prompt("Add Arguments now. Prefix your project arguments with -D and use a space for separation, for example -DBUILD_SHARED_LIBS=ON. Other arguments will also be used, like warning flags.")
                .interact_text()
                .unwrap();

            println!(); //i dont know, it's just here

            let full_cmake_input = format!("{} -DCMAKE_INSTALL_PREFIX=/usr/local -Wno-dev", &input); //adds the -DCMAKE_INSTALL_PREFIX=/usr/local to the input
            let input_vec: Vec<&str> = input.split(' ').collect(); //splits the input into a vector

            configure(input_vec, &directory); //configures the project

            filesystem::remove_file(&arguments_history).unwrap(); //removes the old arguments file
            filesystem::write(arguments_history, full_cmake_input.clone()).unwrap(); //creates a new arguments file with the new input

            make(directory, build_dir, noinstall); //builds the project, what else would it do?
        }
    } else {
        //if the file doesn't exist, ask the user for input
        let mut input: String = Input::new() //input here
            .allow_empty(true)
            .with_prompt("Add Arguments now. Prefix your project arguments with -D and use a space for separation, for example -DBUILD_SHARED_LIBS=ON. Other arguments will also be used, like warning flags.")
            .interact_text()
            .unwrap();
        input = input.trim().to_string();
        println!(); //I still don't know

        println!("{}", input); //prints the input, as you can see (you fuckhead)
        let full_cmake_input = format!("{} -DCMAKE_INSTALL_PREFIX=/usr/local -Wno-dev", &input); //sets the full cmake args
        let input_vec: Vec<&str> = full_cmake_input.split(" ").collect(); //splits the input into a vector
        filesystem::write(arguments_history, full_cmake_input.clone()).unwrap(); //writes the input to the file
        println!("{:?}", input_vec); //prints the input vector

        configure(input_vec, &directory); //configures the project

        make(directory, build_dir, noinstall); //builds the project
    }
}

fn configure(input_vec: Vec<&str>, directory: &Path) {
    //configuration function
    println!(); //I still don't know what this does
    filesystem::create_dir_all(format!("{}/build", directory.to_str().unwrap())).unwrap(); //creates the build directory

    let build_dir = format!("{}/build", directory.to_str().unwrap()); //sets the path to the build directory
    let configure_cmake = commands::Command::new("cmake") //configure command, the core of this function // we are leaving Command here and not using cmd! because that is too tedious
        .current_dir(build_dir)
        .arg("..")
        .arg("-Wno-dev")
        .args(input_vec)
        .output()
        .expect("Failed to configure");

    if !configure_cmake.status.success() {
        //if the configure command failed
        eprintln!("CMake configure failed. See the output above for more information."); //inform the user
        exit(1); //exit the program
    }

    let out = String::from_utf8_lossy(&configure_cmake.stdout).into_owned();
    let logger = log_to_file(directory.to_path_buf(), "configure".to_string(), out);
    println!(
        "\nLog for unin step \"configure\" build can be found here: {}",
        logger
    );
    drop(logger);
}
fn make(directory: PathBuf, build_directory: PathBuf, noinstall: bool) {
    //define the building function

    println!("{}", "Starting to compile in three seconds. This might use up to 100% of your CPU. To cancel, press Ctrl+C".blue()); //compile warning
    sleeping::sleep(Duration::from_secs(3)); //wait 3 secs

    let cores = num_cpus::get(); //number of cores
    println!("Now compiling {}", directory.to_str().unwrap().yellow()); //Start message

    let main_command = cmd!("cmake", "--build", ".", "-j", cores.to_string())
        .dir(&build_directory)
        .stderr_to_stdout()
        .unchecked();

    let mut output_merged = String::new();
    let mut has_error_build = false;

    let cols = terminal_size::terminal_size().unwrap();

    let reader = main_command
        .reader()
        .unwrap_or_else(|e| panic!("Failed to read output from cmake build: {}", e));

    let buf_reader = BufReader::new(reader);

    let mut stdout = std::io::stdout();
    let mut last_display = String::new();
    let mut lower = String::new();

    for line in buf_reader.lines() {
        let content = match line {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading stdout: {}", e);
                continue;
            }
        };

        lower.clear();
        lower.extend(content.chars().flat_map(|c| c.to_lowercase()));
        let is_error =
            lower.contains("error:") || lower.contains("failed") || lower.contains("cmake error");

        if is_error {
            has_error_build = true;
            output_merged.push_str(content.trim_end());
            output_merged.push('\n');
            continue;
        }

        if content.trim_start().starts_with("--") {
            let content_trimmed = content.trim_start().trim_start_matches('-').trim_start();
            let mut truncated = String::new();
            for g in content_trimmed.graphemes(true) {
                if truncated.graphemes(true).count() >= cols.0.0 as usize {
                    break;
                }
                truncated.push_str(g);
            }

            if truncated != last_display {
                last_display.clear();
                last_display.push_str(&truncated);
                print!("\r\x1B[K{}", truncated.bold().purple());
                stdout.flush().ok();
            }

            output_merged.push_str(content_trimmed);
            output_merged.push('\n');
        }
    }

    if has_error_build {
        println!(
            "{}",
            "Compilation failed. The full output will be printed below".red()
        );
        println!("{}", output_merged);
    }
    let logger = log_to_file(
        directory.to_path_buf(),
        "make".to_string(),
        output_merged.clone(),
    );
    println!(
        "\nLog for unin step \"make\" build can be found here: {}",
        logger
    );
    drop(logger);
    if noinstall {
        println!("Skipping install step.");
        println!("The binaries must be somewhere in the build/ directory, go find them");
        exit(0)
    }

    let make_install_process = cmd!("cmake", "--install", ".")
        .dir(&build_directory)
        .stderr_to_stdout();

    let lines = make_install_process
        .reader()
        .unwrap_or_else(|e| panic!("Failed to read output from cmake installer: {}", e));
    let reader = BufReader::new(lines);
    let mut output_merged = String::new();
    let mut has_error_build = false;

    for line in reader.lines().map_while(Result::ok) {
        let cloned_line = line.clone();
        let is_error = cloned_line.contains("error:")
            || cloned_line.contains("failed")
            || cloned_line.contains("CMake Error");

        if !is_error {
            let cloned_line = cloned_line.replace("\n", "");
            print!("\r\x1B[K{}", cloned_line.bold().purple());
            std::io::stdout().flush().unwrap();
            output_merged.push_str(cloned_line.as_str());
            output_merged.push('\n');
            continue;
        } else {
            let cloned_line = cloned_line.replace("\n", "");
            has_error_build = true;
            output_merged.push_str(cloned_line.as_str());
            output_merged.push('\n');
            continue;
        }
    }
    let logger = log_to_file(
        directory.to_path_buf(),
        "install".to_string(),
        output_merged.clone(),
    );
    println!(
        "\nLog for unin step \"install\" build can be found here: {}",
        logger
    );
    if has_error_build {
        println!(
            "{}",
            "Installation failed. The full output will be printed below".red()
        );
        println!("{}", output_merged);
    }

    let binaries: Vec<PathBuf> = find_files_because_the_user_is_too_lazy(build_directory); //this is a Vec<PathBuf> as I declared
    binaries.iter().for_each(|binary| {
        let package = UninPackage {
            name: binary
                .to_str()
                .unwrap()
                .split("/")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .to_string(),
            paths: vec![binary.clone().to_owned()],
            change_date: time_create(),
            updated: false,
        };
        registry_write(&package, true);
    });

    exit(0)
}

pub fn clean(directory: PathBuf) {
    //cleans the build directory
    println!("Cleaning artefacts built."); //notifies the user
    filesystem::remove_dir_all(format!("{}/build", directory.to_str().unwrap())).unwrap(); //actually does it
}
//this is just a test to see how my time is getting tracked in hackatime
