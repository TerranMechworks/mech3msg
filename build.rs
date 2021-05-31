use encoding::all::WINDOWS_1252;
use encoding::{EncoderTrap, Encoding};
use serde::Deserialize;
use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
mod constants {
    // On Windows, the binutils shipped by Mingw-w64 aren't prefixed
    pub const WINDMC: &str = "windmc";
}

#[cfg(not(windows))]
mod constants {
    // On Linux and MacOS, the binutils shipped by Mingw-w64 are prefixed
    pub const WINDMC: &str = "i686-w64-mingw32-windmc";
}

#[derive(Debug, Deserialize)]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<(String, u32, String)>,
}

fn main() {
    // Only re-run the build script if the messages change
    println!("cargo:rerun-if-changed=Mech3Msg.json");

    let out_dir = env::var("OUT_DIR").expect("No OUT_DIR env var");
    let out_path = PathBuf::from(&out_dir);

    let json = read_to_string("Mech3Msg.json").expect("Failed to read JSON");
    let Messages {
        language_id,
        mut entries,
    } = serde_json::from_str(&json).expect("Failed to parse JSON");

    let language = match language_id {
        0x407 => "Language=German\r\n",
        0x409 => "Language=English\r\n",
        0x40c => "Language=French\r\n",
        _ => panic!(
            "Unknown language ID {} - refusing to continue, codepage may be incorrect",
            language_id
        ),
    };

    // Build the message table resource (must use Windows newlines!)
    let mut resources = String::new();
    resources.push_str("MessageIdTypedef=DWORD\r\n");
    resources.push_str("LanguageNames=(\r\n");
    resources.push_str("  English=0x409:MSG1033\r\n");
    resources.push_str("  German=0x407:MSG1031\r\n");
    resources.push_str("  French=0x40c:MSG1036\r\n");
    resources.push_str(")\r\n\r\n");
    for (key, mid, msg) in &entries {
        resources.push_str(&format!("MessageId=0x{:X}\r\n", mid));
        resources.push_str(&format!("SymbolicName={}\r\n", key));
        resources.push_str(language);
        resources.push_str(msg);
        resources.push_str("\r\n.\r\n\r\n");
    }

    let mc_path = out_path.join("messages.mc");
    {
        let buf = WINDOWS_1252
            .encode(&resources, EncoderTrap::Strict)
            .expect("Failed to encode messages");
        let mut file = BufWriter::new(File::create(&mc_path).expect("Failed to create messages"));
        file.write_all(&buf).expect("Failed to write messages");
    }

    // Compile the message table into a resource
    let status = Command::new(constants::WINDMC)
        .arg("--target")
        .arg("pe-i386")
        .arg("--codepage_in")
        .arg("1252")
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
    embed_resource::compile(rc_path);

    // Must be sorted by key, for the binary search in the DLL to work!
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Build the static table for message key to message table ID lookups
    let mut msg_def = String::new();
    let mut msg_map = String::new();

    msg_map.push_str("static LOOKUP: &[(&[u8], i32)] = &[\n");
    for (key, mid, _msg) in &entries {
        msg_def.push_str("#[allow(non_upper_case_globals)]\n");
        msg_def.push_str(&format!("static {0}: &[u8] = b\"{0}\";\n", key));
        msg_map.push_str(&format!("    ({}, 0x{:X}),\n", key, mid));
    }
    msg_map.push_str("];\n");

    let rs_path = out_path.join("lookup.rs");
    {
        let mut file = BufWriter::new(File::create(&rs_path).expect("Failed to create lookup"));
        write!(&mut file, "{}\n{}", msg_def, msg_map).expect("Failed to write lookup");
    }
}
