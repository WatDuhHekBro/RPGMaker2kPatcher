use crate::{
    structs::{LcfDataBase, LcfMapTree, LcfMapUnit, LegacyDatabasePatch, LegacyMapPatch, Patch},
    util::{
        self,
        constants::{ERROR_NO_FILE_DATABASE, ERROR_NO_FILE_MAPTREE},
    },
};
use binrw::{io::Cursor, BinRead, BinWrite, BinWriterExt};
use std::{
    fs::{self, File},
    io,
    path::Path,
};

pub fn overwrite<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}

pub fn hexdump<T: BinWrite>(map: T) -> String
where
    for<'a> <T as BinWrite>::Args<'a>: Default,
{
    let mut writer = Cursor::new(Vec::<u8>::new());
    writer.write_be(&map).unwrap();
    format!("{:02X?}", writer.into_inner())
}

pub fn read_lcfmapunit<P: AsRef<Path>>(path: P) -> Result<LcfMapUnit, binrw::Error> {
    let file = fs::read(path)?;
    let mut reader = Cursor::new(file);
    let map = LcfMapUnit::read_be(&mut reader);
    map
}

// Assume that only one database exists, named "RPG_RT.ldb" in the same directory.
pub fn read_lcfdatabase<P: AsRef<Path>>(path: P) -> Result<LcfDataBase, binrw::Error> {
    let file = fs::read(path).expect(ERROR_NO_FILE_DATABASE);
    let mut reader = Cursor::new(file);
    let database = LcfDataBase::read_be(&mut reader);
    database
}

// Assume that only one map tree exists, named "RPG_RT.lmt" in the same directory.
pub fn read_lcfmaptree<P: AsRef<Path>>(path: P) -> Result<LcfMapTree, binrw::Error> {
    let file = fs::read(path).expect(ERROR_NO_FILE_MAPTREE);
    let mut reader = Cursor::new(file);
    let maptree = LcfMapTree::read_be(&mut reader);
    maptree
}

pub fn read_lcfmapunit_and_patch<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
    path_to_lcfmapunit: P1,
    path_to_patch: P2,
    path_to_patched_lcfmapunit: P3,
) -> Result<LcfMapUnit, Box<dyn std::error::Error>> {
    // Read map
    let file_map = fs::read(path_to_lcfmapunit)?;
    let mut reader = Cursor::new(file_map);
    let mut map = LcfMapUnit::read_be(&mut reader)?;

    // Read patch
    let patch_file_string = &fs::read_to_string(path_to_patch)?;
    let mut patch = toml::from_str::<Patch>(patch_file_string)?;
    patch.trim_dialogue_ending_newline();

    // Apply patch to map
    map.apply_patch(&patch);

    // Write the patched map
    let mut patched_output_file = overwrite(path_to_patched_lcfmapunit)?;
    patched_output_file.write_be(&map)?;

    Ok(map)
}

pub fn read_lcfdatabase_and_patch<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
    path_to_lcfdatabase: P1,
    path_to_patch: P2,
    path_to_patched_lcfdatabase: P3,
) -> Result<LcfDataBase, Box<dyn std::error::Error>> {
    // Read database
    let file_map = fs::read(path_to_lcfdatabase)?;
    let mut reader = Cursor::new(file_map);
    let mut database = LcfDataBase::read_be(&mut reader)?;

    // Read patch
    let patch_file_string = &fs::read_to_string(path_to_patch)?;
    let mut patch = toml::from_str::<Patch>(patch_file_string)?;
    patch.trim_dialogue_ending_newline();

    // Apply patch to map
    database.apply_patch(&patch);

    // Write the patched map
    let mut patched_output_file = overwrite(path_to_patched_lcfdatabase)?;
    patched_output_file.write_be(&database)?;

    Ok(database)
}

// Actually, these operations are so fast that I don't even need to worry about implementing concurrency at all.

pub fn bulk_generate_toml_representations<P1: AsRef<Path>, P2: AsRef<Path>>(
    path_to_original: P1,
    path_to_reference: P2,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulk generating TOML representations...");

    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_reference)?;

    // Read the map tree first (for portrait conditions), assume hardcoded path and only one map tree.
    let path_to_maptree = path_to_original.as_ref().join("RPG_RT.lmt");
    let maptree = read_lcfmaptree(path_to_maptree)?;

    fs::write(
        path_to_reference.as_ref().join("MapTree.toml"),
        maptree.generate_toml_maptree(),
    )?;

    // Read the database second, assume hardcoded path and only one database.
    let path_to_database = path_to_original.as_ref().join("RPG_RT.ldb");
    let database = read_lcfdatabase(path_to_database)?;

    fs::write(
        path_to_reference.as_ref().join("Database.toml"),
        database.generate_toml_database(),
    )?;

    for entry in fs::read_dir(path_to_original)? {
        let entry = entry?;
        // "/path/to/original/Map0134.lmu"
        let path = entry.path();
        // "lmu"
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension == "lmu" {
                // "Map0134"
                let file_stem = path
                    .file_stem()
                    .expect("If Some(extension) exists, why doesn't file_stem exist?!");

                let map = read_lcfmapunit(&path)?;

                // "/path/to/reference/Map0134.toml"
                let mut toml_path = path_to_reference.as_ref().join(file_stem);
                toml_path.set_extension("toml");

                // Write
                fs::write(toml_path, map.generate_toml_map())?;
            }
        }
    }

    Ok(())
}

pub fn bulk_generate_toml_patches<P1: AsRef<Path>, P2: AsRef<Path>>(
    path_to_original: P1,
    path_to_workspace: P2,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulk generating TOML patches...");

    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_workspace)?;

    // Read the map tree first (for portrait conditions), assume hardcoded path and only one map tree.
    let path_to_maptree = path_to_original.as_ref().join("RPG_RT.lmt");
    let maptree = read_lcfmaptree(path_to_maptree)?;
    let game_title = maptree.extract_game_title();

    // You need to read the database before reading any maps for the character names!
    let path_to_database = path_to_original.as_ref().join("RPG_RT.ldb");
    let database = read_lcfdatabase(path_to_database)?;
    let character_names = database.extract_character_names();

    // May as well get the database patch done first while you're here.
    fs::write(
        path_to_workspace.as_ref().join("Database.patch.toml"),
        database.generate_toml_patch(game_title),
    )?;

    for entry in fs::read_dir(path_to_original)? {
        let entry = entry?;
        // "/path/to/original/Map0134.lmu"
        let path = entry.path();
        // "lmu"
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension == "lmu" {
                // "Map0134"
                let file_stem = path
                    .file_stem()
                    .expect("If Some(extension) exists, why doesn't file_stem exist?!");

                let map = read_lcfmapunit(&path)?;

                // "/path/to/workspace/Map0134.patch.toml"
                let mut toml_path = path_to_workspace.as_ref().join(file_stem);
                toml_path.set_extension("patch.toml");

                // Only write the patch if it isn't an empty file.
                let map_name = file_stem
                    .to_str()
                    .expect("OsStr conversion to String failed!")
                    .to_string();
                let stringified_patch =
                    map.generate_toml_patch(&character_names, &map_name, game_title);

                if stringified_patch.len() > 1 {
                    fs::write(toml_path, stringified_patch)?;
                }
            }
        }
    }

    Ok(())
}

pub fn bulk_apply_toml_patches<P1: AsRef<Path>, P2: AsRef<Path>, P3: AsRef<Path>>(
    path_to_original: P1,
    path_to_workspace: P2,
    path_to_patched: P3,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulk applying TOML patches...");

    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_workspace)?;
    fs::create_dir_all(&path_to_patched)?;

    // Read the database first, assume hardcoded path and only one database.
    let path_to_lcfdatabase = path_to_original.as_ref().join("RPG_RT.ldb");
    let path_to_patch = path_to_workspace.as_ref().join("Database.patch.toml");
    let path_to_patched_lcfdatabase = path_to_patched.as_ref().join("RPG_RT.ldb");

    read_lcfdatabase_and_patch(
        path_to_lcfdatabase,
        path_to_patch,
        path_to_patched_lcfdatabase,
    )
    .unwrap();

    for entry in fs::read_dir(path_to_original)? {
        let entry = entry?;
        // "/path/to/original/Map0134.lmu"
        let path = entry.path();
        // "lmu"
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension == "lmu" {
                // "Map0134"
                let file_stem = path
                    .file_stem()
                    .expect("If Some(extension) exists, why doesn't file_stem exist?!");

                // "/path/to/workspace/Map0134.patch.toml"
                let mut toml_path = path_to_workspace.as_ref().join(file_stem);
                toml_path.set_extension("patch.toml");

                // NOTE: You MUST check if a TOML patch exists, because it might not!
                // Path::exists() is used over fs::exists() because I'm only concerned about whether or not the file is accessible.
                // TODO: Fix TOCTOU error... but oh well.
                if toml_path.exists() {
                    // "/path/to/patched/Map0134.lmu"
                    let mut patched_path = path_to_patched.as_ref().join(file_stem);
                    patched_path.set_extension("lmu");

                    // Write
                    read_lcfmapunit_and_patch(&path, &toml_path, &patched_path)?;
                }
            }
        }
    }

    Ok(())
}

pub fn bulk_extract_text<P1: AsRef<Path>, P2: AsRef<Path>>(
    path_to_original: P1,
    path_to_workspace: P2,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulk extracting text...");

    // NOTE: Don't forget to create the leading directories if needed!
    let path_to_extracted = path_to_workspace.as_ref().join("extracted");
    fs::create_dir_all(&path_to_extracted)?;

    // You need to read the database before reading any maps for the character names!
    let path_to_database = path_to_original.as_ref().join("RPG_RT.ldb");
    let database = read_lcfdatabase(path_to_database)?;
    let character_names = database.extract_character_names();

    // You may as well extract the database text while you're here.
    let mut patch = toml::from_str::<Patch>(&fs::read_to_string(
        path_to_workspace.as_ref().join("Database.patch.toml"),
    )?)?;
    patch.trim_dialogue_ending_newline();

    fs::write(
        &path_to_extracted.join("Database.patch.txt"),
        patch.extract_text(&character_names),
    )?;

    for entry in fs::read_dir(path_to_workspace)? {
        let entry = entry?;
        // "/path/to/workspace/Map0134.patch.toml"
        let path = entry.path();
        // "toml"
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension == "toml" {
                // Read patch
                let mut patch = toml::from_str::<Patch>(&fs::read_to_string(&path)?)?;
                patch.trim_dialogue_ending_newline();

                // "Map0134.patch"
                let file_stem = path
                    .file_stem()
                    .expect("If Some(extension) exists, why doesn't file_stem exist?!");
                // "/path/to/workspace/extracted/Map0134.patch.txt"
                let mut text_path = path_to_extracted.join(file_stem);
                text_path.set_extension("patch.txt");
                fs::write(&text_path, patch.extract_text(&character_names))?;
            }
        }
    }

    Ok(())
}

pub fn bulk_import_legacy_patches<P1: AsRef<Path>, P2: AsRef<Path>>(
    path_to_workspace: P1,
    path_to_legacy_workspace: P2,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulk converting legacy patches...");

    // Read database patch
    let toml_path = path_to_workspace.as_ref().join("Database.patch.toml");
    let mut patch = toml::from_str::<Patch>(&fs::read_to_string(&toml_path)?)?;
    patch.trim_dialogue_ending_newline();

    // Read legacy database patch
    let text = fs::read_to_string(
        path_to_legacy_workspace
            .as_ref()
            .join("database.patch.json"),
    )?;
    let legacy_patch: LegacyDatabasePatch = serde_json::from_str(&text)?;

    // Patch and write
    legacy_patch.import_lines_to_toml_database_patch(&mut patch);
    fs::write(&toml_path, util::generate_toml_patch(&patch))?;

    for entry in fs::read_dir(path_to_legacy_workspace)? {
        let entry = entry?;
        // "/path/to/path_to_legacy_workspace/Map0134.patch.json"
        let path = entry.path();
        // "json"
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension == "json" {
                // "Map0134"
                let file_prefix = path
                    .file_prefix()
                    .expect("If Some(extension) exists, why doesn't file_prefix exist?!");
                let map_name = file_prefix
                    .to_str()
                    .expect("OsStr conversion to String failed!")
                    .to_string();

                // "/path/to/workspace/Map0134.patch.toml"
                let mut toml_path = path_to_workspace.as_ref().join(file_prefix);
                toml_path.set_extension("patch.toml");

                // Read patch
                let patch_file_string = &fs::read_to_string(&toml_path);

                // Just in case there's an added legacy patch with no generated equivalent:
                if let Ok(patch_file_string) = patch_file_string {
                    let mut patch = toml::from_str::<Patch>(patch_file_string)?;
                    patch.trim_dialogue_ending_newline();

                    // Read legacy patch
                    let text = fs::read_to_string(path)?;
                    let legacy_patch: LegacyMapPatch = serde_json::from_str(&text)?;

                    // Patch and write
                    legacy_patch.import_lines_to_toml_map_patch(&mut patch, &map_name);
                    fs::write(&toml_path, util::generate_toml_patch(&patch))?;
                } else if map_name != "database" {
                    println!("ERROR: Error on reading TOML patch file for {map_name}!");
                }
            }
        }
    }

    Ok(())
}

// Purpose: Test if the LcfMapUnit in memory is identical to the raw binary output.
pub fn bulk_redundant_serialize<P1: AsRef<Path>, P2: AsRef<Path>>(
    path_to_original: P1,
    path_to_reference: P2,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Bulk serializing in-memory LcfMapUnits...");

    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_reference)?;

    // Read the database first, assume hardcoded path and only one database.
    let path_to_database = path_to_original.as_ref().join("RPG_RT.ldb");
    let database = read_lcfdatabase(path_to_database)?;

    let mut output_file = overwrite(path_to_reference.as_ref().join("RPG_RT.ldb"))?;
    output_file.write_be(&database)?;

    for entry in fs::read_dir(path_to_original)? {
        let entry = entry?;
        // "/path/to/original/Map0134.lmu"
        let path = entry.path();
        // "lmu"
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension == "lmu" {
                // "Map0134"
                let file_stem = path
                    .file_stem()
                    .expect("If Some(extension) exists, why doesn't file_stem exist?!");

                let map = read_lcfmapunit(&path)?;

                // "/path/to/reference/Map0134.lmu"
                let mut new_lmu_path = path_to_reference.as_ref().join(file_stem);
                new_lmu_path.set_extension("lmu");

                // Write
                let mut output_file = overwrite(new_lmu_path)?;
                output_file.write_be(&map)?;
            }
        }
    }

    Ok(())
}
