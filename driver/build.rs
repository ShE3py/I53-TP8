use std::{env, fs};
use std::io::ErrorKind;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    if !cfg!(feature = "compiler") {
        return ExitCode::SUCCESS;
    }

    const SRC: &str = "../compiler/";
    let out = env::var("OUT_DIR").expect("missing OUT_DIR");
    
    if let Err(e) = fs::create_dir(format!("{out}/llvm")) {
        if e.kind() != ErrorKind::AlreadyExists {
            eprintln!("mkdir: {e}");
        }
    }
    
    if let Err(e) = fs::create_dir(format!("{out}/ram")) {
        if e.kind() != ErrorKind::AlreadyExists {
            eprintln!("mkdir: {e}");
        }
    }
    
    println!("cargo::rerun-if-changed={SRC}");
    
    let mut cmd = Command::new("/usr/bin/make");
    cmd
        .current_dir(SRC)
        .arg("-B")
        .arg(format!("{out}/libarc.a"))
        .arg(format!("OUT={out}"));
    
    println!("Executing: {cmd:?}");
    
    match cmd.status() {
        Ok(status) if status.success() => {
            println!("cargo::rustc-link-search={out}");
            println!("cargo::rustc-link-lib=arc");
            println!("cargo::rustc-link-lib=fl");
            println!("cargo::rustc-link-lib=LLVM-18");
            println!("cargo::rustc-link-lib=stdc++");
            ExitCode::SUCCESS
        },
        Ok(status) => {
            if let Some(code) = status.code().and_then(|code| u8::try_from(code).ok()) {
                ExitCode::from(code)
            }
            else {
                eprintln!("Error: {status}");
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
