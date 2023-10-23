// A byte string that uses a length specifier. Can use different encodings.

use crate::types::DynamicInteger;
use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use encoding::{all::WINDOWS_1252, DecoderTrap, EncoderTrap, Encoding};
use std::fmt;

// Before using a font patch, Windows-1251 is used by default
// If you wanted to, you could encode Cyrillic characters during the serialization step, but for simplicity, I won't
pub struct PascalString(String);

impl PascalString {
    pub fn from<S: Into<String>>(string: S) -> PascalString {
        PascalString(string.into())
    }
}

impl BinRead for PascalString {
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

        Ok(Self(
            WINDOWS_1252
                .decode(&bytes, DecoderTrap::Strict)
                .expect("Invalid Windows-1252 string found while decoding/reading."),
        ))
    }
}

impl BinWrite for PascalString {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        // Using String::len() will cause it to miscount the characters
        let count: i32 = self.0.chars().count().try_into().expect(&format!(
            "The length of the PascalString {} could not fit into an i32!",
            self.0
        ));

        DynamicInteger(count).write_options(writer, endian, args)?;

        WINDOWS_1252
            .encode(&self.0, EncoderTrap::Strict)
            .expect("Invalid Windows-1252 string found while encoding/writing.")
            .write_options(writer, endian, args)?;

        Ok(())
    }
}

impl fmt::Debug for PascalString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "\"{}\"", self.0)
    }
}

impl fmt::Display for PascalString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::ops::Deref for PascalString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PascalString {
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
    struct TestStructure(PascalString);

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x05Hello");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, "Hello");
    }

    #[test]
    #[should_panic]
    fn read_excess_length() {
        let mut reader = Cursor::new(b"\x06Hello");
        TestStructure::read(&mut reader).unwrap();
    }

    #[test]
    fn read_not_enough_length() {
        let mut reader = Cursor::new(b"\x03Hello");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, "Hel");
    }

    #[test]
    fn write_success() {
        let data = TestStructure(PascalString::from("Yeetus"));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x06Yeetus");
    }

    #[test]
    fn write_excess_length() {
        let data = TestStructure(PascalString::from("Yeetus"));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_ne!(writer.into_inner(), b"\x07Yeetus");
    }

    #[test]
    fn write_not_enough_length() {
        let data = TestStructure(PascalString::from("Yeetus"));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_ne!(writer.into_inner(), b"\x05Yeetus");
    }

    #[test]
    fn read_extended_ascii_success() {
        let mut reader = Cursor::new(b"\x03\x48\xF6\x72");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, "Hör");
    }

    #[test]
    fn write_extended_ascii_success() {
        let data = TestStructure(PascalString::from("Hör"));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x03\x48\xF6\x72");
    }

    #[test]
    fn read_long_string() {
        let mut reader = Cursor::new(b"\x81\x00Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed a elementum lorem. Curabitur auctor, justo nec fringilla porttitor.");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed a elementum lorem. Curabitur auctor, justo nec fringilla porttitor.");
    }

    #[test]
    fn write_long_string() {
        let data = TestStructure(PascalString::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed a elementum lorem. Curabitur auctor, justo nec fringilla porttitor."));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x81\x00Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed a elementum lorem. Curabitur auctor, justo nec fringilla porttitor.");
    }
}
