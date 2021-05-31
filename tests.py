# Only works on Windows! Some functions need 32-bit Python!
import ctypes
import json
import unittest

from typing import cast, Optional
from shared import format_message, load_library


class TestMessageLibrary(unittest.TestCase):
    DLL_NAME: str = ".\\mech3msg.dll"

    def setUp(self) -> None:
        self.lib = ctypes.CDLL(self.DLL_NAME)

        self.zfn = self.lib.ZLocGetID
        self.zfn.argtypes = [ctypes.c_char_p]
        self.zfn.restype = ctypes.c_int32

    def get_id(self, key: Optional[bytes]) -> int:
        return cast(int, self.zfn(key))

    def test_get_id_null_pointer(self) -> None:
        # The original DLL does not handle this case
        self.assertEqual(self.get_id(None), 0)

    def test_get_id_empty_string(self) -> None:
        self.assertEqual(self.get_id(b""), 0)

    def test_get_id_not_found(self) -> None:
        self.assertEqual(self.get_id(b"MSG_FOO"), 0)

    def test_get_id_first(self) -> None:
        self.assertEqual(self.get_id(b"MSG_BACK"), 1)

    def test_get_id_last(self) -> None:
        # this is the last one in v1.0 only, e.g. v1.2 adds messages after
        self.assertEqual(self.get_id(b"MSG_NOTEXTURE_MEMORY"), 4031)

    def test_message_table(self) -> None:
        with open("Mech3Msg.json", "rb") as f:
            messages = json.load(f)

        language_id = messages["language_id"]
        entries = messages["entries"]

        with load_library(self.DLL_NAME) as hModule:
            for (key, mid, message) in entries:
                expected = message + "\r\n"
                with self.subTest(key=key, mid=mid):
                    with format_message(hModule, language_id, mid) as actual:
                        self.assertEqual(expected, actual)


if __name__ == "__main__":
    import sys

    TestMessageLibrary.DLL_NAME = sys.argv.pop()
    unittest.main()
