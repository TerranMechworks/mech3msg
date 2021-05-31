# Only works on Windows! Some functions need 32-bit Python!
import ctypes
import ctypes.wintypes

from contextlib import contextmanager
from typing import Iterator

# https://docs.microsoft.com/en-us/windows/desktop/api/libloaderapi/nf-libloaderapi-loadlibraryexw

LoadLibraryEx = ctypes.windll.kernel32.LoadLibraryExW
FreeLibrary = ctypes.windll.kernel32.FreeLibrary

# DONT_RESOLVE_DLL_REFERENCES is deprecated
# This also enables a 64-bit process to load the 32-bit DLL
LOAD_LIBRARY_AS_DATAFILE = 0x00000002
LOAD_LIBRARY_AS_IMAGE_RESOURCE = 0x00000020

# https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-formatmessage
# https://stackoverflow.com/questions/18905702/python-ctypes-and-mutable-buffers

FormatMessage = ctypes.windll.kernel32.FormatMessageW
LocalFree = ctypes.windll.kernel32.LocalFree
FORMAT_MESSAGE_ALLOCATE_BUFFER = 0x00000100  # call LocalFree after
FORMAT_MESSAGE_FROM_HMODULE = 0x00000800
FORMAT_MESSAGE_IGNORE_INSERTS = 0x00000200  # don't format the message


@contextmanager
def load_library(lpLibFileName: str) -> Iterator[int]:
    hFile = None  # reserved for future use
    dwFlags = LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE
    hModule: int = LoadLibraryEx(lpLibFileName, hFile, dwFlags)
    yield hModule
    FreeLibrary(hModule)


@contextmanager
def format_message(hModule: int, dwLanguageId: int, dwMessageId: int) -> Iterator[str]:
    # need to pass a pointer to this with FORMAT_MESSAGE_ALLOCATE_BUFFER
    lpBuffer = ctypes.wintypes.LPWSTR()
    dwFlags = (
        FORMAT_MESSAGE_ALLOCATE_BUFFER
        | FORMAT_MESSAGE_FROM_HMODULE
        | FORMAT_MESSAGE_IGNORE_INSERTS
    )
    # dwLanguageId = 0  # use default lookup order
    dwSize = FormatMessage(
        dwFlags, hModule, dwMessageId, dwLanguageId, ctypes.byref(lpBuffer), 0, None
    )
    if not dwSize or lpBuffer.value is None:
        raise ctypes.WinError()
    msg = lpBuffer.value[:dwSize]
    # this returns the message as a string with trailing "\r\n"
    yield msg
    LocalFree(lpBuffer)
