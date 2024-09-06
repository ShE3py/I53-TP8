use clap::{Parser, ValueEnum};
use rame::run::Ram;
use rame::{Integer, RoCode};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// The program to execute.
    #[arg(value_name = "FILE")]
    path: PathBuf,
    
    /// The integer's type bits.
    #[arg(short, long, default_value = "32")]
    bits: Bits,
}

#[derive(ValueEnum, Copy, Clone, Default)]
enum Bits {
    #[clap(name = "8")] Int8,
    #[clap(name = "16")] Int16,
    #[clap(name = "32")] #[default] Int32,
    #[clap(name = "64")] Int64,
    #[clap(name = "128")] Int128,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.bits {
        Bits::Int8 => run::<i8>(cli),
        Bits::Int16 => run::<i16>(cli),
        Bits::Int32 => run::<i32>(cli),
        Bits::Int64 => run::<i64>(cli),
        Bits::Int128 => run::<i128>(cli),
    }
}

fn run<T: Integer>(cli: Cli) {
    println!("Output = {:?}", Ram::without_inputs(RoCode::<T>::parse(&cli.path)).run());
}
