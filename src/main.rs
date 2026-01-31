mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod util;
mod wrappers; // Intermediary data types for specialized functionality (upfront byte counts & null ID lists (similar to NullStrings))

use crate::structs::{patch::Dialogue, LcfMapUnit, LcfMapUnitToml, LegacyPatch, Patch};
use binrw::{
    io::{Cursor, Seek, Write},
    BinRead, BinWrite, BinWriterExt,
};
use dotenvy::dotenv;
use std::fs::{self, File};

// The reason this error is thrown in every potential line instead of propagating upwards via "?" is because
// if you used "?" for all of the DynamicInteger::read()'s, you'll only see the error thrown in your main function.
// Not helpful at all for debugging issues.
pub const ERROR_BINRW_READ: &str = "Binary read failed!";

fn main() {
    dotenv().ok();

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
    let mut map = LcfMapUnit::read(&mut reader).unwrap();
    //println!("{map:?}\n");
    //let blob = map.get_event(53).unwrap().get_page(1).unwrap();
    //println!("{:?}\n", blob);
    //fs::write("test/test.toml", toml::to_string(&map).unwrap()).unwrap();
    //fs::write("test/test.json", serde_json::to_string_pretty(&map).unwrap()).unwrap();
    //fs::write("test.toml", map.generate_toml_map()).unwrap();
    fs::write("test.toml", map.generate_toml_patch()).unwrap();

    //let mut writer = Cursor::new(Vec::<u8>::new());
    //writer.write_be(&map).unwrap();
    //println!("{:02X?}", writer.into_inner());
    fs::remove_file("/home/watduhhekbro/external/workspace/Map0134-gen.lmu").ok();
    let mut output_file =
        File::create("/home/watduhhekbro/external/workspace/Map0134-gen.lmu").unwrap();
    output_file.write_be(&map).unwrap();

    // Test TOML bidirectional transportability
    /*let new = toml::from_str::<LcfMapUnitToml>(&fs::read_to_string("test.toml").unwrap()).unwrap();
    println!("{new:?}");*/
    /*fs::remove_file("/home/watduhhekbro/external/workspace/Map0133-gen2.lmu").ok();
    let mut output_file =
        File::create("/home/watduhhekbro/external/workspace/Map0133-gen2.lmu").unwrap();
    output_file.write_be(&new).unwrap();*/

    let mut read_patch: Patch = toml::from_str(&fs::read_to_string("test.toml").unwrap()).unwrap();
    read_patch.trim_dialogue_ending_newline();
    map.apply_patch(&read_patch);
}
