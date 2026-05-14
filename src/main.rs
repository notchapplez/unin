pub mod cmake;
mod comms;
mod confidcure;
mod gleam;
mod go;
mod haskell;
pub mod installer;
mod logging;
pub mod make;
mod meson;
mod rust;
pub mod tools;
pub mod zig;

use crate::tools::*;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::{
    io::{self, Write},
    path::PathBuf,
    process::exit,
};
use unin_bin::registry;

#[derive(Parser, Debug)]
#[command(name = "unin", version = "0.1.0", author = "notchapplez")]
struct Cli {
    #[arg(
        long,
        value_enum,
        help = "set up the necessary tools for the compilation"
    )]
    setup: Option<SetupMode>,

    #[arg(
        value_name = "PATH",
        default_value = ".",
        help = "Path to the directory to compile"
    )]
    path: PathBuf,

    #[arg(long, default_value = "false", help = "Skip the install step")]
    noinstall: bool,

    #[arg(
        long,
        default_value = "false",
        help = "debug purposes",
        required = false
    )]
    test: bool,

    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = ".",
        help = "Clean artefacts built",
        required = false,
        conflicts_with_all = ["setup", "noinstall", "test", "path", "uninstall"]
    )]
    clean: Option<PathBuf>,

    #[arg(
        long,
        required = false,
        help = "Uninstall a package",
        conflicts_with_all = ["setup", "noinstall", "test", "path", "clean"]

    )]
    uninstall: Option<String>,

    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "true",
        required = false,
    )]
    queue_registry: Option<bool>,
}

#[derive(Clone, Debug, ValueEnum)]
enum SetupMode {
    Full,    //done
    Rust,    //done
    Cmake,   //done
    Make,    //done, hell yeah
    Go,      //done
    Zig,     //done, but has bugs with files
    Swift,   //not done yet (todo!())
    Haskell, //somewhat works
    D,       //what the hell is this anyway (todo!()) isn't done
}

fn main() {
    let cli = Cli::parse();

    registry::update_check_registry();

    if cli.clean.is_some() {
        println!(
            "Cleaning {}",
            cli.clean
                .clone()
                .unwrap()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .yellow()
        );
        let _ = io::stdout().flush();
        detect_clean(
            cli.clean
                .clone()
                .unwrap_or_else(|| panic!("Couldn't unwrap."))
                .to_str()
                .unwrap()
                .to_owned(),
        );
        exit(0);
    }
    if cli.test {
        registry::temp_test();
        exit(0)
    }
    if !cli.uninstall.clone().unwrap_or_default().is_empty() {
        registry::registry_uninstall(cli.uninstall.clone().unwrap());
        exit(0);
    }
    if cli.queue_registry.is_some() {
        registry::get_registry();
        exit(0);
    }

    if let Some(mode) = cli.setup {
        match mode {
            SetupMode::Full => installer::setup_files_full(),
            SetupMode::Rust => installer::setup_files_lang("rust".to_owned()),
            SetupMode::Cmake => installer::setup_files_lang("cmake".to_owned()),
            SetupMode::Make => installer::setup_files_lang("make".to_owned()),
            SetupMode::Go => installer::setup_files_lang("go".to_owned()),
            SetupMode::Zig => installer::setup_files_lang("zig".to_owned()),
            SetupMode::Swift => installer::setup_files_lang("swift".to_owned()),
            SetupMode::Haskell => installer::setup_files_lang("haskell".to_owned()),
            SetupMode::D => installer::setup_files_lang("d".to_owned()),
        }
    }

    detect(cli.path.to_str().unwrap().to_owned(), cli.noinstall);
}

//meow! i enjoyed coding this man, have fun being submitted to beest :3
