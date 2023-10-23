use crate::types::DynamicInteger;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use std::fmt;

pub struct U8Array(pub Vec<u8>);

impl BinRead for U8Array {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut bytes = vec![];
        let count = <DynamicInteger>::read_options(reader, endian, ())?;

        for _ in 0..*count {
            let byte = <u8>::read_options(reader, endian, ())?;
            bytes.push(byte);
        }

        Ok(Self(bytes))
    }
}

impl BinWrite for U8Array {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let count: i32 = self.0.len().try_into().expect(&format!(
            "The length of the u8 array {:?} could not fit into an i32!",
            self.0
        ));

        DynamicInteger(count).write_options(writer, endian, args)?;
        self.0.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl fmt::Debug for U8Array {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl std::ops::Deref for U8Array {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for U8Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{binrw, io::Cursor, BinWriterExt};

    #[binrw]
    #[br(big)]
    struct TestStructure(U8Array);

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x05Hello");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, b"Hello".to_vec());
    }

    #[test]
    fn write_success() {
        let data = TestStructure(U8Array(b"Yeetus".to_vec()));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x06Yeetus");
    }
}
