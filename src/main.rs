use std::ffi::CString;
use std::os::raw::c_char;

#[cfg(all(target_env = "gnu"))]
#[link(name = "stdc++")]
extern {
    fn printf(format: *const c_char) -> libc::c_int;
}

fn main() {
    let string = CString::new("Hello, FFI!\n").unwrap();
    unsafe {
        printf(string.as_ptr());
    }
}
