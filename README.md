# MechWarrior 3 Messages (Mech3Msg) replacement

This project produces a DLL that implements the Zipper Interactive localization API used in [MechWarrior 3](https://en.wikipedia.org/wiki/MechWarrior_3), and so can be used to replace the default `Mech3Msg.dll`. The replacement DLL can also be used for [Recoil](https://en.wikipedia.org/wiki/Recoil_(video_game)), another Zipper game (see below).

Obviously, this is an unofficial fan effort and not connected to the developers or publishers. [Join us on Discord](https://discord.gg/Be53gMy)!

## Requirements

[Rust](https://www.rust-lang.org/) is required. Since the original game is x86/32-bit only, the only two possible targets:

1. `i686-pc-windows-gnu` aka. GNU, i.e. [Mingw-w64](http://mingw-w64.org). This is the default target.
1. `i686-pc-windows-msvc` aka. MSVC. Support for this is experimental. Change the target in `.cargo/config.toml`.

A Windows resource message compiler is also required. The only supported message compiler is `windmc` (Windows)/`i686-w64-mingw32-windmc` (macOS/Linux), provided by `binutils` via [Mingw-w64](http://mingw-w64.org). This is required even if MSVC is targetted, due to the build script (`build.rs`). To install Mingw-w64:

* **macOS**: `brew install mingw-w64`
* **Ubuntu**: `apt install mingw-w64`
* **Windows**: See [Mingw-w64](http://mingw-w64.org)

Cross-compilation from Linux/macOS is supported, see [building](#building).

Additionally, a file named `Mech3Msg.json` is required. This is a dump of all message keys, IDs, and values. I cannot provide a file with the game's values due to copyright concerns, but it can be generated from the original DLL using [mech3ax](https://github.com/TerranMechworks/mech3ax). Alternatively, there's a dummy version in `.github/Mech3Msg.json` used for testing.

## Building

It's recommended to always build with `--release`. This results in a significantly smaller DLL. Additionally, building the default dev version may fail when cross compiling due to:

```plain
undefined reference to `rust_eh_personality'
```

Except for common Windows libraries (e.g. `kernel32.dll`, `msvcrt.dll`, or `vsruntime*.dll` depending on the compiler), everything is statically linked. Therefore, the release version is built using aggressive Link Time Optimization (LTO) to remove any unnecessary linked code. Additionally, all symbols are stripped for GNU, and debug information is omitted for MSVC.

## Testing

To run the tests, a Windows 32-bit Python 3.7+ installation is strictly necessary. 64-bit Python will not be able to load the 32-bit DLL! The tests functionally test the DLL and message table, so can only be run on Windows.

The tests can also be used to check that the produced DLL functions identically to the original. Alternatively, a program like [Resource Hacker](http://www.angusj.com/resourcehacker/) can be used to dump the message tables in both the original DLL and produced DLL, and compare those. This bypasses the message extraction via mech3ax, and so could be useful to diagnose issues with that. Minor differences in ordering should be allowed using this method.

## Recoil support

Recoil is supported, however the input file must still be called `Mech3Msg.json`, and the output file must be renamed from `mech3msg.dll` to `messages.dll`.

## Internals

### Adding messages

Messages may be added as long as the keys and IDs do not collide with existing keys or IDs. Although this is not validated, it is extremely important. Only ASCII is supported (technically ANSI codepage 1252).

### On message table IDs

Although most messages are looked up by key, some are referenced directly by message ID (for example, the CD check message box title and contents). Even though the different versions remove and add keys, they go to great lengths to preserve compatible IDs between versions. You should do the same.

### Common issues

If the game crashes when using a new DLL, then it is likely a key was looked up but not found. The game doesn't have great error checking around this. The symptom is the game crashes after going to a black screen; the menu is never seen. The most common cause for this is using a DLL created from a `Mech3Msg.json` from a different version.

Messages from different versions can be merged into one JSON file to support different versions. But there's a simpler way. Most people run version 1.2, since this was the last official patch. So in practice, replacement DLLs can be based on v1.2 and require people to patch to that.

## License

Licensed under the European Union Public Licence (EUPL) 1.2 ([LICENSE](LICENSE) or https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).
