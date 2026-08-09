use crate::tools::detect;
use colored::Colorize;
use git2::Repository;
use std::fs;

pub fn git(repo: String, noinstall: bool) {
    //unin config file is unin.json

    let clone_location = std::path::Path::new(
        format!("{}/.cache/unin", std::env::home_dir().unwrap().display()).as_str(),
    )
    .to_owned();

    if !clone_location.clone().exists() {
        println!(
            "{}",
            format!(
                "Cache directory not found, creating it at {}",
                clone_location.display()
            )
            .yellow()
        );
        fs::create_dir_all(&clone_location).unwrap();
    }

    Repository::clone(&repo, &clone_location).unwrap();

    detect(clone_location.to_str().unwrap().to_string(), noinstall); //this is the builder

    fs::remove_dir_all(clone_location).unwrap();
}
