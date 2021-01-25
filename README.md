# MechWarrior 3 Messages (Mech3Msg) replacement

This project produces a DLL that implements the [MechWarrior 3](https://en.wikipedia.org/wiki/MechWarrior_3) localization API, and so can be used to replace the default `Mech3Msg.dll`. It can also do more. For example, the resolution for [`GetTickCount`](https://docs.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount) "is typically in the range of 10 milliseconds to 16 milliseconds". This causes problems on modern PCs when running MechWarrior 3. The DLL can patch this function to a custom implementation using a high-resolution timer ([`QueryPerformanceCounter`](https://docs.microsoft.com/en-us/windows/win32/api/profileapi/nf-profileapi-queryperformancecounter)), to fix some issues without requiring workarounds like [dgVoodoo 2](http://dege.freeweb.hu/).

The replacement DLL can also be used for another Zipper Interactive game, [Recoil](https://en.wikipedia.org/wiki/Recoil_(video_game)) (in this case, the DLL is called `messages.dll` and there's some extra steps).

Obviously, this is an unofficial fan effort and not connected to the developers or publishers. [Join us on Discord](https://discord.gg/Be53gMy)!

## Requirements

[Rust](https://www.rust-lang.org/) is required, this project uses the `nightly` toolchain. The only valid/supported target is `i686-pc-windows-gnu`, i.e. [Mingw-w64](http://mingw-w64.org). This is because:

* The original game is x86/32-bit only
* The build script relies on `binutils` provided by Mingw-w64 to compile resources

Command-line proficiency is assumed; all commands provided are for Bash-compatible shells, and not Powershell! (Even Windows-only commands.)

Cross-compilation from Linux/macOS is supported, although the tests won't be run-able (it produces a `.exe`, if it builds at all due to exception handling). You can install the target with:

```bash
rustup +nightly target add i686-pc-windows-gnu
```

You will also need a file named `Mech3Msg.json`, which is a dump of all message keys, IDs, and values. I cannot provide this file, but you can generate one from the original DLL using [mech3ax](https://github.com/TerranMechworks/mech3ax):

```bash
unzbd messages "Mech3Msg.dll" "Mech3Msg.json" --dump-ids
```

## Building

It's recommended you use the `release` version, as this results in a significantly smaller DLL:

```bash
cargo build --release
```

Except for `KERNEL32.dll` and `msvcrt.dll`, everything is statically linked. Therefore, the release version is built using aggressive Link Time Optimization (LTO) to remove any unnecessary linked code. Additionally, all symbols are stripped.

If patching of `GetTickCount` is not wanted (for example when targeting Recoil), this can be disabled via:

```bash
cargo build --release --no-default-features
```

## Comparing

On Windows, you can use `compare.py` to test if the DLLs have the same behaviour (assuming the messages were correctly extracted):

```bash
python3 compare.py "Mech3Msg.json" "Mech3Msg.dll" "target/i686-pc-windows-gnu/release/mech3msg.dll"
```

Alternatively, you can use a program like [Resource Hacker](http://www.angusj.com/resourcehacker/) to dump the message tables in both the original DLL and produced DLL, and compare those. Minor differences in ordering should be allowed.

## Internals

### Adding messages

Messages may be added as long as the keys and IDs do not collide with existing keys or IDs. Although this is not validated, it is extremely important. Only ASCII is supported, technically codepage 1252.

### On message table IDs

Although most messages are looked up by key, some are referenced directly by message ID (for example, the CD check message box title and contents). Even though the different versions remove and add keys, they go to great lengths to preserve compatible IDs between versions. You should do the same.

### Common issues

If the game crashes when using a new DLL, then it is likely a key was looked up but not found. The game doesn't have great error checking around this. The symptom is the game crashes after going to a black screen; the menu is never seen. The most common cause for this is using a DLL created from a `Mech3Msg.json` from a different version.

Messages from different versions can be merged into one JSON file to support different versions. But there's a simpler way. Most people run version 1.2, since this was the last official patch. So in practice replacement DLLs can be based on v1.2, and require people to patch to that.

### MSVC support

This could probably be changed to support `msvc`, too, but it's a pain. If you would like to see what this entails, the [embed_resource](https://docs.rs/embed-resource/) crate provides some insight. However, the crate itself can't be used, because it doesn't expose `windmc`/`mc.exe`, only `windres`/`res.exe`.

### Recoil

Recoil support in both mech3msg and mech3ax is experimental:

```bash
unzbd messages "Messages.dll" "Mech3Msg.json" --dump-ids --skip-data 48
cargo build --release
cp "target/i686-pc-windows-gnu/release/mech3msg.dll" "Messages.dll"
```

## License

MechWarrior 3 Messages is GPLv3 licensed. Please see `LICENSE.txt`.
