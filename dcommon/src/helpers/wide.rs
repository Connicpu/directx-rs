use std::ffi::OsString;

use std::ptr::NonNull;

use winapi::um::combaseapi::{CoTaskMemAlloc, CoTaskMemFree};
use wio::wide::FromWide;

pub unsafe fn wstrlen(mut pwstr: *const u16) -> usize {
    let mut len = 0;
    while *pwstr != 0 {
        len += 1;
        pwstr = pwstr.offset(1);
    }
    len
}

/// Represents a known-width UTF16/UCS-2 borrowed string
pub struct WideStr<'a> {
    /// The array of character units
    pub data: &'a [u16],
}

impl<'a> WideStr<'a> {
    /// Convert this value to an OsString.
    pub fn to_os_string(&self) -> OsString {
        OsString::from_wide(self.data)
    }

    /// Attempt to convert this string to UTF-8. Will replace bad codepoints with the question
    /// mark diamond.
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.data)
    }
}

impl<'a> WideStr<'a> {
    /// Construct a WideStr from a pointer to an array and its length.
    pub unsafe fn from_raw(ptr: *const u16, len: usize) -> WideStr<'a> {
        WideStr {
            data: std::slice::from_raw_parts(ptr, len),
        }
    }
}

#[repr(C)]
/// A c-style wide string.
pub struct WideCStr {
    dummy: u16,
}

impl WideCStr {
    /// Convert this value to an OsString.
    pub fn to_os_string(&self) -> OsString {
        unsafe {
            let ptr = self.as_ptr();
            let len = wstrlen(ptr);
            let slice = std::slice::from_raw_parts(ptr, len);
            OsString::from_wide(slice)
        }
    }

    /// Attempt to convert this string to UTF-8. Will replace bad codepoints with the question
    /// mark diamond.
    pub fn to_string_lossy(&self) -> String {
        unsafe {
            let ptr = self.as_ptr();
            let len = wstrlen(ptr);
            let slice = std::slice::from_raw_parts(ptr, len);
            String::from_utf16_lossy(slice)
        }
    }
}

impl WideCStr {
    /// Construct the value from a c-style string pointer.
    pub unsafe fn from_ptr<'a>(ptr: *const u16) -> &'a WideCStr {
        &*(ptr as *const WideCStr)
    }

    /// Convert this back to a c-style string pointer.
    pub unsafe fn as_ptr(&self) -> *const u16 {
        self as *const WideCStr as *const u16
    }
}

#[repr(transparent)]
pub struct CoTaskWString {
    ptr: NonNull<u16>,
}

impl CoTaskWString {
    pub fn create(data: &[u16]) -> CoTaskWString {
        unsafe {
            let size = (data.len() + 1) * 2;
            let mem = CoTaskMemAlloc(size) as *mut u16;
            if mem.is_null() {
                eprintln!("oom");
                std::process::abort();
            }
            std::ptr::copy_nonoverlapping(data.as_ptr(), mem, data.len());
            *mem.offset(data.len() as isize) = 0;
            CoTaskWString {
                ptr: NonNull::new_unchecked(mem),
            }
        }
    }

    pub fn len(&self) -> usize {
        unsafe { wstrlen(self.ptr.as_ptr()) }
    }

    pub fn data(&self) -> &[u16] {
        let len = self.len();
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), len) }
    }
}

impl Clone for CoTaskWString {
    fn clone(&self) -> Self {
        CoTaskWString::create(self.data())
    }
}

impl Drop for CoTaskWString {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.ptr.as_ptr() as _)
        }
    }
}
