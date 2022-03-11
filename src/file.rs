use std::{
    ffi::{
        CString, NulError, IntoStringError,
    },
    os::raw::{
        c_int, c_char,
    },
    string::{
        FromUtf8Error,
    }, borrow::Cow,
};

mod C {
    use std::marker::{PhantomData,PhantomPinned};
    use std::os::raw::c_char;
    use std::os::raw::c_int;

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
        pub fn fputs(str: *const c_char, stream: *mut FILE) -> c_int;
        pub fn fclose(stream: *mut FILE) -> c_int;
        pub fn fgets(string: *mut c_char, num: c_int, stream: *mut FILE) -> *mut c_char;
        pub fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int;
        pub fn feof(stream: *mut FILE) -> c_int;
        pub fn ferror(stream: *mut FILE) -> c_int;
        pub fn clearerr(stream: *mut FILE);

        // non std lib
        pub fn allocate_fpos_t() -> *mut fpos_t;
        pub fn deallocate_fpos_t(ptr: *mut fpos_t);
    }

    pub struct fpos_heap {
        ptr: *mut fpos_t,
        _marker: PhantomData<fpos_t>,
    }

    impl fpos_heap {
        pub fn new() -> Self {
            unsafe {
                fpos_heap {
                    ptr: allocate_fpos_t(),
                    _marker: PhantomData
                }
            }
        }

        pub fn as_ptr(&mut self) -> *mut fpos_t {
            self.ptr
        }
    }

    impl Drop for fpos_heap {
        fn drop(&mut self) {
            unsafe {
                deallocate_fpos_t(self.ptr);
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error<'a> {
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
    FileWrite(Cow<'a, str>),
    #[error("cannot read from file: {0}")]
    FileRead(Cow<'a, str>),
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

pub struct CFileHandler {
    _fhandler: *mut C::FILE,
    _marker: std::marker::PhantomData<C::FILE>,
}

impl CFileHandler {
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
                return Err(Error::FileRead(format!("gets {}", err).into()))
            }

            let reached_eof = C::feof(self._fhandler) != 0;

            let len = buff
                .iter()
                .position(|&c| c == 0)
                .expect("ffi: fgets() buffer overflow");
            buff.truncate(len);

            String::from_utf8(buff)
                .map(|s| (s, reached_eof))
                .map_err(|e| e.into())
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

pub fn open_file<'a, 'b, 'c>(filename: &'a str, mode: &'b str) -> Result<'c, CFileHandler> {
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