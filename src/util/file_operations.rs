use crate::structs::{LcfMapUnit, Patch};
use binrw::{io::Cursor, BinRead, BinWrite, BinWriterExt};
use std::{
    fs::{self, File},
    io,
    path::Path,
};

pub fn overwrite<S: AsRef<Path>>(path: S) -> io::Result<File> {
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

pub fn read_lcfmapunit<S: AsRef<Path>>(path: S) -> Result<LcfMapUnit, binrw::Error> {
    let file = fs::read(path)?;
    let mut reader = Cursor::new(file);
    let map = LcfMapUnit::read_be(&mut reader);
    map
}

pub fn read_lcfmapunit_and_patch<S: AsRef<Path>>(
    path_to_lcfmapunit: S,
    path_to_patch: S,
    path_to_patched_lcfmapunit: S,
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

// Actually, these operations are so fast that I don't even need to worry about implementing concurrency at all.

pub fn bulk_generate_toml_maps<S: AsRef<Path>>(
    path_to_original: S,
    path_to_reference: S,
) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_reference)?;

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

pub fn bulk_generate_toml_patches<S: AsRef<Path>>(
    path_to_original: S,
    path_to_workspace: S,
) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_workspace)?;

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
                let stringified_patch = map.generate_toml_patch(Some(&map_name));

                if stringified_patch.len() > 1 {
                    fs::write(toml_path, stringified_patch)?;
                }
            }
        }
    }

    Ok(())
}

pub fn bulk_apply_toml_patches<S: AsRef<Path>>(
    path_to_original: S,
    path_to_workspace: S,
    path_to_patched: S,
) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_workspace)?;
    fs::create_dir_all(&path_to_patched)?;

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

pub fn bulk_convert_legacy_patches<S: AsRef<Path>>(
    path_to_legacy_workspace: S,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// Purpose: Test if the LcfMapUnit in memory is identical to the raw binary output.
pub fn bulk_serialize_lcfmapunits<S: AsRef<Path>>(
    path_to_original: S,
    path_to_reference: S,
) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: Don't forget to create the leading directories if needed!
    fs::create_dir_all(&path_to_reference)?;

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
