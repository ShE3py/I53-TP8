#![cfg(feature = "optimizer")]

use clap::{Parser, ValueHint};
use rame_driver::optimize;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;

/// Optimize a RAM program.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    /// The program to optimize.
    #[arg(value_name = "infile", value_hint = ValueHint::FilePath)]
    infile: PathBuf,
    
    /// Where to place the optimized program.
    #[arg(short = 'o', value_name = "outfile", default_value = "a.out", value_hint = ValueHint::FilePath)]
    outfile: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    
    let infile = match File::open(&cli.infile) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {e}", &cli.infile.display());
            exit(1);
        },
    };
    
    optimize(infile, &cli.infile, Some(&cli.outfile));
}
