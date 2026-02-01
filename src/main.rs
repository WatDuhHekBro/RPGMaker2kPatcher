mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod util;
mod wrappers; // Intermediary data types for specialized functionality (upfront byte counts & null ID lists (similar to NullStrings))

use crate::util::file_operations;
use binrw::BinWriterExt;
use dotenvy::dotenv;
use std::fs;

fn main() {
    dotenv().ok();

    let map = file_operations::read_lcfmapunit("/home/watduhhekbro/external/workspace/Map0134.lmu")
        .unwrap();
    fs::write(
        "/home/watduhhekbro/external/workspace/Map0134.toml",
        map.generate_toml_map(),
    )
    .unwrap();
    fs::write("test.toml", map.generate_toml_patch()).unwrap();

    let mut output_file =
        file_operations::overwrite("/home/watduhhekbro/external/workspace/Map0134-gen.lmu")
            .unwrap();
    output_file.write_be(&map).unwrap();

    let map = file_operations::read_lcfmapunit_and_patch(
        "/home/watduhhekbro/external/workspace/Map0134.lmu",
        "/home/watduhhekbro/external/workspace/test.toml",
        "/home/watduhhekbro/external/workspace/patched.lmu",
    )
    .unwrap();

    fs::write(
        "/home/watduhhekbro/external/workspace/patched.toml",
        map.generate_toml_map(),
    )
    .unwrap();
}
