use clap::{Parser, ValueEnum};
use rame::run::Ram;
use rame::{Integer, RoCode};
use std::fmt::Display;
use std::io;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Neg;
use std::path::PathBuf;
use std::process::exit;

#[cfg_attr(feature = "optimizer", doc = "Run, test or optimize a RAM program.")]
#[cfg_attr(not(feature = "optimizer"), doc = "Run or test a RAM program.")]
#[derive(Parser)]
#[cfg_attr(not(any(feature = "optimizer", feature = "dynamic_jumps")), command(version = concat!(env!("CARGO_PKG_VERSION"), " (no optimizer or dynamic jumps)")))]
#[cfg_attr(feature = "optimizer", command(version = concat!(env!("CARGO_PKG_VERSION"), " (with optimizer, no dynamic jumps)")))]
#[cfg_attr(feature = "dynamic_jumps", command(version = concat!(env!("CARGO_PKG_VERSION"), " (with dynamic jumps, no optimizer)")))]
struct Cli {
    /// The program to execute.
    #[arg(value_name = "FILE")]
    path: PathBuf,
    
    /// The integer's type bits.
    #[arg(short, long, default_value = "16")]
    bits: Bits,
    
    /// The program's arguments.
    #[arg(value_name = "ARGS", value_delimiter = ',', num_args = 0..)]
    input: Option<Vec<i128>>,
    
    /// Test the program's output.
    #[arg(short = 't', long = "test", value_delimiter = ',', num_args = 0..)]
    output: Option<Vec<i128>>,
    
    /// Optimize the program into the specified file (experimental).
    #[arg(short = 'o', long = "optimize", value_name = "FILE")]
    #[cfg(feature = "optimizer")]
    optimize: Option<Option<PathBuf>>,
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

#[cfg(feature = "optimizer")] type Optimize = Option<Option<PathBuf>>;
#[cfg(not(feature = "optimizer"))] type Optimize = ();

fn run<T: Integer + Neg<Output = T> + TryFrom<i128, Error: Display>>(cli: Cli) {
    fn cvt<T: Integer + TryFrom<i128, Error: Display>>(opt: Option<Vec<i128>>) -> Option<Vec<T>> {
        opt.map(|vec| Vec::from_iter(vec.into_iter().map(|v| match T::try_from(v) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("invalid integer {v}: {e}");
                exit(1);
            }
        })))
    }
    
    let input = cvt(cli.input);
    let ouput = cvt(cli.output);
    
    #[cfg(feature = "optimizer")] let optimize = cli.optimize;
    #[cfg(not(feature = "optimizer"))] let optimize = ();
    
    if let Some(input) = input {
        run_file(cli.path, input, ouput, optimize)
    } else {
        run_file(cli.path, Stdin::<T>::new(), ouput, optimize)
    }
}

#[cfg_attr(not(feature = "optimizer"), expect(unused_mut, unused_variables))]
fn run_file<T: Integer + Neg<Output = T>, I: Iterator<Item = T>>(path: PathBuf, input: impl IntoIterator<IntoIter = I>, output: Option<Vec<T>>, optimize: Optimize) {
    let mut code = RoCode::<T>::parse(path.as_path());
    
    #[cfg(feature = "optimizer")]
    if optimize.is_some() {
        code = ::rame::opt::SeqRewriter::from(&code).optimize().rewritten();
        
        if let Some(Some(f)) = optimize {
            if let Err(e) = code.write_to_file(f) {
                eprintln!("error: unable to save optimized code: {e}");
            }
        }
    }
    
    let ret = Ram::new(code, input).run();
    
    match output {
        Some(output) => if ret != output {
            eprintln!("error: output mismatch");
            eprintln!(" computed: {ret:?}");
            eprintln!(" expected: {output:?}");
            exit(1);
        },
        None => println!("Output = {:?}", ret),
    }
}
