// Really unorthodox edge case for LcfDataBase.
// Functions just like a NullTerminatedList except instead of 0x00, it's the EOF.
// -----
// My old JS code did something similar, albeit in a much more obtuse way.
// writer.js::createStart() => "//writer.writeInt8(0); // I don't know why the database doesn't have that last byte at the end."
// general.js::handleData() => "if(!isDatabase) download(new Uint8Array(createStart(data, MAP).concat(0)), filename + '.lmu');"

use crate::util::constants::ERROR_BINRW_READ;
use binrw::{
    io::{Read, Seek, SeekFrom, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileTerminatedList<T: BinRead>(pub Vec<T>);

impl<T: for<'a> BinRead<Args<'a> = ()>> BinRead for FileTerminatedList<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut entries: Vec<T> = Vec::new();

        // You need to return to your original position after reading the magic bytes
        // Notably, this is NOT the start position
        let file_size = {
            let original_position = reader.stream_position().expect(ERROR_BINRW_READ);

            let file_size = reader.seek(SeekFrom::End(0)).expect(ERROR_BINRW_READ);

            reader
                .seek(SeekFrom::Start(original_position))
                .expect(ERROR_BINRW_READ);

            file_size
        };

        loop {
            let current_position = reader.stream_position().expect(ERROR_BINRW_READ);

            if current_position == file_size {
                break;
            } else if current_position > file_size {
                panic!("current_position > file_size ?! Overshot...");
            } else {
                let entry = T::read_options(reader, endian, args).expect(ERROR_BINRW_READ);
                entries.push(entry);
            }
        }

        Ok(FileTerminatedList(entries))
    }
}

impl<T: BinRead + BinWrite + 'static> BinWrite for FileTerminatedList<T>
where
    for<'a> T: BinRead + BinWrite<Args<'a> = ()>,
    /*for<'a> T: BinRead + BinWrite,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <T as BinWrite>::Args<'a>: Default,*/
{
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.0.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for FileTerminatedList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for FileTerminatedList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{io::Cursor, BinWriterExt};

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x02\x04\x06");
        let data = FileTerminatedList::<u8>::read_be(&mut reader).unwrap();
        assert_eq!(data[0], 2);
        assert_eq!(data[1], 4);
        assert_eq!(data[2], 6);
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn write_success() {
        let data: FileTerminatedList<u8> = FileTerminatedList(vec![2, 4, 6]);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x02\x04\x06");
    }
}
