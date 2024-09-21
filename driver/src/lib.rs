use std::ffi::{c_char, c_int, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::exit;

pub fn compile<P: AsRef<Path>>(filename: P) {
    extern "C" {
        fn arc_compile_file(filename: *const c_char) -> c_int;
    }
    
    let f = CString::new(filename.as_ref().as_os_str().as_bytes()).unwrap();
    let ret = unsafe { arc_compile_file(f.as_ptr()) };
    if ret != 0 {
        exit(ret);
    }
}
