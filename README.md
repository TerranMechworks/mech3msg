# MechWarrior 3 Messages (Mech3Msg) replacement

This project produces a DLL that implements the Zipper Interactive localization API used in [MechWarrior 3](https://en.wikipedia.org/wiki/MechWarrior_3), and so can be used to replace the default `Mech3Msg.dll`. The replacement DLL can also be used for [Recoil](https://en.wikipedia.org/wiki/Recoil_(video_game)), another Zipper game (see below).

Obviously, this is an unofficial fan effort and not connected to the developers or publishers. [Join us on Discord](https://discord.gg/Be53gMy)!

## Requirements

[Rust](https://www.rust-lang.org/) is required, this project uses the `nightly` toolchain. Since the original game is x86/32-bit only, the only two supported targets are `i686-pc-windows-gnu` aka. GNU, i.e. [Mingw-w64](http://mingw-w64.org), and `i686-pc-windows-msvc` aka. MSVC.

You will also need `windmc` (Windows)/`i686-w64-mingw32-windmc` (macOS/Linux), provided by `binutils` via Mingw-w64, even if you target MSVC.

Command-line proficiency is assumed; all commands provided are for Bash-compatible shells, and not Powershell! (Even Windows-only commands.)

Cross-compilation from Linux/macOS is supported, although the tests won't be run-able (it produces a `.exe` - if it builds at all due to exception handling). You can install the target with:

```bash
rustup +nightly target add i686-pc-windows-gnu
```

You will also need a file named `Mech3Msg.json`, which is a dump of all message keys, IDs, and values. I cannot provide a file with the game's values, but you can generate one from the original DLL using [mech3ax](https://github.com/TerranMechworks/mech3ax):

```bash
unzbd messages "Mech3Msg.dll" "Mech3Msg.json" --dump-ids
```

Alternatively, there's a dummy version in `.github/Mech3Msg.json` used for testing.

## Building

It's recommended you use the `release` version, as this results in a significantly smaller DLL using GNU, and a reasonably-sized DLL using MSVC:

```bash
cargo build --release
```

Except for `kernel32.dll` and `msvcrt.dll` (in some cases), everything is statically linked. Therefore, the release version is built using aggressive Link Time Optimization (LTO) to remove any unnecessary linked code. Additionally, all symbols are stripped for GNU, and debug information is omitted for MSVC.

## Testing

To run the tests, you'll need a 32-bit Python 3.7+ installation on Windows. 64-bit Python will not be able to load the 32-bit DLL! The tests functionally test the DLL and message table, so can only be run on Windows.

The tests can also be used to check that the produced DLL functions identically to the original. Alternatively, a program like [Resource Hacker](http://www.angusj.com/resourcehacker/) can be used to dump the message tables in both the original DLL and produced DLL, and compare those. This bypasses the message extraction via mech3ax, and so could be useful to diagnose issues with that. Minor differences in ordering should be allowed using this method.

## Recoil support

Recoil support in both mech3msg and mech3ax is experimental. The input file must still be called `Mech3Msg.json`, and the output file must be renamed to `Messages.dll`:

```bash
unzbd messages "Messages.dll" "Mech3Msg.json" --dump-ids --skip-data 48
cargo build --release
cp "target/i686-pc-windows-gnu/release/mech3msg.dll" "Messages.dll"
```

## Kernel32.dll function hooking

Although [ZipperFixup](https://github.com/TerranMechworks/ZipperFixup) is now preferred as a more comprehensive solution, this repo also has a proof of concept to hook functions in `kernel32.dll` to fix a common issue without requiring workarounds like [dgVoodoo 2](http://dege.freeweb.hu/).

The resolution for [`GetTickCount`](https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount) "is typically in the range of 10 milliseconds to 16 milliseconds". On modern processors, this isn't enough for fast clock speeds, and causes e.g. physics glitches. When built with the `hook` feature, the DLL can patch this function to a custom implementation using a high-resolution timer ([`QueryPerformanceCounter`](https://docs.microsoft.com/en-us/windows/win32/api/profileapi/nf-profileapi-queryperformancecounter)).

This code path is not well-tested and remains largely unmaintained.

## Internals

### Adding messages

Messages may be added as long as the keys and IDs do not collide with existing keys or IDs. Although this is not validated, it is extremely important. Only ASCII is supported, technically codepage 1252.

### On message table IDs

Although most messages are looked up by key, some are referenced directly by message ID (for example, the CD check message box title and contents). Even though the different versions remove and add keys, they go to great lengths to preserve compatible IDs between versions. You should do the same.

### Common issues

If the game crashes when using a new DLL, then it is likely a key was looked up but not found. The game doesn't have great error checking around this. The symptom is the game crashes after going to a black screen; the menu is never seen. The most common cause for this is using a DLL created from a `Mech3Msg.json` from a different version.

Messages from different versions can be merged into one JSON file to support different versions. But there's a simpler way. Most people run version 1.2, since this was the last official patch. So in practice, replacement DLLs can be based on v1.2 and require people to patch to that.

## License

MechWarrior 3 Messages is GPLv3 licensed. Please see `LICENSE.txt`.
