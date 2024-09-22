use clap::{Parser, ValueHint};
use rame_driver::{open, optimize};
use std::path::PathBuf;

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
    
    let infile = open(&cli.infile);
    optimize(infile, &cli.infile, Some(&cli.outfile));
}
