use clap::{Parser, ValueHint};
use std::path::PathBuf;
use rame_driver::Driver;

/// Compiles an algorithmic program into a RAM one.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    /// The program to compile.
    #[arg(value_name = "infile", value_hint = ValueHint::FilePath)]
    infile: PathBuf,
    
    /// Where to place the compiled program.
    #[arg(short = 'o', value_name = "outfile", default_value = "a.out", value_hint = ValueHint::FilePath)]
    outfile: PathBuf,
    
    /// Turn on all optimizations.
    #[arg(short = 'O', default_value_t = false)]
    optimize: bool,
}

fn main() {
    let cli = Cli::parse();

    Driver::new()
        .infile(&cli.infile)
        .outfile(&cli.outfile)
        .compile(true)
        .optimize(cli.optimize)
        .drive();
}
