use std::fmt::{Debug, Display};
use clap::{Parser, ValueHint};
use rame::runner::Ram;
use rame_driver::{cvt, Bits, Driver, Stdin};
use std::path::PathBuf;
use rame::model::{Integer, RoCode};

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

    /// The integers' width.
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

fn poly<T: Integer + TryFrom<i128, Error: Display + Debug>>(code: &RoCode<i128>, args: &[i128]) {
    let args: Vec<T> = cvt(&args);
    let offset = args.len();
    let args = args.into_iter().chain(Stdin::new(|i| print!("E{} =", i + offset)));
    let ram = Ram::new(code.try_cast().unwrap(), args);

    println!("Output = {:?}", ram.run());
}

fn main() {
    let cli = Cli::parse();

    #[cfg(feature = "optimizer")] let optimize = cli.optimize;
    #[cfg(not(feature = "optimizer"))] let optimize = false;

    #[cfg(feature = "compiler")] let compile = cli.compile;
    #[cfg(not(feature = "compiler"))] let compile = false;

    let code = Driver::new()
        .infile(&cli.infile)
        .optimize(optimize)
        .compile(compile)
        .drive();

    match cli.bits {
        Bits::Int8   => poly::<i8>  (&code, &cli.args),
        Bits::Int16  => poly::<i16> (&code, &cli.args),
        Bits::Int32  => poly::<i32> (&code, &cli.args),
        Bits::Int64  => poly::<i64> (&code, &cli.args),
        Bits::Int128 => poly::<i128>(&code, &cli.args),
    }
}
