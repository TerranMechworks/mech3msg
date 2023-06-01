#![no_std]

// this may also work with `target_env = "msvc"`?
#[cfg(all(target_os = "windows", target_env = "gnu"))]
#[link(name = "msvcrt")]
extern "C" {
    fn abort() -> !;
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { abort() }
}

#[no_mangle]
pub extern "C" fn rust_eh_register_frames() {}
#[no_mangle]
pub extern "C" fn rust_eh_unregister_frames() {}

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

fn strlen(ptr: *const u8) -> usize {
    let mut n = 0;
    while unsafe { *ptr.offset(n) } != 0 {
        n += 1;
    }
    n as usize
}

const INVALID_ID: i32 = 0;

#[no_mangle]
pub extern "C" fn ZLocGetID(ptr: *const u8) -> i32 {
    if ptr.is_null() {
        return INVALID_ID;
    }
    let len = strlen(ptr);
    let slice = core::ptr::slice_from_raw_parts(ptr, len);
    let loc = unsafe { &*slice };
    LOOKUP
        .binary_search_by_key(&loc, |&(key, _)| &key)
        .ok()
        .map(|index| LOOKUP[index].1)
        .unwrap_or(INVALID_ID)
}
