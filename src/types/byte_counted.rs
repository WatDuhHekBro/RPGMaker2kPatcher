use crate::{types::DynamicInteger, util::constants::*};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

// NOTE: Cannot use Vec's directly, must use a wrapper like DynamicIntegerArray!
#[derive(Debug, Deserialize, Serialize)]
pub struct ByteCounted<T: BinRead>(pub T);

impl<T: for<'a> BinRead<Args<'a> = ()>> BinRead for ByteCounted<T>
where
    T: BinRead,
    for<'a> T::Args<'a>: Default,
{
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let byte_count = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);
        let mut bytes: Vec<u8> = Vec::with_capacity(byte_count.0 as usize);

        for _ in 0..*byte_count {
            let byte = u8::read_be(reader).expect(ERROR_BINRW_READ);
            bytes.push(byte);
        }

        // This entire section will now operate on this section of bytes
        let mut reader = Cursor::new(&bytes);
        let inner = T::read_options(&mut reader, endian, args).expect(ERROR_BINRW_READ);

        Ok(ByteCounted(inner))
    }
}

impl<T> BinWrite for ByteCounted<T>
where
    //for<'a> T: BinRead + BinWrite<Args<'a> = ()>,
    /*T: for<'a> BinRead + for<'a> BinWrite,
    for<'a> T::Args<'a>: Default,*/
    for<'a> T: BinRead + BinWrite,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <T as BinWrite>::Args<'a>: Default,
{
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        // Create the subsection of bytes to count
        let mut subsection_writer = Cursor::new(Vec::<u8>::new());
        subsection_writer.write_type(&self.0, endian).unwrap();
        let bytes = subsection_writer.into_inner();

        // Then write the sub-section into the main section of bytes
        DynamicInteger(bytes.len().try_into().unwrap()).write_options(writer, endian, args)?;
        bytes.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for ByteCounted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for ByteCounted<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DynamicIntegerArray;
    use binrw::{binrw, io::Cursor};

    #[binrw]
    #[derive(Debug)]
    struct TestStructure {
        pub id: DynamicInteger,
        pub value: ByteCounted<DynamicIntegerArray>,
    }

    #[test]
    fn read_i32_be() {
        let mut reader = Cursor::new(b"\x04\x01\x02\x03\x04\x05");
        let data = ByteCounted::<i32>::read_be(&mut reader).unwrap();
        assert_eq!(data.0, 0x01020304);
    }

    #[test]
    fn read_i32_le() {
        let mut reader = Cursor::new(b"\x04\x01\x02\x03\x04\x05");
        let data = ByteCounted::<i32>::read_le(&mut reader).unwrap();
        assert_eq!(data.0, 0x04030201);
    }

    #[test]
    fn read_complex() {
        let mut reader = Cursor::new(b"\x01\x04\x03\x04\x05\x06\x07\x08\x09\x0A");
        let data = TestStructure::read_be(&mut reader).unwrap();
        println!("{data:?}");
    }

    #[test]
    #[should_panic]
    fn read_complex_byte_count_lower_than_should_be() {
        let mut reader = Cursor::new(b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A");
        let data = TestStructure::read_be(&mut reader).unwrap();
        println!("{data:?}");
    }

    /*#[test]
    #[should_panic]
    fn read_complex_byte_count_higher_than_should_be() {
        let mut reader = Cursor::new(b"\x01\x05\x03\x04\x05\x06\x07\x08\x09\x0A");
        let data = TestStructure::read_be(&mut reader).unwrap();
        println!("{data:?}");
    }*/

    #[test]
    fn write_i32_be() {
        let data = ByteCounted::<i32>(0x01020304);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x04\x01\x02\x03\x04");
    }

    #[test]
    fn write_i32_le() {
        let data = ByteCounted::<i32>(0x01020304);
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x04\x04\x03\x02\x01");
    }
}
