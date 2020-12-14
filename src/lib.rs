#![no_std]
#![feature(core_intrinsics)]
use core::ptr::slice_from_raw_parts;

const INVALID: i32 = 0;

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

#[inline]
fn strlen(ptr: *const u8) -> usize {
    let mut n = 0;
    unsafe {
        while *ptr.offset(n) != 0 {
            n += 1;
        }
    }
    n as usize
}

#[no_mangle]
pub extern "C" fn ZLocGetID(ptr: *const u8) -> i32 {
    if ptr.is_null() {
        return INVALID;
    }
    let len = strlen(ptr);
    let slice = slice_from_raw_parts(ptr, len);
    let loc = unsafe { &*slice };
    LOOKUP
        .binary_search_by_key(&loc, |&(key, _)| &key)
        .map(|index| LOOKUP[index].1)
        .unwrap_or(INVALID)
}

#[cfg(not(test))]
mod not_test {
    // The test framework will automatically pull in std, and these would conflict
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        core::intrinsics::abort()
    }
    #[no_mangle]
    pub extern "C" fn rust_eh_register_frames() {}
    #[no_mangle]
    pub extern "C" fn rust_eh_unregister_frames() {}
}

#[cfg(test)]
mod test {
    // WARNING: The tests will only be run-able on Windows (since an exe will be
    // produced when compiling). On other platforms, it's also usual to encounter
    // linking errors (e.g. `_Unwind_Resume`), because of the exception handling
    // (DWARF-2 vs SJLJ vs SEH).
    use super::*;

    #[test]
    fn nullptr() {
        assert_eq!(ZLocGetID(core::ptr::null()), 0);
    }

    #[test]
    fn empty_str() {
        let empty = [0u8];
        assert_eq!(ZLocGetID(empty.as_ptr()), 0);
    }

    #[test]
    fn not_found() {
        let empty = b"MSG_FOO\0";
        assert_eq!(ZLocGetID(empty.as_ptr()), 0);
    }

    #[test]
    fn found_first() {
        let empty = b"MSG_BACK\0";
        assert_eq!(ZLocGetID(empty.as_ptr()), 1);
    }

    #[test]
    fn found_last() {
        // this is the last one in v1.0 only, e.g. v1.2 adds messages after
        let empty = b"MSG_NOTEXTURE_MEMORY\0";
        assert_eq!(ZLocGetID(empty.as_ptr()), 4031);
    }
}
