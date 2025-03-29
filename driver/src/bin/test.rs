use std::any::type_name;
use clap::Parser;
use rame::model::{Integer, RoCode};
use rame::runner::Ram;
use rame_driver::{Bits, Driver};
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{exit, ExitCode, Termination};
use std::str::FromStr;

#[cfg(feature = "optimizer")]
use std::io::{self, Write};

/// Test an algorithmic program.
#[derive(Parser)]
#[command(version, arg_required_else_help = !cfg!(feature = "compiler"))]
struct Cli {
    /// The path of the compiler to use.
    #[arg(short = 'c', long = "cc", value_name = "compiler", required = !cfg!(feature = "compiler"))]
    compiler: Option<PathBuf>,

    /// The integers' width.
    #[arg(short, long, default_value = "16")]
    bits: Bits,

    /// The files to test.
    #[arg(value_name = "infile", default_value = "tests")]
    infiles: Vec<PathBuf>,
}

struct UnitTest<T: Integer> {
    input: Vec<T>,
    output: Vec<T>,
}

impl<T: Integer> UnitTest<T> {
    /// Returns `Some` with the current output if the test failed.
    #[must_use]
    fn run(&self, code: RoCode<T>) -> Option<Vec<T>> {
        let mut ram = Ram::new(code, self.input.iter().copied());
        let out = ram.run();

        (out != &self.output).then_some(ram.into())
    }
}

impl<T: Integer> Display for UnitTest<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "# TEST: {:?} => {:?}", self.input, self.output)
    }
}

fn parse_headers<T: Integer>(path: &Path) -> Vec<UnitTest<T>> {
    let f = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            exit(1);
        },
    };
    
    let r = BufReader::new(f);
    let mut tests = Vec::new();
    
    for l in r.lines() {
        let l = match l {
            Ok(l) => l,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                exit(1);
            }
        };
        
        if l.trim_start().starts_with("# TEST: ") {
            let Some((input, output)) = &l.trim()["# TEST: ".len()..].split_once("=>") else {
                eprintln!("{}: bad test", path.display());
                exit(1);
            };

            let input = parse_vec(input, path);
            let output = parse_vec(output, path);

            tests.push(UnitTest {
                input,
                output,
            })
        }
    }
    
    tests
}

fn parse_vec<T: FromStr<Err: Display>>(s: &str, path: &Path) -> Vec<T> {
    let s = s.trim().strip_prefix('[').expect("missing `[`").strip_suffix(']').expect("missing `]`");

    if s.is_empty() {
        return Vec::new();
    }
    
    let iter = s.split(',');
    
    let mut v = Vec::with_capacity(iter.size_hint().0);
    for elem in iter {
        v.push(match T::from_str(elem.trim()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: {}: parsing {elem:?} as {}: {e}", path.display(), type_name::<T>());
                exit(1)
            }
        });
    }

    v
}

#[must_use]
fn scan_file<T: Integer + TryFrom<i128, Error: Debug>>(p: &Path, cc: &Option<PathBuf>) -> bool {
    match fs::metadata(p) {
        Ok(m) => if m.is_dir() {
            let mut ok = true;
            for entry in fs::read_dir(p).unwrap() {
                ok &= scan_file::<T>(&entry.unwrap().path(), cc);
            }

            return ok;
        },
        Err(e) => {
            eprintln!("{}: {}: {e}", env!("CARGO_BIN_NAME"), p.display());
            return false;
        },
    }

    print!("{}... ", p.display());
    _ = io::stdout().flush();

    let tests = parse_headers::<T>(p);
    if tests.is_empty() {
        eprintln!("{}: {}: no test", env!("CARGO_BIN_NAME"), p.display());
        return false;
    }

    let code = Driver::new()
        .infile(p)
        .compile(true)
        .compiler(cc.as_ref())
        .try_drive();

    let code = match code {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: failed to compiled `{}`: {e}", p.display());
            return false;
        }
    };

    #[cfg(feature = "optimizer")]
    let opt = code.optimize();

    let mut ok = true;

    for test in tests {
        if let Some(out) = test.run(code.try_cast().unwrap()) {
            if ok {
                println!("failed");
                ok = false;
            }

            eprintln!(" {test}: got {out:?} instead");
            continue;
        }

        #[cfg(feature = "optimizer")]
        if let Some(out) = test.run(opt.try_cast().unwrap()) {
            if ok {
                println!("failed");
                ok = false;
            }
            
            eprintln!("opt: {test}: got {out:?} instead");
            continue;
        }
    }

    if ok {
        println!("ok");
    }
    ok
}

fn main() -> impl Termination {
    let cli = Cli::parse();

    let ok = match cli.bits {
        Bits::Int8 => cli.infiles.iter().all(|p| scan_file::<i8>(&p, &cli.compiler)),
        Bits::Int16 => cli.infiles.iter().all(|p| scan_file::<i16>(&p, &cli.compiler)),
        Bits::Int32 => cli.infiles.iter().all(|p| scan_file::<i32>(&p, &cli.compiler)),
        Bits::Int64 => cli.infiles.iter().all(|p| scan_file::<i64>(&p, &cli.compiler)),
        Bits::Int128 => cli.infiles.iter().all(|p| scan_file::<i128>(&p, &cli.compiler)),
    };
    
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
