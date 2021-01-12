use super::patch::patch_iat;
use winapi;
use winapi::shared::minwindef::{DWORD, HINSTANCE, LPVOID};
use winapi::shared::ntdef::LARGE_INTEGER;

const KERNEL32: &[u8] = b"kernel32.dll";
const GET_TICK_COUNT: &[u8] = b"GetTickCount";

static mut FREQ: u64 = 0;

extern "system" fn get_tick_count() -> DWORD {
    let mut count: u64 = 0;
    let ptr = unsafe { core::mem::transmute::<&mut u64, &mut LARGE_INTEGER>(&mut count) };
    unsafe {
        winapi::um::profileapi::QueryPerformanceCounter(ptr);
    };
    count *= 1000;
    unsafe {
        count /= FREQ;
    };
    count as DWORD
}

extern "system" fn setup(_lp_thread_parameter: LPVOID) -> DWORD {
    let freq_ptr = unsafe { core::mem::transmute::<&mut u64, &mut LARGE_INTEGER>(&mut FREQ) };
    unsafe {
        winapi::um::profileapi::QueryPerformanceFrequency(freq_ptr);
    };
    let func_ptr =
        unsafe { core::mem::transmute::<extern "system" fn() -> DWORD, DWORD>(get_tick_count) };
    unsafe { patch_iat(KERNEL32, GET_TICK_COUNT, func_ptr) };
    0
}

#[no_mangle]
pub extern "stdcall" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> i32 {
    let res = match fdw_reason {
        winapi::um::winnt::DLL_PROCESS_ATTACH => {
            unsafe {
                // offload the setup to a separate thread
                winapi::um::processthreadsapi::CreateThread(
                    core::ptr::null_mut(),
                    0,
                    Some(setup),
                    hinst_dll as _,
                    0,
                    core::ptr::null_mut(),
                );
            }
            true
        }
        _ => true,
    };
    res as i32
}
