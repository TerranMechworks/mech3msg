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

#[cfg(all(target_os = "windows", target_env = "msvc"))]
mod msvc {
    // With MSVC, for some reason this is required. This is what std does:
    // https://github.com/rust-lang/libc/blob/a016994b91a5b0dbd8f234b60af936eedb227b22/src/windows/mod.rs#L253

    // Linking to "libcmt" works, but produces a large DLL and imports "KERNEL32.DLL"
    // Linking to "msvcrt" does not work; _memcmp is not found
    // Linking to "vcruntime" works, and produces a small DLL and imports "VCRUNTIME140.DLL" (with /NOENTRY)
    // The dependency on the Microsoft Visual C++ Redistributable for Visual
    // Studio 2015, 2017 and 2019 (vc_redist.x86.exe) is unfortunate, but probably
    // the least bad option.

    #[link(name = "vcruntime")]
    extern "C" {}
}

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
