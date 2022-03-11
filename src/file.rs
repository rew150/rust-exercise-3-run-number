use std::ffi::{CString, NulError, IntoStringError};

mod C {
    use std::os::raw::c_char;
    use std::os::raw::c_int;

    // opaque struct FILE
    #[repr(C)]
    pub struct FILE {
        _data: [u8; 0],
        _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
    }
    // opaque struct fpos_t
    #[repr(C)]
    pub struct fpos_t {
        _data: [u8; 0],
        _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
    }

    #[cfg(all(target_env = "gnu"))]
    #[link(name = "stdc++")]
    extern {
        pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
        pub fn fputs(str: *const c_char, stream: *mut FILE) -> c_int;
        pub fn fclose(stream: *mut FILE) -> c_int;
        pub fn fgets(string: *mut c_char, num: c_int, stream: *mut FILE) -> *mut c_char;
        pub fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int;
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // CString error
    #[error(transparent)]
    CStringNul(#[from] NulError),
    #[error(transparent)]
    CStringIntoString(#[from] IntoStringError),
    // C file api error
    #[error("cannot fopen")]
    FileOpen,
    #[error("cannot write to file: {0}")]
    FileWrite(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct CFileHandler {
    _fhandler: *mut C::FILE,
    _marker: std::marker::PhantomData<C::FILE>,
}

impl CFileHandler {
    pub fn puts(&self, string: &str) -> Result<()> {
        let string = CString::new(string)?;
        let string = string.as_ptr();
        unsafe {
            let res = C::fputs(string, self._fhandler);
            if res > 0 {
                Ok(())
            } else {
                Err(Error::FileWrite("fputs"))
            }
        }
    }

    pub fn gets(&self, max_chars: u16) -> Result<String> {
        todo!();
        let mut cstr = CString::default();
        cstr.into_string().map_err(Into::into)
    }
}

impl Drop for CFileHandler {
    fn drop(&mut self) {
        unsafe {
            C::fclose(self._fhandler);
        }
    }
}

pub fn open_file<'a, 'b>(filename: &'a str, mode: &'b str) -> Result<CFileHandler> {
    let filename = CString::new(filename)?;
    let filename = filename.as_ptr();
    let mode = CString::new(mode)?;
    let mode = mode.as_ptr();
    unsafe {
        let _fhandler = C::fopen(filename, mode);
        if _fhandler.is_null() {
            Err(Error::FileOpen)
        } else {
            Ok(CFileHandler {
                _fhandler,
                _marker: std::marker::PhantomData
            })
        }
    }
}