use clap::Parser;
use rame::model::{Integer, RoCode};
use rame::runner::Ram;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::str::FromStr;
use std::{fmt, fs, io};
use std::fmt::{Display, Formatter};
use rame_driver::create_temp_out;

/// Test an algorithmic program.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// The path of the compiler to use
    #[arg(short = 'c', long = "cc", value_name = "compiler", required = !cfg!(feature = "compiler"))]
    compiler: Option<PathBuf>,

    /// The files to test.
    #[arg(value_name = "infile", default_value = "tests", allow_hyphen_values = true)]
    infiles: Vec<PathBuf>,
}

struct UnitTest<T: Integer> {
    input: Vec<T>,
    output: Vec<T>,
}

impl<T: Integer> UnitTest<T> {
    #[must_use]
    fn run(&self, code: RoCode<T>) -> Option<Vec<T>> {
        let ram = Ram::new(code, self.input.iter().copied());
        let out = ram.run();

        (out.as_slice() != &self.output).then_some(out)
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
            
            let input = parse_vec(input).expect("invalid input");
            let output = parse_vec(output).expect("invalid output");
            
            tests.push(UnitTest {
                input,
                output,
            })
        }
    }
    
    tests
}

fn parse_vec<T: FromStr>(s: &str) -> Result<Vec<T>, T::Err> {
    let s = s.trim().strip_prefix('[').unwrap().strip_suffix(']').unwrap();
    
    if s.is_empty() {
        return Ok(Vec::new());
    }
    
    let iter = s.split(',');
    
    let mut v = Vec::with_capacity(iter.size_hint().0);
    for elem in iter {
        v.push(T::from_str(elem.trim())?);
    }
    
    Ok(v)
}

fn scan_file(p: &Path, cc: &Option<PathBuf>) {
    match fs::metadata(p) {
        Ok(m) => if m.is_dir() {
            for entry in fs::read_dir(p).unwrap() {
                scan_file(&entry.unwrap().path(), cc);
            }
            
            return;
        },
        Err(e) => {
            eprintln!("{}: {e}", p.display());
            exit(1);
        },
    }
    
    let tests = parse_headers::<i32>(p);
    if tests.is_empty() {
        eprintln!("{}: no test", p.display());
        exit(1);
    }

    let (ram, path) = if let Some(cc) = cc {
        let (_f, path) = create_temp_out(p);

        // SAFETY: unix
        let path = PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(path.into_bytes_with_nul()) });

        let mut cmd = Command::new(cc);
        cmd.arg(p).arg("-o").arg(&path);

        match cmd.status() {
            Ok(s) if s.success() => match File::open(&path) {
                Ok(f) => (f, path),
                Err(e) => { eprintln!("error: failed to open `{}`: {e}", path.display()); exit(1) }
            },
            Ok(s) => { eprintln!("error: {cmd:?}: {s}"); exit(1) },
            Err(e) => { eprintln!("error: `{}`: {e}", cc.display()); exit(1) },
        }
    }
    else {
        #[cfg(feature = "compiler")]
        { rame_driver::compile_tmp(p) }

        #[cfg(not(feature = "compiler"))]
        unreachable!("no compiler specified")
    };

    let code = match RoCode::<i32>::parse(ram) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: failed to parse `{}`: {e}", path.display());
            exit(1);
        }
    };

    // `cli.compiler` use the filename instead of a fd
    _ = fs::remove_file(&path);

    #[cfg(feature = "optimizer")]
    let _ = io::stdout().flush();

    #[cfg(feature = "optimizer")]
    let opt = rame::optimizer::SeqRewriter::from(&code).optimize().rewritten();

    print!("{}... ", p.display());
    let mut ok = true;

    for test in tests {
        if let Some(out) = test.run(code.clone()) {
            if ok {
                println!("failed");
                ok = false;
            }

            eprintln!(" {test}: got {out:?} instead");
            continue;
        }

        #[cfg(feature = "optimizer")]
        assert!(test.run(opt.clone()).is_none(), "optimizer check");
    }

    if ok {
        println!("ok");
    }
}

fn main() {
    let cli = Cli::parse();
    cli.infiles.iter().for_each(|p| scan_file(&p, &cli.compiler));
}
