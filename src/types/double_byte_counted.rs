// The purpose of this struct is because of redundant byte count fields.
// LcfMapUnit: 81.5.51 (redundant bytecount) and 81.5.52 (bytecount for commands)
// LcfDataBase: 25.21 (redundant bytecount) and 25.22 (bytecount for commands)
// -----
// ~~Unless needed, just assume the next ID is +1 and consume both fields.~~ (Actually, just store the next ID.)
// To use this, DoubleByteCounted<> with magic number of the redundant field (map.51) and (db.21)
// -----
// 33 02 81 13 (147 Bytes)
// 34 81 13 (147 Bytes)

use crate::{types::DynamicInteger, util::constants::*};
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

// NOTE: Cannot use Vec's directly, must use a wrapper like DynamicIntegerArray!
#[derive(Debug, Deserialize, Serialize)]
pub struct DoubleByteCounted<T: BinRead> {
    pub inner: T,
    pub next_id: DynamicInteger,
}

impl<T: for<'a> BinRead<Args<'a> = ()>> BinRead for DoubleByteCounted<T>
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
        let redundant_byte_count_length = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);
        let redundant_byte_count = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);
        let next_id = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);
        let byte_count = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);

        // May as well do some error checking while we're here
        if *redundant_byte_count_length != redundant_byte_count.size() as i32 {
            panic!("Redundant byte count doesn't match the amount of bytes that were specified!");
        }
        if *byte_count != *redundant_byte_count {
            panic!("Redundant byte count doesn't match the next field's byte count!");
        }

        // Read the next field's byte count instead
        let mut bytes: Vec<u8> = Vec::with_capacity(*byte_count as usize);

        for _ in 0..*byte_count {
            let byte = u8::read_be(reader).expect(ERROR_BINRW_READ);
            bytes.push(byte);
        }

        // This entire section will now operate on this section of bytes
        let mut reader = Cursor::new(&bytes);
        let inner = T::read_options(&mut reader, endian, args).expect(ERROR_BINRW_READ);

        Ok(DoubleByteCounted { inner, next_id })
    }
}

impl<T> BinWrite for DoubleByteCounted<T>
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
        (): Self::Args<'_>,
    ) -> BinResult<()> {
        // Create the subsection of bytes to count
        let mut subsection_writer = Cursor::new(Vec::<u8>::new());
        subsection_writer.write_type(&self.inner, endian).unwrap();
        let bytes = subsection_writer.into_inner();
        let bytes_count = DynamicInteger(bytes.len().try_into().unwrap());

        // First write 0x33 as the redundant byte count ID
        // NOTE: Actually you DON'T, it's already there from the binrw magic number.
        //0x33u8.write_be(writer)?;
        // Then write the size of the bytes count (I know)
        bytes_count.size().write_be(writer)?;
        // Then the byte count (I know)
        bytes_count.write_be(writer)?;
        // Then 0x34
        let _ = &self.next_id.write_be(writer)?;
        // Then the byte count again (Yep... I know)
        bytes_count.write_be(writer)?;
        // Then finally write the sub-section into the main section of bytes
        bytes.write_be(writer)?;

        Ok(())
    }
}

impl<T: BinRead + BinWrite> std::ops::Deref for DoubleByteCounted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: BinRead + BinWrite> std::ops::DerefMut for DoubleByteCounted<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PascalString;
    use binrw::io::Cursor;

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x01\x06\x34\x06\x05Hello\x7F\x7F");
        let data = DoubleByteCounted::<PascalString>::read_be(&mut reader).unwrap();
        assert_eq!(**data, "Hello");
    }

    #[test]
    fn write_success() {
        let data = DoubleByteCounted {
            inner: PascalString::from("Hello"),
            next_id: DynamicInteger(0x7F),
        };
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_be(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x01\x06\x7F\x06\x05Hello");
    }

    #[test]
    #[should_panic]
    fn should_panic_at_the_disco_length_prefix_mismatch() {
        let mut reader = Cursor::new(b"\x02\x06\x34\x06\x05Hello\x7F\x7F");
        let data = DoubleByteCounted::<PascalString>::read_be(&mut reader).unwrap();
        assert_eq!(**data, "Hello");
    }

    #[test]
    #[should_panic]
    fn should_panic_at_the_disco_byte_count_mismatch() {
        let mut reader = Cursor::new(b"\x01\x06\x34\x01\x05Hello\x7F\x7F");
        let data = DoubleByteCounted::<PascalString>::read_be(&mut reader).unwrap();
        assert_eq!(**data, "Hello");
    }

    #[test]
    #[should_panic]
    fn should_panic_at_the_disco_wrong_byte_count() {
        let mut reader = Cursor::new(b"\x01\x04\x34\x04\x05Hello\x7F\x7F");
        let data = DoubleByteCounted::<PascalString>::read_be(&mut reader).unwrap();
        assert_eq!(**data, "Hello");
    }
}
