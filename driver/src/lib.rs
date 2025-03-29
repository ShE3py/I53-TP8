use clap::ValueEnum;
use rame::model::{Integer, ParseCodeError, RoCode};
use std::ffi::{c_char, CString};
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

mod stdin;
mod tmp;

pub use stdin::Stdin;
pub use tmp::TempFile;

#[cfg(feature = "compiler")]
extern "C" {
    pub fn arc_compile_file(infile: *const c_char, outfile: *const c_char);
}

/// Builder pattern for [`RoCode`].
#[derive(Debug, Default)]
#[must_use]
pub struct Driver {
    infile: Option<PathBuf>,
    outfile: Option<PathBuf>,
    compile: bool,
    compiler: Option<PathBuf>,
    optimize: bool,
}

impl Driver {
    pub fn new() -> Driver {
        Driver::default()
    }

    pub fn infile(&mut self, infile: &Path) -> &mut Self {
        self.infile = Some(infile.to_owned()).filter(|infile| infile.as_os_str() != "-");
        self
    }

    pub fn outfile(&mut self, outfile: &Path) -> &mut Self {
        self.outfile = Some(outfile.to_owned());
        self
    }

    pub fn compile(&mut self, compile: bool) -> &mut Self {
        self.compile = compile;
        self
    }

    pub fn compiler(&mut self, compiler: Option<&PathBuf>) -> &mut Self {
        self.compiler = compiler.cloned();
        self
    }

    pub fn optimize(&mut self, optimize: bool) -> &mut Self {
        self.optimize = optimize;
        self
    }

    pub fn try_drive(&self) -> Result<RoCode<i128>, ParseCodeError<i128>> {
        let infile = self.infile.as_ref().map(|pb| pb.as_path());
        let outfile = self.outfile.as_ref().map(|pb| pb.as_path());

        let code = if !self.compile {
            // The code is already in RAM format;
            infile.map_or_else(
                || RoCode::try_from(Stdin::new(|i| print!("{i} | ")).fuse().collect::<Vec<_>>()),
                |path| RoCode::parse(open(path))
            )
        }
        else {
            // Compile algorithmic code to RAM/LLVM.
            let create_temp_file = || TempFile::new(outfile.or(infile).unwrap_or("stdin".as_ref()));
            
            let stdin = infile.is_none().then(|| {
                let temp_file = create_temp_file();
                let mut writer = BufWriter::new(&temp_file.file);
                if let Err(e) = io::copy(&mut io::stdin().lock(), &mut writer) {
                    eprintln!("error: failed to read stdin: {e}");
                    exit(1);
                }
                
                drop(writer);
                temp_file
            });
            
            let infile = infile.unwrap_or_else(|| stdin.as_ref().unwrap().as_ref());

            let unoptimized = self.optimize.then(create_temp_file);
            let outfile = unoptimized.as_ref().map(|tf| tf.as_ref()).or(outfile);
            let compiled = outfile.is_none().then(create_temp_file);
            let compiled = outfile.unwrap_or_else(|| compiled.as_ref().unwrap().as_ref());

            match self.compiler.as_ref() {
                Some(cc) => {
                    let mut cmd = Command::new(cc);
                    cmd.arg(infile).arg("-o").arg(compiled);

                    match cmd.status() {
                        Ok(s) if s.success() => {},
                        Ok(s) => { eprintln!("error: {cmd:?}: {s}"); exit(1) },
                        Err(e) => { eprintln!("error: `{}`: {e}", cc.display()); exit(1) },
                    }
                },

                None => {
                    let infile = to_cstring(infile);
                    let outfile = to_cstring(compiled);

                    unsafe { arc_compile_file(infile.as_ptr(), outfile.as_ptr()) };
                }
            }
            
            Ok(RoCode::default()) // LLVM backend
            //RoCode::parse(open(compiled))
        };

        if !self.optimize {
            return code;
        };

        let optimized = code?.optimize();

        if let Some(outfile) = outfile {
            if let Err(e) = optimized.write_to_file(outfile) {
                eprintln!("{}: {}: {e}", env!("CARGO_PKG_NAME"), outfile.display());
                exit(1);
            }
        }

        Ok(optimized)
    }

    pub fn drive(&self) -> RoCode<i128> {
        match self.try_drive() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}: {e}", env!("CARGO_PKG_NAME"));
                exit(1);
            },
        }
    }
}

/// Open a file, handling potential errors.
#[must_use]
pub fn open<P: AsRef<Path>>(path: P) -> File {
    let path = path.as_ref();
    match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {}: {e}", env!("CARGO_PKG_NAME"), path.display());
            exit(1);
        },
    }
}

/// Converts a [`Path`] into a [`CString`].
#[must_use]
pub fn to_cstring<P: AsRef<Path>>(path: P) -> CString {
    let path = path.as_ref();
    match CString::new(OsStrExt::as_bytes(path.as_os_str())) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {}: {e}", env!("CARGO_PKG_NAME"), path.display());
            exit(1);
        },
    }
}

/// How many bits should the program be run with.
#[derive(ValueEnum, Copy, Clone, Debug, Default)]
pub enum Bits {
    #[clap(name = "8")] Int8,
    #[clap(name = "16")] #[default] Int16,
    #[clap(name = "32")] Int32,
    #[clap(name = "64")] Int64,
    #[clap(name = "128")] Int128,
}

/// Convert CLI args
pub fn cvt<T: Integer + TryFrom<i128, Error: Display>>(args: &[i128]) -> Vec<T> {
    args.iter().copied().map(|v| match T::try_from(v) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid integer {v}: {e}");
            exit(1);
        }
    }).collect()
}
