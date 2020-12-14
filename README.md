# MechWarrior 3 Messages (Mech3Msg) replacement

This project produces a DLL that implements the MechWarrior 3 localization API, and so can be used to replace the default `Mech3Msg.dll`.

Obviously, this is an unofficial fan effort and not connected to the developers or publishers. [Join us on Discord](https://discord.gg/Be53gMy)!

## Requirements

[Rust](https://www.rust-lang.org/) is required, this project uses the `nightly` toolchain. The only valid/supported target is `i686-pc-windows-gnu`, i.e. [Mingw-w64](http://mingw-w64.org). This is because:

* The original game is x86/32-bit only
* The build script relies on `binutils` provided by Mingw-w64 to compile resources

Cross-compilation from Linux/macOS is supported, although the tests won't be run-able (it produces an exe, if it builds at all due to exception handling). You can install the target with:

```bash
rustup +nightly target add i686-pc-windows-gnu
```

You will also need a file named `Mech3Msg.json`, which is a dump of all message keys, IDs, and values. I cannot provide this file, but you can generate one from the original DLL using [mech3ax](https://github.com/tobywf/mech3ax/):

```bash
unzbd messages "Mech3Msg.dll" "Mech3Msg.json" --dump-ids
```

## Building

It's recommended you use the `release` version, as this results in a significantly smaller DLL:

```bash
cargo build --release
```

Except for `KERNEL32.dll` and `msvcrt.dll`, everything is statically linked. Therefore, the release version is built using aggressive Link Time Optimization (LTO), to remove any unnecessary linked code. Additionally, all symbols are stripped.

## Internals

### Adding messages

Messages may be added as long as the keys and IDs do not collide with existing keys or IDs. Although this is not validated, it is extremely important. Only ASCII is supported, technically codepage 1252.

### On message table IDs

Although most messages are looked up by key, some are referenced directly by message ID (for example, the CD check message box title and contents). Even though the different versions remove and add keys, they go to great lengths to preserve compatible IDs between versions.

### Common issues

If the game crashes when using a new DLL, then it is likely a key was looked up but not found. The game doesn't have great error checking around this. The symptom is the game crashes after going to a black screen; the menu is never seen. The most common cause for this is using a DLL created from a `Mech3Msg.json` from a different version.

### MSVC support

This could probably be changed to support `msvc`, too, but it's a pain. If you would like to see what this entails, the [embed_resource](https://docs.rs/embed-resource/) crate provides some insight. However, the crate itself can't be used, because it doesn't expose `windmc`/`mc.exe`, only `windres`/`res.exe`.

## License

MechWarrior 3 Messages is GPLv3 licensed. Please see `LICENSE.txt`.
