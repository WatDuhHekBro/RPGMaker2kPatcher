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
