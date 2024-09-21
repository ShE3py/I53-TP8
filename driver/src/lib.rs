use std::ffi::{c_char, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

mod stdin;

pub use stdin::Stdin;

pub fn compile<P: AsRef<Path>, Q: AsRef<Path>>(infile: P, outfile: Q) {
    extern "C" {
        fn arc_compile_file(infile: *const c_char, outfile: *const c_char);
    }
    
    let infile = CString::new(infile.as_ref().as_os_str().as_bytes()).unwrap();
    let outfile = CString::new(outfile.as_ref().as_os_str().as_bytes()).unwrap();
    
    unsafe { arc_compile_file(infile.as_ptr(), outfile.as_ptr()) };
}
