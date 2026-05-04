//these are all imports
use crate::logging::log_to_file;
use crate::tools::{find_files_because_the_user_is_too_lazy, install_to_bin};
use colored::Colorize;
use dialoguer::{Confirm, Input};
use regex::Regex;
use std::{
    fs as filesystem,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{self as commands, Stdio, exit},
    thread as sleeping,
    time::Duration,
};
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

        let check_user_continue_old_args: bool = Confirm::new() //asks the user if they want to use the old arguments
            .with_prompt("Do you want to use the already used, cached arguments? (y/n)")
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
    let configure_cmake = commands::Command::new("cmake") //configure command, the core of this function
        .current_dir(build_dir)
        .arg("..")
        .arg("-Wno-dev")
        .args(input_vec)
        .output()
        .expect("Failed to configure");

    print!(
        "{}",
        String::from_utf8_lossy(&configure_cmake.stdout).yellow()
    ); //prints stdout
    eprint!("{}", String::from_utf8_lossy(&configure_cmake.stderr).red()); //prints stderr

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

    if noinstall {
        //if the user only wants to build
        println!("Skipping install step."); //notifies
        let mut make_process = commands::Command::new("make") //builds the project
            .current_dir(build_directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start make");

        if let Some(stdout) = make_process.stdout.take() {
            //If the stdout is not None
            let buf_reader = BufReader::new(stdout); //opens a BufReader

            for line in buf_reader.lines() {
                //does this for every line in the stdout
                match line {
                    //matches the line
                    Ok(content) => {
                        //if the line is fine
                        let content = content.replace('\r', ""); //replaces \r with nothing
                        println!("{}", content.bold().green()); //prints the content
                    }
                    Err(e) => println!("Error reading stdout: {}", e), //prints the error if there is one
                }
            }
        }

        if let Some(stderr) = make_process.stderr.take() {
            //if the stderr is not None
            let buf_reader = BufReader::new(stderr); //opens another BufReader

            for line in buf_reader.lines() {
                //does this for every line in the stderr
                match line {
                    //matches the line
                    Ok(content) => {
                        //if the line is fine
                        let content = content.replace('\r', ""); //replaces \r with nothing
                        eprintln!("{}", content.red()); //prints the error
                    }
                    Err(e) => println!("Error reading stderr: {}", e), //if the error has an error, print the error of the error
                }
            }
        }

        let make_process_status = make_process
            .wait_with_output()
            .expect("Command isn't running."); //waits for the command to finish

        if make_process_status.status.code() != Option::from(0) {
            println!("{}", "Compilation failed.".red());
            exit(1);
        } else {
            println!("{}", "Compilation finished successfully.".green());
        }
        let out = String::from_utf8_lossy(&make_process_status.stdout).into_owned();
        let logger = log_to_file(directory.to_path_buf(), "make".to_string(), out);
        println!(
            "\nLog for unin step \"make\" build can be found here: {}",
            logger
        );
        drop(logger);
    } else {
        //if the user wants to build and install
        let mut make_process = commands::Command::new("cmake") //starts compiling
            .arg("--build")
            .current_dir(&build_directory)
            .arg(".")
            .arg("-j")
            .arg(cores.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start cmake build");

        if let Some(stdout) = make_process.stdout.take() {
            //if the stdout is not None
            let buf_reader = BufReader::new(stdout); //opens a BufReader

            for line in buf_reader.lines() {
                //does this for every line in the stdout
                match line {
                    //matches the line
                    Ok(content) => {
                        //if the line is fine
                        if content.contains("Building") {
                            let mut contented = content.split("[").collect::<Vec<&str>>();
                            contented[0] = "[";
                            let contented_string = contented.iter().copied().collect::<String>();
                            print!("\r\x1B[K{}", contented_string.bold().purple());
                            std::io::stdout().flush().unwrap();
                        }
                    }
                    Err(e) => println!("Error reading stdout: {}", e), //if there is an error, print the error
                }
            }
        }

        if let Some(stderr) = make_process.stderr.take() {
            //if the stderr is not None
            let buf_reader = BufReader::new(stderr); //opens another BufReader

            for line in buf_reader.lines() {
                //does this for every line in the stderr
                match line {
                    //matches the line
                    Ok(content) => {
                        //if the line is fine
                        let content = content.replace('\r', ""); //replaces \r with nothing
                        eprintln!("{}", content.red()); //prints the error
                    }
                    Err(e) => println!("Error reading stderr: {}", e), //if there is an error, print the error
                }
            }
        }

        let waiter = make_process
            .wait_with_output()
            .expect("Command isn't running.");
        let out = String::from_utf8_lossy(&waiter.stdout).into_owned();
        let logger = log_to_file(directory.to_path_buf(), "make".to_string(), out);
        println!(
            "\nLog for unin step \"make\" build can be found here: {}",
            logger
        );
        drop(logger);

        if waiter.status.code() != Option::from(0) {
            println!("{}", "Compilation failed, not installing.".red());
            exit(1);
        }

        let make_install_process = commands::Command::new("sudo") //actually installs the project
            .current_dir(&build_directory)
            .arg("cmake")
            .arg("--install")
            .arg(".")
            .spawn()
            .unwrap();

        let waiter = make_install_process.wait_with_output().unwrap();
        if waiter.status.code() != Option::from(0) {
            println!("{}", "Installation failed.".red());
            exit(1);
        }
        let out = String::from_utf8_lossy(&waiter.stdout).into_owned();
        let logger = log_to_file(directory.to_path_buf(), "install".to_string(), out);
        println!(
            "\nLog for unin step \"install\" build can be found here: {}",
            logger
        );
        // I still need to know the binary paths
        //soooo
        let binaries: Vec<PathBuf> = find_files_because_the_user_is_too_lazy(build_directory); //this is a Vec<PathBuf>
        //add these fuckers to the registry
        let _ = install_to_bin(binaries); //this also registers the binaries to the registry
    }
    exit(0)
}

pub fn clean(directory: PathBuf) {
    //cleans the build directory
    println!("Cleaning artefacts built."); //notifies the user
    filesystem::remove_dir_all(format!("{}/build", directory.to_str().unwrap())).unwrap(); //actually does it
}
//this is just a test to see how my time is getting tracked in hackatime
