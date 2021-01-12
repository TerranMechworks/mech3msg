use cstr_core::CStr;
use winapi;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD, PBYTE, PDWORD};
use winapi::um::winnt::{
    IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE, IMAGE_ORDINAL_FLAG32,
    PAGE_READWRITE, PIMAGE_DOS_HEADER, PIMAGE_IMPORT_BY_NAME, PIMAGE_IMPORT_DESCRIPTOR,
    PIMAGE_NT_HEADERS32, PIMAGE_THUNK_DATA32,
};

const HOOKED_MSG: &[u8] = b"HOOKED\0";
const PDWORD_SIZE: SIZE_T = core::mem::size_of::<PDWORD>() as SIZE_T;

#[inline]
unsafe fn unprotect_patch(dest: PDWORD, func: DWORD) {
    let mut old_protect: DWORD = 0;
    let res = winapi::um::memoryapi::VirtualProtect(
        dest as _,
        PDWORD_SIZE,
        PAGE_READWRITE,
        &mut old_protect,
    );
    if res == 0 {
        panic!("Failed to make memory writable!");
    }
    *dest = func;
    let mut junk: DWORD = 0;
    winapi::um::memoryapi::VirtualProtect(dest as _, PDWORD_SIZE, old_protect, &mut junk);
}

pub unsafe fn patch_iat(dll_name: &[u8], func_name: &[u8], func_ptr: DWORD) -> bool {
    let handle = winapi::um::libloaderapi::GetModuleHandleA(core::ptr::null());
    // we'll use this to convert Relative Virtual Address (RVA) offsets to Virtual Address (VA) pointers
    let base_addr = handle as PBYTE;

    let dos_header = handle as PIMAGE_DOS_HEADER;
    if (*dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return false;
    }
    let nt_header = base_addr.offset((*dos_header).e_lfanew as isize) as PIMAGE_NT_HEADERS32;
    if (*nt_header).Signature != IMAGE_NT_SIGNATURE {
        return false;
    }

    // import directory table
    let idt_offset = (*nt_header).OptionalHeader.DataDirectory
        [IMAGE_DIRECTORY_ENTRY_IMPORT as usize]
        .VirtualAddress;
    let idt_ptr = base_addr.offset(idt_offset as isize) as PIMAGE_IMPORT_DESCRIPTOR;

    let mut i = 0;
    loop {
        let import_desc = idt_ptr.offset(i) as PIMAGE_IMPORT_DESCRIPTOR;
        i += 1;

        // end of import directory table
        if *(*import_desc).u.Characteristics() == 0 {
            return false;
        }

        let import_dll_name_ptr = base_addr.offset((*import_desc).Name as isize);
        // winapi::um::debugapi::OutputDebugStringA(import_dll_name_ptr as *const _);
        let import_dll_name = CStr::from_ptr(import_dll_name_ptr as _).to_bytes();

        // strcmpi without allocating
        if import_dll_name.len() != dll_name.len() {
            continue;
        }
        if !import_dll_name
            .iter()
            .zip(dll_name)
            .all(|(a, b)| a.to_ascii_lowercase() == *b)
        {
            continue;
        }

        let thunks = base_addr.offset((*import_desc).FirstThunk as isize) as PIMAGE_THUNK_DATA32;
        let import_lookup_table = base_addr.offset(*(*import_desc).u.OriginalFirstThunk() as isize)
            as PIMAGE_THUNK_DATA32;

        let mut j = 0;
        loop {
            let orig_thunk = import_lookup_table.offset(j) as PIMAGE_THUNK_DATA32;

            // end of import lookup table
            if *(*orig_thunk).u1.Function() == 0 {
                break;
            }

            // don't test/patch ordinal imports. imports from system DLLs don't
            // use ordinals, since they can change between Windows versions, so
            // it's unlikely we'll want to patch these.
            if *(*orig_thunk).u1.Ordinal() & IMAGE_ORDINAL_FLAG32 != 0 {
                j += 1;
                continue;
            }

            let import = base_addr.offset(*(*orig_thunk).u1.AddressOfData() as isize)
                as PIMAGE_IMPORT_BY_NAME;

            let import_func_name_ptr = (*import).Name.as_ptr();
            // winapi::um::debugapi::OutputDebugStringA(import_func_name_ptr);
            let import_func_name = CStr::from_ptr(import_func_name_ptr).to_bytes();

            if import_func_name == func_name {
                let thunk = thunks.offset(j) as PDWORD;
                unprotect_patch(thunk, func_ptr);
                // let us know we hooked the function (terrible message)
                winapi::um::debugapi::OutputDebugStringA(HOOKED_MSG.as_ptr() as *const _);
                return true;
            }

            j += 1;
        }
    }
}
