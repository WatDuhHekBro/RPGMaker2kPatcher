//mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod util;
mod wrappers; // Intermediary data types for specialized functionality (upfront byte counts & null ID lists (similar to NullStrings))

use crate::{structs::LcfDataBase, util::file_operations};
use binrw::{BinRead, BinWriterExt};
use dotenvy::dotenv;
use std::{env, fs, io::Cursor};

fn main() {
    dotenv().ok();

    // Environment Variables
    let path_to_original = env::var("PATH_TO_ORIGINAL").unwrap();
    let path_to_workspace = env::var("PATH_TO_WORKSPACE").unwrap();
    let path_to_reference = env::var("PATH_TO_REFERENCE").unwrap();
    let path_to_patched = env::var("PATH_TO_PATCHED").unwrap();
    let path_to_legacy_workspace = env::var("PATH_TO_WORKSPACE_LEGACY").unwrap();

    // CLI Arguments
    let args = env::args().collect::<Vec<String>>();
    let command = args.get(1);
    //let command = Some(&String::from("test"));

    match command {
        Some(command) => match command.as_str() {
            "decompileMaps" => {
                file_operations::bulk_generate_toml_maps(&path_to_original, &path_to_reference)
                    .unwrap();
            }
            "generatePatches" => {
                file_operations::bulk_generate_toml_patches(&path_to_original, &path_to_workspace)
                    .unwrap();
            }
            "applyPatches" => {
                file_operations::bulk_apply_toml_patches(
                    &path_to_original,
                    &path_to_workspace,
                    &path_to_patched,
                )
                .unwrap();
            }
            "convertLegacyPatches" => {
                file_operations::bulk_convert_legacy_patches(
                    &path_to_workspace,
                    &path_to_legacy_workspace,
                )
                .unwrap();
            }
            "testHexdump" => {
                let path = args.get(2);

                match path {
                    Some(path) => {
                        let map = file_operations::read_lcfmapunit(path).unwrap();
                        let hexdump = file_operations::hexdump(map);
                        fs::write(format!("{path}.txt"), hexdump).unwrap();
                    }
                    None => {
                        println!("Enter in a valid path!");
                    }
                }
            }
            "testLMUSerialization" => {
                file_operations::bulk_serialize_lcfmapunits(&path_to_original, &path_to_reference)
                    .unwrap();
            }
            "test" => {
                println!("Testing...");

                // Read database
                let db =
                    fs::read("/home/watduhhekbro/external/workspace/debug/src/RPG_RT.ldb").unwrap();
                let mut reader = Cursor::new(db);
                let db = LcfDataBase::read_be(&mut reader).unwrap();
                println!("{db:?}");

                let mut file = file_operations::overwrite("/home/watduhhekbro/external/workspace/debug/out/RPG_RT.ldb").unwrap();
                file.write_be(&db).unwrap();
                fs::write("/home/watduhhekbro/external/workspace/debug/out/RPG_RT.txt", format!("{db:?}")).unwrap();
            }
            _ => {
                println!("Unknown command!");
            }
        },
        None => {
            println!("Available commands: decompileMaps, generatePatches, applyPatches, convertLegacyPatches, testLMUSerialization");
        }
    }
}
