mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod wrappers; // Intermediary data types for specialized functionality (upfront byte counts & null ID lists (similar to NullStrings))

use crate::structs::{patch::Dialogue, LcfMapUnit, LegacyPatch, Patch};
use binrw::{
    io::{Cursor, Seek, Write},
    BinRead, BinWrite, BinWriterExt,
};
use std::fs::File;

// The reason this error is thrown in every potential line instead of propagating upwards via "?" is because
// if you used "?" for all of the DynamicInteger::read()'s, you'll only see the error thrown in your main function.
// Not helpful at all for debugging issues.
pub const ERROR_BINRW_READ: &str = "Binary read failed!";

fn main() {
    /*let a = Patch {
        dialogue: vec![Dialogue {
            event: 1,
            page: 2,
            command_start: 3,
            command_length: 4,
            original: "asdf\\c",
            patched: "noice\\n\n\\n\\c\n\\c",
        }],
    };
    println!("{}", toml::to_string(&a).unwrap());
    return;*/

    //return crate::dialogue::core::main().unwrap();

    let mut reader = Cursor::new(include_bytes!(
        "/home/watduhhekbro/external/workspace/Map0134.lmu"
    ));
    let servers = LcfMapUnit::read(&mut reader).unwrap();
    println!("{servers:?}\n");
    //let blob = servers.get_event(53).unwrap().get_page(1).unwrap();
    //println!("{:?}\n", blob);

    let mut writer = Cursor::new(Vec::<u8>::new());
    writer.write_be(&servers).unwrap();
    //println!("{:02X?}", writer.into_inner());
    let mut output_file =
        File::create_new("/home/watduhhekbro/external/workspace/Map0134-gen.lmu").unwrap();
    output_file.write_be(&servers).unwrap();
}
