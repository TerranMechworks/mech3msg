# Only works on Windows!
import ctypes
import ctypes.wintypes
import json
from contextlib import contextmanager

# https://docs.microsoft.com/en-us/windows/desktop/api/libloaderapi/nf-libloaderapi-loadlibraryexw

LoadLibraryEx = ctypes.windll.kernel32.LoadLibraryExW
FreeLibrary = ctypes.windll.kernel32.FreeLibrary

# DONT_RESOLVE_DLL_REFERENCES is deprecated
LOAD_LIBRARY_AS_DATAFILE = 0x00000002
LOAD_LIBRARY_AS_IMAGE_RESOURCE = 0x00000020

# https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-formatmessage
# https://stackoverflow.com/questions/18905702/python-ctypes-and-mutable-buffers

FormatMessage = ctypes.windll.kernel32.FormatMessageW
LocalFree = ctypes.windll.kernel32.LocalFree
FORMAT_MESSAGE_ALLOCATE_BUFFER = 0x00000100  # call LocalFree after
FORMAT_MESSAGE_FROM_HMODULE = 0x00000800
FORMAT_MESSAGE_IGNORE_INSERTS = 0x00000200  # don't format the message


def zlocgetid(dll):
    lib = ctypes.CDLL(dll)  # cdecl calling conventions
    fn = lib.ZLocGetID
    fn.argtypes = [ctypes.c_char_p]
    fn.restype = ctypes.c_int32
    return fn


@contextmanager
def load_library(lpLibFileName):
    hFile = None  # reserved for future use
    dwFlags = LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE
    hModule = LoadLibraryEx(lpLibFileName, hFile, dwFlags)
    yield hModule
    FreeLibrary(hModule)


@contextmanager
def format_message(hModule, dwLanguageId, dwMessageId):
    lpBuffer = (
        ctypes.wintypes.LPWSTR()
    )  # need to pass a pointer to this with FORMAT_MESSAGE_ALLOCATE_BUFFER
    dwFlags = (
        FORMAT_MESSAGE_ALLOCATE_BUFFER
        | FORMAT_MESSAGE_FROM_HMODULE
        | FORMAT_MESSAGE_IGNORE_INSERTS
    )
    # dwLanguageId = 0  # use default lookup order
    dwSize = FormatMessage(
        dwFlags, hModule, dwMessageId, dwLanguageId, ctypes.byref(lpBuffer), 0, None
    )
    if not dwSize:
        raise ctypes.WinError()
    msg = lpBuffer.value[:dwSize]
    yield msg
    LocalFree(lpBuffer)


def run(message_json, dll_a, dll_b):
    with open(message_json, "rb") as f:
        messages = json.load(f)

    language_id = messages["language_id"]
    entries = messages["entries"]

    print("Comparing ZLocGetID...")
    fn_a = zlocgetid(dll_a)
    fn_b = zlocgetid(dll_b)

    for (key, mid, _msg) in entries:
        key = key.encode("ascii")
        assert fn_a(key) == mid, f"A: {key}"
        assert fn_b(key) == mid, f"B: {key}"

    print("Comparing messages...")
    with load_library(dll_a) as hModuleA, load_library(dll_b) as hModuleB:
        for (key, mid, _msg) in entries:
            with format_message(hModuleA, language_id, mid) as msg_a, format_message(
                hModuleB, language_id, mid
            ) as msg_b:
                assert msg_a == msg_b, f"{key}: '{msg_a}' != '{msg_b}'"

    print("Done")


if __name__ == "__main__":
    import sys

    _, message_json, dll_a, dll_b = sys.argv
    run(message_json, dll_a, dll_b)
