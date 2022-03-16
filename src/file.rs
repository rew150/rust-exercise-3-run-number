use std::{
    ffi::{
        CString, NulError, IntoStringError, c_void
    },
    os::raw::{
        c_int, c_char,
    },
    string::{
        FromUtf8Error,
    }, borrow::Cow, mem::size_of,
};

use self::C::wrapped_fpos;

mod C {
    use std::marker::{PhantomData,PhantomPinned};
    use std::os::raw::c_char;
    use std::os::raw::c_int;

    use libc::{c_void, size_t};

    // opaque struct FILE
    #[repr(C)]
    pub struct FILE {
        _data: [u8; 0],
        _marker: PhantomData<(*mut u8, PhantomPinned)>,
    }
    // opaque struct fpos_t
    #[repr(C)]
    pub struct fpos_t {
        _data: [u8; 0],
        _marker: PhantomData<(*mut u8, PhantomPinned)>,
    }

    extern {
        pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE;
        pub fn fclose(stream: *mut FILE) -> c_int;
        pub fn fputs(str: *const c_char, stream: *mut FILE) -> c_int;
        pub fn fgets(string: *mut c_char, num: c_int, stream: *mut FILE) -> *mut c_char;
        pub fn fwrite(buffer: *const c_void, size: size_t, count: size_t, stream: *mut FILE) -> size_t;
        pub fn fread(buffer: *mut c_void, size: size_t, count: size_t, stream: *mut FILE) -> size_t;
        pub fn fgetc(stream: *mut FILE) -> c_int;
        pub fn fflush(stream: *mut FILE) -> c_int;
        pub fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int;
        pub fn fsetpos(stream: *mut FILE, pos: *const fpos_t) -> c_int;
        pub fn feof(stream: *mut FILE) -> c_int;
        pub fn ferror(stream: *mut FILE) -> c_int;
        pub fn clearerr(stream: *mut FILE);

        // non std lib
        pub fn allocate_fpos_t() -> *mut fpos_t;
        pub fn deallocate_fpos_t(ptr: *mut fpos_t);
        pub fn copy_fpos_t(dst: *mut fpos_t, src: *const fpos_t);
    }

    pub struct wrapped_fpos {
        ptr: *mut fpos_t,
        _marker: PhantomData<fpos_t>,
    }

    impl wrapped_fpos {
        pub fn new() -> Self {
            unsafe {
                wrapped_fpos {
                    ptr: allocate_fpos_t(),
                    _marker: PhantomData
                }
            }
        }

        pub fn clone_from_ptr(src: *mut fpos_t) -> Self {
            let res = wrapped_fpos::new();
            unsafe {
                copy_fpos_t(res.ptr, src);
            }
            res
        }

        pub fn as_mut_ptr(&mut self) -> *mut fpos_t {
            self.ptr
        }

        pub fn as_ptr(&self) -> *const fpos_t {
            self.ptr
        }
    }

    impl Clone for wrapped_fpos {
        fn clone(&self) -> wrapped_fpos {
            wrapped_fpos::clone_from_ptr(self.ptr)
        }
    }

    impl Drop for wrapped_fpos {
        fn drop(&mut self) {
            unsafe {
                deallocate_fpos_t(self.ptr);
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // CString error
    #[error(transparent)]
    CStringNul(#[from] NulError),
    // String error
    #[error(transparent)]
    StringFromUTF8(#[from] FromUtf8Error),
    #[error(transparent)]
    CStringIntoString(#[from] IntoStringError),
    // C file api error
    #[error("cannot fopen")]
    FileOpen,
    #[error("cannot write to file: {0}")]
    FileWrite(String),
    #[error("cannot read from file: {0}")]
    FileRead(String),
    #[error("cannot get pos: {0}")]
    FileGetPos(String),
    #[error("cannot set pos: {0}")]
    FileSetPos(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct CFileHandler {
    _fhandler: *mut C::FILE,
    _marker: std::marker::PhantomData<C::FILE>,
}

impl CFileHandler {
    pub fn err_ind(&self) -> bool {
        unsafe {
            C::ferror(self._fhandler) != 0
        }
    }

    pub fn eof_ind(&self) -> bool {
        unsafe {
            C::feof(self._fhandler) != 0
        }
    }

    pub fn puts(&mut self, string: &str) -> Result<()> {
        let string = CString::new(string)?;
        let string = string.as_ptr();
        unsafe {
            let res = C::fputs(string, self._fhandler);
            if res > 0 {
                Ok(())
            } else {
                Err(Error::FileWrite("fputs".into()))
            }
        }
    }

    pub fn gets(&mut self, max_chars: c_int) -> Result<(String, bool)> {
        assert!(max_chars > 0, "gets max_chars must exceeds 0");

        unsafe {
            // clear error, eof flags
            C::clearerr(self._fhandler);

            let mut buff = vec![0u8; max_chars as usize];
            let ptr = buff.as_mut_ptr() as *mut i8;

            C::fgets(ptr, max_chars, self._fhandler);

            let err = C::ferror(self._fhandler);
            if err != 0 {
                return Err(Error::FileRead(format!("gets {}", err)))
            }

            let len = buff
                .iter()
                .position(|&c| c == 0)
                .expect("ffi: fgets() buffer overflow");
            buff.truncate(len);

            String::from_utf8(buff)
                .map(|s| (s, self.eof_ind()))
                .map_err(|e| e.into())
        }
    }

    pub fn write_flush<T>(&mut self, data: &[T]) -> usize {
        unsafe {
            let written = C::fwrite(data.as_ptr() as *const c_void, size_of::<T>(), data.len(), self._fhandler);
            C::fflush(self._fhandler);
            written
        }
    }

    pub fn read_until_char(&self, start_pos: C::wrapped_fpos, until: u8) -> Result<(String, bool)> {
        let mut res = vec![];
        unsafe {
            // clear error, eof flags
            C::clearerr(self._fhandler);

            let setposres = C::fsetpos(self._fhandler, start_pos.as_ptr());
            if setposres != 0 {
                return Err(Error::FileSetPos(format!("{}", setposres)))
            }

            let mut c;
            while {
                c = C::fgetc(self._fhandler) as u8;
                !self.err_ind() && !self.eof_ind()
            } {
                res.push(c);
                if c == until {
                    break;
                }
            }

            let err = C::ferror(self._fhandler);
            if err != 0 {
                return Err(Error::FileRead(format!("{}", err)));
            }
        }
        String::from_utf8(res)
            .map(|s| (s, self.eof_ind()))
            .map_err(|e| e.into())
    }

    pub fn current_pos(&self) -> Result<wrapped_fpos> {
        unsafe {
            let mut pos = wrapped_fpos::new();
            let err = C::fgetpos(self._fhandler, pos.as_mut_ptr());
            if err == 0 {
                Ok(pos)
            } else {
                Err(Error::FileGetPos(format!("{}", err)))
            }
        }
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