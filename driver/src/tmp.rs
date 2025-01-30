use std::ffi::{c_char, c_int, CString, OsStr};
use std::fs::File;
use std::io;
use std::os::fd::{FromRawFd as _, OwnedFd};
use std::os::unix::ffi::OsStrExt as _;
use std::path::Path;

#[derive(Debug)]
pub struct TempFile {
    pub file: File,
    pub path: CString,
}

impl AsRef<Path> for TempFile {
    fn as_ref(&self) -> &Path {
        Path::new(OsStr::from_bytes(self.path.as_bytes_with_nul()))
    }
}

impl TempFile {
    pub fn new<P: AsRef<Path>>(model: P) -> TempFile {
        let prefix = model.as_ref().file_stem().unwrap_or(OsStr::new("a")).as_bytes();
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
        let path = unsafe { CString::from_vec_with_nul_unchecked(template) };

        TempFile {
            file: File::from(fd),
            path,
        }
    }
}
