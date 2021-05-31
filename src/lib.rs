#![no_std]
#![feature(core_intrinsics)]
use core::ptr::slice_from_raw_parts;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}
#[no_mangle]
pub extern "C" fn rust_eh_register_frames() {}
#[no_mangle]
pub extern "C" fn rust_eh_unregister_frames() {}

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

const INVALID_ID: i32 = 0;

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
        return INVALID_ID;
    }
    let len = strlen(ptr);
    let slice = slice_from_raw_parts(ptr, len);
    let loc = unsafe { &*slice };
    LOOKUP
        .binary_search_by_key(&loc, |&(key, _)| &key)
        .map(|index| LOOKUP[index].1)
        .unwrap_or(INVALID_ID)
}
