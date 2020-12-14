use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
mod constants {
    // On Windows, the binutils shipped by Mingw-w64 aren't prefixed
    pub const WINDMC: &str = "windmc";
    pub const WINDRES: &str = "windres";
    pub const LIB: &str = "messages.lib";
}

#[cfg(not(windows))]
mod constants {
    // On Linux and MacOS, the binutils shipped by Mingw-w64 are prefixed
    pub const WINDMC: &str = "i686-w64-mingw32-windmc";
    pub const WINDRES: &str = "i686-w64-mingw32-windres";
    pub const LIB: &str = "libmessages.a";
}

fn main() {
    // Only re-run the build script if the messages change
    println!("cargo:rerun-if-changed=Mech3Msg.json");

    let json = read_to_string("Mech3Msg.json").expect("Failed to read JSON");
    let mut messages: Vec<(String, i32, String)> =
        serde_json::from_str(&json).expect("Failed to parse JSON");
    // Must be sorted by key, for the binary search in the DLL to work!
    messages.sort_by(|a, b| a.0.cmp(&b.0));

    // Build the static table for message key to message table ID lookups
    let mut msg_def = String::new();
    let mut msg_map = String::new();

    msg_map.push_str("static LOOKUP: &[(&[u8], i32)] = &[\n");
    for (key, mid, _msg) in &messages {
        msg_def.push_str("#[allow(non_upper_case_globals)]\n");
        msg_def.push_str(&format!("static {0}: &[u8] = b\"{0}\";\n", key));
        msg_map.push_str(&format!("    ({}, 0x{:X}),\n", key, mid));
    }
    msg_map.push_str("];\n");

    let out_dir = env::var("OUT_DIR").expect("No OUT_DIR env var");
    let out_path = PathBuf::from(&out_dir);

    let rs_path = out_path.join("lookup.rs");
    {
        let mut file = BufWriter::new(File::create(&rs_path).expect("Failed to create lookup"));
        write!(&mut file, "{}\n{}", msg_def, msg_map).expect("Failed to write lookup");
    }

    // Build the message table resource (must use Windows newlines!)
    let mut resources = String::new();
    // These values aren't supported by binutils
    // resources.push_str("MessageIdTypedef=DWORD\r\n");
    // resources.push_str("LanguageNames=(English=0x409:MSG1033)\r\n\r\n");
    for (key, mid, msg) in &messages {
        resources.push_str(&format!("MessageId=0x{:X}\r\n", mid));
        resources.push_str(&format!("SymbolicName={}\r\n", key));
        resources.push_str("Language=English\r\n");
        resources.push_str(msg);
        resources.push_str("\r\n.\r\n\r\n");
    }

    let mc_path = out_path.join("messages.mc");
    {
        let mut file = BufWriter::new(File::create(&mc_path).expect("Failed to create messages"));
        write!(&mut file, "{}", resources).expect("Failed to write messages");
    }

    // Compile the message table into a resource
    let status = Command::new(constants::WINDMC)
        .arg("--target")
        .arg("pe-i386")
        .arg("--codepage_out")
        .arg("1252")
        .arg("--headerdir")
        .arg(&out_dir)
        .arg("--rcdir")
        .arg(&out_dir)
        .arg("--ascii_out")
        .arg(&mc_path)
        .status()
        .expect("Failed to execute windmc");
    assert!(
        status.success(),
        "Failed to compile message table ({})",
        status
    );

    // Compile the resource into a linkable library
    let rc_path = out_path.join("messages.rc");
    let lib_path = out_path.join(constants::LIB);

    let status = Command::new(constants::WINDRES)
        .arg("--target")
        .arg("pe-i386")
        .arg("--input")
        .arg(&rc_path)
        .arg("--output")
        .arg(&lib_path)
        .arg("--output-format")
        .arg("coff")
        .status()
        .expect("Failed to execute windres");
    assert!(status.success(), "Failed to compile resources ({})", status);

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=messages");
}
