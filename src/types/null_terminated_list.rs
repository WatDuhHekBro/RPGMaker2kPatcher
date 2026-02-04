// Very nicely modular type that just loops through arbitrary data types, with the
// list ending if exactly one byte 0x00 is present (and ONLY reading that byte if so).
// -----
// Used mainly for top-level file headers and event command lists.

use crate::util::constants::ERROR_BINRW_READ;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NullTerminatedList<T: BinRead>(pub Vec<T>);

impl<T: for<'a> BinRead<Args<'a> = ()>> BinRead for NullTerminatedList<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut entries: Vec<T> = Vec::new();

        loop {
            let mut next_byte = [0u8];
            reader.read_exact(&mut next_byte).expect(ERROR_BINRW_READ);

            if next_byte[0] == 0x00 {
                // This will mean that the zero byte counts as read by this point
                break;
            } else {
                // First rewind because stepping here means there's actual data now
                reader.seek_relative(-1).expect(ERROR_BINRW_READ);

                // Then read the arbitrary data entry
                let entry = T::read_options(reader, endian, args).expect(ERROR_BINRW_READ);
                entries.push(entry);
            }
        }

        Ok(NullTerminatedList(entries))
    }
}

impl<T: BinRead + BinWrite + 'static> BinWrite for NullTerminatedList<T>
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
        0u8.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for NullTerminatedList<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for NullTerminatedList<T> {
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
        let mut reader = Cursor::new(b"\x02\x04\x06\x00\x01");
        let data = NullTerminatedList::<u8>::read_be(&mut reader).unwrap();
        assert_eq!(data[0], 2);
        assert_eq!(data[1], 4);
        assert_eq!(data[2], 6);
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn write_success() {
        let data: NullTerminatedList<u8> = NullTerminatedList(vec![2, 4, 6]);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x02\x04\x06\x00");
    }

    #[test]
    fn read_empty_success() {
        let mut reader = Cursor::new(b"\x00\x02\x04\x06\x00\x01");
        let data = NullTerminatedList::<u8>::read_be(&mut reader).unwrap();
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn write_empty_success() {
        let data: NullTerminatedList<u8> = NullTerminatedList(vec![]);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x00");
    }
}
