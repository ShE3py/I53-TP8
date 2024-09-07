use clap::{Parser, ValueEnum};
use rame::run::Ram;
use rame::{Integer, RoCode};
use std::io;
use std::io::Write;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// The program to execute.
    #[arg(value_name = "FILE")]
    path: PathBuf,
    
    /// The integer's type bits.
    #[arg(short, long, default_value = "16")]
    bits: Bits,
}

#[derive(ValueEnum, Copy, Clone, Default)]
enum Bits {
    #[clap(name = "8")] Int8,
    #[clap(name = "16")] #[default] Int16,
    #[clap(name = "32")] Int32,
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

struct Stdin<T: Integer> {
    buf: String,
    i: usize,
    _phantom: PhantomData<T>,
}

impl<T: Integer> Stdin<T> {
    fn new() -> Stdin<T> {
        Stdin {
            buf: String::new(),
            i: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T: Integer> Iterator for Stdin<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            print!("E{} = ", self.i);
            drop(io::stdout().flush());
            
            self.buf.clear();
            match io::stdin().read_line(&mut self.buf) {
                Ok(0) => {
                    println!("<eof>");
                    return None
                },
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error: failed to read stdin: {e}");
                    return None;
                }
            };
            
            match T::from_str(self.buf.trim()) {
                Ok(v) => {
                    self.i += 1;
                    break Some(v)
                },
                Err(e) => eprintln!("error: {e}"),
            }
        }
    }
}

fn run<T: Integer>(cli: Cli) {
    println!("Output = {:?}", Ram::new(RoCode::<T>::parse(&cli.path), Stdin::<T>::new()).run());
}
