mod dialogue; // Dialogue line parsing modules
mod structs; // Main structures
mod types; // Custom data types
mod util;
mod wrappers; // Intermediary data types for specialized functionality (upfront byte counts & null ID lists (similar to NullStrings))

use crate::util::file_operations;
use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    //let args = env::args().collect::<Vec<String>>();
    //if let Some(port) = args.get(1)

    let path_to_original = env::var("PATH_TO_ORIGINAL").unwrap();
    let path_to_workspace = env::var("PATH_TO_WORKSPACE").unwrap();
    let path_to_reference = env::var("PATH_TO_REFERENCE").unwrap();
    let path_to_patched = env::var("PATH_TO_PATCHED").unwrap();

    //file_operations::bulk_generate_toml_maps(&path_to_original, &path_to_reference).unwrap();
    //file_operations::bulk_generate_toml_patches(&path_to_original, &path_to_workspace).unwrap();
    /*file_operations::bulk_apply_toml_patches(
        &path_to_original,
        &path_to_workspace,
        &path_to_patched,
    )
    .unwrap();*/

    /*file_operations::bulk_serialize_lcfmapunits(
        &path_to_original,
        &"/home/watduhhekbro/external/workspace/dev2/".to_string(),
    )
    .unwrap();*/

    file_operations::bulk_generate_toml_maps(
        "/home/watduhhekbro/external/workspace/diff/2/",
        "/home/watduhhekbro/external/workspace/diff/2-toml/",
    )
    .unwrap();

    //file_operations::bulk_convert_legacy_patches(&path_to_legacy_workspace).unwrap();
}
