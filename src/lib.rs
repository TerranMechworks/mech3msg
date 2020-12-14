use std::ffi::CStr;
use std::os::raw::c_char;

const INVALID: i32 = 0;

include!(concat!(env!("OUT_DIR"), "/lookup.rs"));

#[no_mangle]
pub extern "C" fn ZLocGetID(ptr: *const c_char) -> i32 {
    if ptr.is_null() {
        return INVALID;
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    match cstr.to_str() {
        Ok(key) => LOOKUP
            .binary_search_by_key(&key, |&(key, _mid)| &key)
            .map(|index| LOOKUP[index].1)
            .unwrap_or(INVALID),
        Err(_) => INVALID,
    }
}
