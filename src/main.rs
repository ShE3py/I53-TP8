use clap::Parser;
use rame::RoCode;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// The program to execute.
    #[arg(value_name = "FILE")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let code = RoCode::<u8>::parse(&cli.path);
    
    println!("{code}");
}
