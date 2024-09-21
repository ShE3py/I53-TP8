use std::env;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    const SRC: &str = "../compiler/";
    let out = env::var("OUT_DIR").expect("missing OUT_DIR");
    
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
