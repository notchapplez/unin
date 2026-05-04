use crate::tools::find_files_because_the_user_is_too_lazy;
use colored::Colorize;
use std::path::PathBuf;
use std::process::exit;
use std::{fs, io};

pub fn compile_go(directory: PathBuf, noinstall: bool) {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;

    let mut child = Command::new("go")
        .args(["build", "-o", "unin_built_temp/"]) // don't specify no file for ****'s sake
        .current_dir(&directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn failed");

    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");

    let (tx, rx) = mpsc::channel::<(String, bool)>();

    let tx1 = tx.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    eprintln!("{}", l);
                    let _ = tx1.send((l, true));
                }
                Err(_) => break,
            }
        }
    });

    thread::spawn(move || {
        let mut out = io::stdout();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let _ = writeln!(out, "{}", l);
                    let _ = out.flush();
                    let _ = tx.send((l, false));
                }
                Err(_) => break,
            }
        }
    });

    while let Ok((line, _is_err)) = rx.recv() {
        print!("got {}", line);
        io::stdout().flush().unwrap();
    }

    let status = child.wait().expect("wait failed");
    if !status.success() {
        eprintln!("process exited with {}", status);
    }

    if noinstall {
        println!(
            "{}",
            "Skipping installation. Binaries can be found in unin_built_temp/"
                .yellow()
                .underline()
        );
        let test = PathBuf::from(format!("{}/unin_built_temp/", directory.to_str().unwrap()));
        println!();
        println!("I found some files, here they are:");
        find_files_because_the_user_is_too_lazy(test)
            .iter()
            .for_each(|x| println!("{}", x.to_str().unwrap()));
        exit(0)
    }
}
pub fn clean(directory: PathBuf) {
    let build_dir = PathBuf::from(format!("{}/unin_built_temp/", directory.to_str().unwrap()));
    if !build_dir.exists() {
        println!("Nothing to do");
        exit(0);
    }
    fs::remove_dir_all(build_dir).expect("The cleaner fucked up. i don't know why.")
}
