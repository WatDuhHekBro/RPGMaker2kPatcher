//mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod util;

use crate::{structs::LcfDataBase, util::file_operations};
use binrw::{BinRead, BinWriterExt};
use dotenvy::dotenv;
use std::{env, fs, io::Cursor, path::Path};

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
            "decompile" => {
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
            "testLib" => {
                let subpath_reference = Path::new(&path_to_original).join("original-recompiled");
                let subpath_workspace_default =
                    Path::new(&path_to_original).join("patches-original");
                let subpath_workspace_custom = Path::new(&path_to_original).join("patches-custom");
                let subpath_patched_default =
                    Path::new(&path_to_original).join("original-default-patches");
                let subpath_patched_custom = Path::new(&path_to_original).join("translated-custom");

                println!(
                    "Testing whether or not original binary read/write identical to original..."
                );
                file_operations::bulk_serialize_lcfmapunits(&path_to_original, &subpath_reference)
                    .unwrap();

                println!("\nTesting whether or not patched binary (from default patches) identical to original...");
                file_operations::bulk_generate_toml_patches(
                    &path_to_original,
                    &subpath_workspace_default,
                )
                .unwrap();
                file_operations::bulk_apply_toml_patches(
                    &path_to_original,
                    &subpath_workspace_default,
                    &subpath_patched_default,
                )
                .unwrap();

                println!("\nTesting whether or not the already-patched binaries are identical (to legacy patches)...");
                file_operations::bulk_generate_toml_patches(
                    &path_to_original,
                    &subpath_workspace_custom,
                )
                .unwrap();
                file_operations::bulk_convert_legacy_patches(
                    &subpath_workspace_custom,
                    &path_to_legacy_workspace,
                )
                .unwrap();
                file_operations::bulk_apply_toml_patches(
                    &path_to_original,
                    &subpath_workspace_custom,
                    &subpath_patched_custom,
                )
                .unwrap();
            }
            "test" => {
                println!("Testing...");

                // Read database
                let db =
                    fs::read("/home/watduhhekbro/external/workspace/debug/src/RPG_RT.ldb").unwrap();
                let mut reader = Cursor::new(db);
                let db = LcfDataBase::read_be(&mut reader).unwrap();
                //println!("{db:?}");

                let mut file = file_operations::overwrite(
                    "/home/watduhhekbro/external/workspace/debug/ref/RPG_RT.ldb",
                )
                .unwrap();
                file.write_be(&db).unwrap();
                fs::write(
                    "/home/watduhhekbro/external/workspace/debug/ref/RPG_RT.txt",
                    format!("{db:?}"),
                )
                .unwrap();
            }
            _ => {
                println!("Unknown command!");
            }
        },
        None => {
            println!("Available commands: decompile, generatePatches, applyPatches, convertLegacyPatches");
        }
    }
}
