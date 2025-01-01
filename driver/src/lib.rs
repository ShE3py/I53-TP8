use clap::ValueEnum;
use rame::model::{Integer, RoCode};
use std::ffi::{c_char, c_int, CString, OsStr};
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::exit;

#[cfg(feature = "compiler")]
use std::{
    ffi::OsString,
    io::Seek,
    os::fd::{RawFd, IntoRawFd},
    path::PathBuf
};

mod stdin;

pub use stdin::Stdin;

#[cfg(feature = "compiler")]
extern "C" {
    pub fn arc_compile_file(infile: *const c_char, outfile: *const c_char);
    pub fn arc_compile_file_fd(infile: *const c_char, outpath: *const c_char, outfile: RawFd);
}

pub fn create_temp_out(model: &Path) -> (File, CString) {
    let prefix = model.file_stem().unwrap_or(OsStr::new("a")).as_bytes();
    let infix = OsStr::new("XXXXXX").as_bytes();
    let suffix = OsStr::new("tmp").as_bytes();
    
    let mut buf = Vec::with_capacity(prefix.len() + '.'.len_utf8() + infix.len() + '.'.len_utf8() + suffix.len() + '\0'.len_utf8());
    buf.extend_from_slice(prefix);
    buf.push(b'.');
    buf.extend_from_slice(infix);
    buf.push(b'.');
    buf.extend_from_slice(suffix);
    buf.push(b'\0');
    
    // SAFETY: `c_outfile` was `Ok` so no null interior byte, we just
    //  pushed a null byte as the last element.
    let template = unsafe { CString::from_vec_with_nul_unchecked(buf) };
    
    // The roundtrip is for Miri.
    let mut template = template.into_bytes();
    
    // More checks.
    let suffix_len: c_int = ('.'.len_utf8() + suffix.len()).try_into().expect("file extension too large");
    debug_assert_eq!(&template[template.len() - suffix_len as usize - 6..template.len() - suffix_len as usize], b"XXXXXX");
    
    // SAFETY: `template` is writable, we have `prefixXXXXXXsuffix`
    let fd = unsafe {
        libc::mkstemps(
            template.as_mut_ptr() as *mut c_char,
            suffix_len,
        )
    };
    
    if fd == -1 {
        panic!("mkstemps: {}", io::Error::last_os_error());
    }
    
    // SAFETY: inherently safe.
    unsafe {
        libc::unlink(
            template.as_ptr() as *const c_char,
        );
    }
    
    // SAFETY: the fd is ours.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    
    // SAFETY: `mkstemps` should have preserved our safety rules
    let c_template = unsafe { CString::from_vec_with_nul_unchecked(template) };
    
    (File::from(fd), c_template)
}

/// Compiles the algorithmic program `infile` into `outfile`.
#[cfg(feature = "compiler")]
pub fn compile<P: AsRef<Path>, Q: AsRef<Path>>(infile: P, outfile: Q, optimize: bool) {
    let outfile = outfile.as_ref();
    let c_infile = CString::new(infile.as_ref().as_os_str().as_bytes()).expect("infile");
    
    if !optimize {
        let c_outfile = CString::new(outfile.as_os_str().as_bytes()).expect("outfile");
        unsafe { arc_compile_file(c_infile.as_ptr(), c_outfile.as_ptr()) };
    }
    else {
        let (mut f, c_intermediate) = create_temp_out(outfile);
        
        // Compile into an intermediate tempfile
        unsafe { arc_compile_file_fd(c_infile.as_ptr(), c_intermediate.as_ptr(), f.try_clone().expect("cloning tempfile").into_raw_fd()) };
        
        // Optimize the artifact.
        f.rewind().expect("rewind");
        crate::optimize(f, OsStr::from_bytes(c_intermediate.as_bytes_with_nul()).as_ref(), Some(outfile));
    }
}

/// Compiles the algorithmic program `infile` into a tempfile.
#[cfg(feature = "compiler")]
pub fn compile_tmp<P: AsRef<Path>>(infile: P) -> (File, PathBuf) {
    let infile = infile.as_ref();
    let c_infile = CString::new(OsStrExt::as_bytes(infile.as_os_str())).expect("infile");

    let (mut f, c_intermediate) = create_temp_out(infile);

    // Compile into an intermediate tempfile
    unsafe { arc_compile_file_fd(c_infile.as_ptr(), c_intermediate.as_ptr(), f.try_clone().expect("cloning tempfile").into_raw_fd()) };

    // SAFETY: unix
    let tmppath = PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(c_intermediate.into_bytes()) });

    f.rewind().expect("rewind");
    (f, tmppath)

}

/// Optimize the RAM program `infile` into `outfile`.
#[cfg_attr(not(feature = "optimizer"), doc(hidden))]
pub fn optimize<Q: AsRef<Path>>(infile: File, inpath: &Path, outfile: Option<Q>) -> RoCode<i64> {
    let incode = parse::<i64>(infile, inpath);

    #[cfg(feature = "optimizer")] {
        use rame::optimizer::SeqRewriter;

        let outcode = SeqRewriter::from(&incode).optimize().rewritten();

        if let Some(outfile) = outfile {
            if let Err(e) = outcode.write_to_file(outfile) {
                eprintln!("error: unable to save optimized code: {e}");
                exit(1);
            }
        }

        outcode
    }

    #[cfg(not(feature = "optimizer"))] {
        eprintln!("warning: tried to optimize while the optimizer is opted out");

        if let Some(outpath) = outfile {
            if let Err(e) = std::fs::copy(inpath, outpath) {
                eprintln!("error: copy failed: {e}");
                exit(1);
            }
        }

        incode
    }
}

/// Open a file, handling potential errors.
#[must_use]
pub fn open(inpath: &Path) -> File {
    match File::open(inpath) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {e}", &inpath.display());
            exit(1);
        },
    }
}

/// Parse a RAM program, handling potential errors.
#[must_use]
pub fn parse<T: Integer>(infile: File, inpath: &Path) -> RoCode<T> {
    match RoCode::<T>::parse(infile) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}: {e}", inpath.display());
            exit(1);
        }
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

pub fn cvt<T: Integer + TryFrom<i128, Error: Display>>(args: Vec<i128>, _ty: &RoCode<T>) -> Vec<T> {
    args.into_iter().map(|v| match T::try_from(v) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid integer {v}: {e}");
            exit(1);
        }
    }).collect()
}

#[macro_export]
macro_rules! monomorphize {
    ($inpath:expr, $compile:expr, $bits:expr, $code:ident, $body:tt) => {
        let inpath: &::std::path::Path = $inpath;

        let (infile, inpath) = {
            #[cfg(feature = "compiler")]
            if $compile {
                let (tmpfile, tmppath) = $crate::compile_tmp(inpath);
                (tmpfile, ::std::borrow::Cow::Owned(tmppath))
            }
            else {
                ($crate::open(inpath), ::std::borrow::Cow::Borrowed(inpath))
            }

            #[cfg(not(feature = "compiler"))]
            ($crate::open(inpath), ::std::borrow::Cow::Borrowed(inpath))
        };

        match $bits {
            Bits::Int8 => {
                let $code = $crate::parse::<i8>(infile, &*inpath);
                $body
            },
            Bits::Int16 => {
                let $code = $crate::parse::<i8>(infile, &*inpath);
                $body
            },
            Bits::Int32 => {
                let $code = $crate::parse::<i8>(infile, &*inpath);
                $body
            },
            Bits::Int64 => {
                let $code = $crate::parse::<i8>(infile, &*inpath);
                $body
            },
            Bits::Int128 => {
                let $code = $crate::parse::<i8>(infile, &*inpath);
                $body
            },
        }
    };
}
