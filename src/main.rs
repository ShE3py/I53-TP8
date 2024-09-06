use clap::Parser;
use std::path::PathBuf;
use rame::RoCode;

#[derive(Parser)]
struct Cli {
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let code = RoCode::<u8>::parse(&cli.path);
    
    println!("{code}");
}
