// A dynamic integer (count) followed by # of bytes as an array of DynamicIntegers

use crate::types::DynamicInteger;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use std::fmt;

pub struct DynamicIntegerArray(pub Vec<DynamicInteger>);

impl DynamicIntegerArray {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl BinRead for DynamicIntegerArray {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut numbers: Vec<DynamicInteger> = vec![];
        let count = <DynamicInteger>::read_options(reader, endian, ())?;

        for _ in 0..*count {
            let num = <DynamicInteger>::read_options(reader, endian, ())?;
            numbers.push(num);
        }

        Ok(Self(numbers))
    }
}

impl BinWrite for DynamicIntegerArray {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let count: i32 = self.0.len().try_into().expect(&format!(
            "The length of the DynamicInteger array {:?} could not fit into an i32!",
            self.0
        ));

        DynamicInteger(count).write_options(writer, endian, args)?;
        self.0.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl fmt::Debug for DynamicIntegerArray {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl std::ops::Deref for DynamicIntegerArray {
    type Target = Vec<DynamicInteger>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DynamicIntegerArray {
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
    struct TestStructure(DynamicIntegerArray);

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x07\x00\x86\x0E\x86\x0E\x00\x06\xCE\x15\x04\x69");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(data.0.len(), 7);
        assert_eq!(*data.0[0], 0x00);
        assert_eq!(*data.0[1], 0x06 * 0x80 + 0x0E);
        assert_eq!(*data.0[2], 0x06 * 0x80 + 0x0E);
        assert_eq!(*data.0[3], 0x00);
        assert_eq!(*data.0[4], 0x06);
        assert_eq!(*data.0[5], (0xCE - 0x80) * 0x80 + 0x15);
        assert_eq!(*data.0[6], 0x04);

        // The next byte should not be read if successfully bound by length
        //assert_eq!(*data.0[7], 0x69);
    }

    #[test]
    fn write_success() {
        let data = TestStructure(DynamicIntegerArray(vec![
            DynamicInteger(0x00),
            DynamicInteger(0x06 * 0x80 + 0x0E),
            DynamicInteger(0x06 * 0x80 + 0x0E),
            DynamicInteger(0x00),
            DynamicInteger(0x06),
            DynamicInteger((0xCE - 0x80) * 0x80 + 0x15),
            DynamicInteger(0x04),
        ]));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(
            writer.into_inner(),
            b"\x07\x00\x86\x0E\x86\x0E\x00\x06\xCE\x15\x04"
        );
    }

    #[test]
    fn write_success2() {
        let data = TestStructure(DynamicIntegerArray(vec![
            DynamicInteger(0x00),
            DynamicInteger(0x06 * 0x80 + 0x0E),
            DynamicInteger(0x06 * 0x80 + 0x0E),
            DynamicInteger(0x00),
            DynamicInteger(0x06),
            DynamicInteger((0xCE - 0x80) * 0x80 + 0x15),
            DynamicInteger(0x04),
            DynamicInteger(0x69),
        ]));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(
            writer.into_inner(),
            b"\x08\x00\x86\x0E\x86\x0E\x00\x06\xCE\x15\x04\x69"
        );
    }

    #[test]
    fn is_empty_should_succeed() {
        let mut reader = Cursor::new(b"\x00\x00\x86\x0E\x86\x0E\x00\x06\xCE\x15\x04\x69");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(data.0.is_empty(), true);
    }

    #[test]
    fn is_empty_should_fail() {
        let mut reader = Cursor::new(b"\x07\x00\x86\x0E\x86\x0E\x00\x06\xCE\x15\x04\x69");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(data.0.is_empty(), false);
    }
}
