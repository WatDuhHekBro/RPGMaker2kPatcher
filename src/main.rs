//mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod util;

use crate::{structs::LcfMapTree, util::file_operations};
use binrw::{BinRead, BinWriterExt};
use dotenvy::dotenv;
use std::{env, fs, io::Cursor, path::Path};

fn main() {
    dotenv().ok();

    // Environment Variables
    let path_to_original = env::var("PATH_TO_ORIGINAL");
    let path_to_reference = env::var("PATH_TO_REFERENCE");
    let path_to_workspace = env::var("PATH_TO_WORKSPACE");
    let path_to_patched = env::var("PATH_TO_PATCHED");
    let path_to_legacy_workspace = env::var("PATH_TO_WORKSPACE_LEGACY");

    // CLI Arguments
    let args = env::args().collect::<Vec<String>>();
    let command = args.get(1);
    //let command = Some(&String::from("testApplyPatches"));

    match command {
        Some(command) => match command.as_str() {
            "decompile" => {
                let path_to_original = path_to_original.expect(".env PATH_TO_ORIGINAL missing!");
                let path_to_reference = path_to_reference.expect(".env PATH_TO_REFERENCE missing!");

                file_operations::bulk_generate_toml_representations(
                    &path_to_original,
                    &path_to_reference,
                )
                .unwrap();
            }
            "generatePatches" => {
                let path_to_original = path_to_original.expect(".env PATH_TO_ORIGINAL missing!");
                let path_to_workspace = path_to_workspace.expect(".env PATH_TO_WORKSPACE missing!");

                file_operations::bulk_generate_toml_patches(&path_to_original, &path_to_workspace)
                    .unwrap();
            }
            "applyPatches" => {
                let path_to_original = path_to_original.expect(".env PATH_TO_ORIGINAL missing!");
                let path_to_workspace = path_to_workspace.expect(".env PATH_TO_WORKSPACE missing!");
                let path_to_patched = path_to_patched.expect(".env PATH_TO_PATCHED missing!");

                file_operations::bulk_apply_toml_patches(
                    &path_to_original,
                    &path_to_workspace,
                    &path_to_patched,
                )
                .unwrap();
            }
            "extractText" => {
                let path_to_original = path_to_original.expect(".env PATH_TO_ORIGINAL missing!");
                let path_to_workspace = path_to_workspace.expect(".env PATH_TO_WORKSPACE missing!");

                file_operations::bulk_extract_text(&path_to_original, &path_to_workspace).unwrap();
            }
            "importLegacyPatches" => {
                let path_to_workspace = path_to_workspace.expect(".env PATH_TO_WORKSPACE missing!");
                let path_to_legacy_workspace =
                    path_to_legacy_workspace.expect(".env PATH_TO_WORKSPACE_LEGACY missing!");

                file_operations::bulk_import_legacy_patches(
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
                let path_to_original = path_to_original.expect(".env PATH_TO_ORIGINAL missing!");

                let subpath_original_toml = Path::new(&path_to_original).join("original-toml");
                let subpath_reference = Path::new(&path_to_original).join("original-recompiled");
                let subpath_reference_toml =
                    Path::new(&path_to_original).join("original-recompiled-toml");
                let subpath_workspace_default =
                    Path::new(&path_to_original).join("patches-original");
                let subpath_workspace_custom = Path::new(&path_to_original).join("patches-custom");
                let subpath_patched_default =
                    Path::new(&path_to_original).join("original-default-patches");
                let subpath_patched_default_toml =
                    Path::new(&path_to_original).join("original-default-patches-toml");
                let subpath_patched_custom = Path::new(&path_to_original).join("translated-custom");
                let subpath_patched_custom_toml =
                    Path::new(&path_to_original).join("translated-custom-toml");

                println!(
                    "In order to test whether or not original binary read/write identical to original...\nCreate a git repository, put all files in \"<root>\" to \"<repo>/bin\" and all files in \"<root>/original-toml\" to \"<repo>/toml\".\nThen replace \"<repo>/bin\" = \"<root>/original-recompiled\" and \"<repo>/bin\" = \"<root>/original-recompiled-toml\".\nThen check \"git status | grep modified\".\n"
                );
                file_operations::bulk_redundant_serialize(&path_to_original, &subpath_reference)
                    .unwrap();
                file_operations::bulk_generate_toml_representations(
                    &path_to_original,
                    &subpath_original_toml,
                )
                .unwrap();
                file_operations::bulk_generate_toml_representations(
                    &subpath_reference,
                    &subpath_reference_toml,
                )
                .unwrap();

                println!("\nIn order to test whether or not patched binary (from default patches) identical to original...\nUsing the same git repo, replace \"<repo>/bin\" = \"<root>/original-default-patches\" and \"<repo>/bin\" = \"<root>/original-default-patches-toml\".\nThen check \"git status | grep modified\".\n");
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
                file_operations::bulk_generate_toml_representations(
                    &subpath_patched_default,
                    &subpath_patched_default_toml,
                )
                .unwrap();

                if let Ok(path_to_legacy_workspace) = path_to_legacy_workspace {
                    println!("\nIn order to test whether or not the already-patched binaries are identical (to legacy patches)...\nCreate a git repository, put all files in \"<translated-root>\" to \"<repo>/bin\" and all files in \"<translated-root>/original-toml\" to \"<repo>/toml\".\nThen replace \"<repo>/bin\" = \"<root>/translated-custom\" and \"<repo>/bin\" = \"<root>/translated-custom-toml\".\nThen check \"git status | grep modified\".\n");
                    file_operations::bulk_generate_toml_patches(
                        &path_to_original,
                        &subpath_workspace_custom,
                    )
                    .unwrap();
                    file_operations::bulk_import_legacy_patches(
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
                    file_operations::bulk_generate_toml_representations(
                        &subpath_patched_custom,
                        &subpath_patched_custom_toml,
                    )
                    .unwrap();
                } else {
                    println!("\nTo test if it's identical to an already-patched binary, set PATH_TO_WORKSPACE_LEGACY in .env!");
                }
            }
            "testApplyPatches" => {
                println!("Reapplying patch and automatically decompiling patched folder for easier debugging...");

                let path_to_original = path_to_original.expect(".env PATH_TO_ORIGINAL missing!");
                let path_to_workspace = path_to_workspace.expect(".env PATH_TO_WORKSPACE missing!");
                let path_to_patched = path_to_patched.expect(".env PATH_TO_PATCHED missing!");

                file_operations::bulk_apply_toml_patches(
                    &path_to_original,
                    &path_to_workspace,
                    &path_to_patched,
                )
                .unwrap();

                file_operations::bulk_generate_toml_representations(
                    &path_to_patched,
                    &path_to_patched,
                )
                .unwrap();
            }
            "test" => {
                println!("Testing...");

                let db =
                    fs::read("/home/watduhhekbro/external/workspace/debug/src/RPG_RT.lmt").unwrap();
                let mut reader = Cursor::new(db);
                let db = LcfMapTree::read_be(&mut reader).unwrap();
                fs::write("test.toml", db.generate_toml_maptree()).unwrap();
                //println!("{db:?}");

                let mut file = file_operations::overwrite(
                    "/home/watduhhekbro/external/workspace/debug/out/RPG_RT.lmt",
                )
                .unwrap();
                file.write_be(&db).unwrap();
                fs::write(
                    "/home/watduhhekbro/external/workspace/debug/out/MapTree.txt",
                    format!("{db:?}"),
                )
                .unwrap();
            }
            _ => {
                println!("Unknown command!");
            }
        },
        None => {
            println!("Available commands: decompile, generatePatches, applyPatches, extractText, importLegacyPatches");
        }
    }
}
