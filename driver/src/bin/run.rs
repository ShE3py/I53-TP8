use clap::{Parser, ValueHint};
use rame::runner::Ram;
use rame_driver::{cvt, monomorphize, Bits, Stdin};
use std::path::PathBuf;

/// Run an algorithmic or RAM program.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    /// The program to run.
    #[arg(value_name = "infile", value_hint = ValueHint::FilePath)]
    infile: PathBuf,
    
    /// The program's arguments.
    #[arg(value_name = "args", value_delimiter = ',', num_args = 0..)]
    args: Vec<i128>,
    
    /// The integer's type bits.
    #[arg(short, long, default_value = "16")]
    bits: Bits,
    
    /// Optimize the RAM program before running it.
    #[arg(short = 'O', default_value_t = false)]
    #[cfg(feature = "optimizer")]
    optimize: bool,
    
    /// Compile the algorithmic program as a first step.
    #[arg(short = 'c', default_value_t = false)]
    #[cfg(feature = "compiler")]
    compile: bool,
}

fn main() {
    let cli = Cli::parse();
    
    monomorphize!(&cli.infile, cli.compile, cli.bits, code, {
        #[cfg(feature = "optimizer")]
        let code = if !cli.optimize { code } else {
            rame::optimizer::SeqRewriter::from(&code).optimize().rewritten()
        };
        
        let args = cvt(cli.args, &code);
        let ram = Ram::new(code, args.into_iter().chain(Stdin::new()));
        
        println!("Output = {:?}", ram.run());
    });
}
