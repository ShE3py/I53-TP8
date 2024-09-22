use clap::{Parser, ValueHint};
use rame::model::{Integer, RoCode};
use rame::optimizer::SeqRewriter;
use rame::runner::Ram;
use rame_driver::compile_tmp;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;

/// Run an algorithmic or RAM program.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    /// The program to run.
    #[arg(value_name = "infile", value_hint = ValueHint::FilePath)]
    infile: PathBuf,
}

struct UnitTest<T: Integer> {
    input: Vec<T>,
    output: Vec<T>,
}

impl<T: Integer> UnitTest<T> {
    fn run(&self, code: RoCode<T>) {
        let ram = Ram::new(code, self.input.iter().copied());
        assert_eq!(ram.run().as_slice(), &self.output);
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
        v.push(T::from_str(elem)?);
    }
    
    Ok(v)
}

fn main(){
    let cli = Cli::parse();
    
    let tests = parse_headers::<i16>(&cli.infile);
    if tests.is_empty() {
        eprintln!("{}: no test", cli.infile.display());
        exit(1);
    }
    
    let (f, _) = compile_tmp(&cli.infile);
    let code = RoCode::<i16>::parse(f).unwrap();
    let opt = SeqRewriter::from(&code).optimize().rewritten();
    
    print!("{}... ", cli.infile.display());
    for test in tests {
        test.run(code.clone());
        test.run(opt.clone());
    }
    println!("ok");
}
