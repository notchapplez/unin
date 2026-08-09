use std::path::PathBuf;
use duct::cmd;
use core::time::Duration;
use std::fs::File;
use crate::tools::calculate_hash;
use crate::uniconf::Uniconf;

// pub struct Uniconf {
//     bin_name: String,
//     lang: String,
//     compile_instruct: Vec<String>,
//     bin_location: Vec<String>, //. for path root
//     install_command: Vec<String>,
//     clean_command: Vec<String>,
//     license: String,
//     author: String,
// }

pub fn uninjson_compile(uniconf: Uniconf, dir: &PathBuf) {
    println!(
        "Compiling project \"{}\" in language {}. This project is licensed to {} under a license of type {}",
        uniconf.bin_name, uniconf.lang, uniconf.author, uniconf.license
    );

    let spinner = indicatif::ProgressBar::new_spinner();

    spinner.set_message("Running. This may take a while, as well as use up a lot of RAM and CPU.");
    spinner.enable_steady_tick(Duration::from_millis(10));

    uniconf.compile_instruct.iter().for_each(|i|{
        let temp: (&str, &str) = i.split_once(" ").unwrap();

        let mut hash_value = calculate_hash(&temp.1.to_string()).to_string();

        let file: File = File::create(format!("{}/.{}-{:?}", dir.to_str().unwrap(), temp.0, hash_value.truncate(10))).unwrap();
        let comp_cmd = cmd!(temp.0, temp.1)
            .dir(dir)
            .stderr_to_stdout()
            .stdout_file()
            .run();

        if comp_cmd.is_err() {
            println!("One of the compilation processes failed. Output will be shown below.")
        }
    });

    spinner.finish_and_clear();

    todo!()
}