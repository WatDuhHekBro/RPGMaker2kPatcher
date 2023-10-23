// 32-bit signed integer that can be represented by 1-5 BE bytes, depending on length.
// -----------------------------------------------------------------------------------
// [c000 s???] [c??? ????] [c??? ????] [c??? ????] [c??? ????]
// c = read next byte (continue), s = sign, ? = value

use binrw::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use std::fmt;

// TODO: Option --> Result? unwrap in the class itself
// Assume continue bits are already checked
// Meaning that the most sigificant bit is 1 unless it's the last byte
fn read_dynamic_integer(digits: Vec<u8>) -> Option<i32> {
    if digits.len() > 5 {
        return None;
    }

    let mut result: i32 = 0;

    // Debug Pretty Print
    //println!("0b...7...6...5...4...3...2...1...0");

    for (index, digit) in digits.iter().rev().enumerate() {
        // Convert u8 to i32 (0-255)
        let mut digit = *digit as i32;

        // If it's the 1st byte of a 5-byte representation, zero out additional bits
        if index == 4 {
            digit &= 0b0000_1111;
        }
        // Otherwise, just zero out the continue bit
        else {
            digit &= 0b0111_1111;
        }

        // Shift it to the correct position (by 7 bits each)
        digit <<= index * 7;

        // Then add those bits onto the resulting integer
        result |= digit;

        // Debug Pretty Print
        //println!("{result:#034b} ({result})");
    }

    Some(result)
}

fn write_dynamic_integer(number: i32) -> Vec<u8> {
    let mut bytes: [u8; 5] = [
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b1000_0000,
        0b0000_0000, // no continue bit
    ];

    // For each section of the number, take a 7-bit bitmask (or 4-bit for the 1st byte of a 5-byte representation)
    // First however, cast the number to a u32 in order to shift the sign bit (unsigned right shift >>>)
    let number = number as u32;

    // 1st byte
    bytes[0] |= ((number & (0b0000_1111 << 28)) >> 28) as u8; // This part would have problems with an i32

    // 2nd byte
    bytes[1] |= ((number & (0b0111_1111 << 21)) >> 21) as u8;

    // 3rd byte
    bytes[2] |= ((number & (0b0111_1111 << 14)) >> 14) as u8;

    // 4th byte
    bytes[3] |= ((number & (0b0111_1111 << 7)) >> 7) as u8;

    // 5th byte
    bytes[4] = (number & 0b0111_1111) as u8;

    // Debug
    //println!("{} = {bytes:?}", number as i32);

    // Finally, loop through each byte to build the vector, ignoring unnecessary bytes
    let mut result: Vec<u8> = Vec::new();
    let mut is_adding_bytes = false;

    for byte in bytes {
        // Start adding bytes once you find a byte that isn't 0x80
        if !is_adding_bytes && byte != 0b1000_0000 {
            is_adding_bytes = true;
        }

        // Don't put in an "else" clause if you want to count the current byte (given that it's the start)
        if is_adding_bytes {
            result.push(byte);
        }
    }

    result
}

// TODO: Maybe find a way to pass args from DynamicInteger? (# of bytes as u8 (1-5))

pub struct DynamicInteger(pub i32);

impl DynamicInteger {
    // # of bytes as u8 (1-5)
    // Not as elegant as storing the length inside the struct itself, but you only need to use this in a few scenarios (byte count alignment)
    pub fn size(&self) -> u8 {
        write_dynamic_integer(self.0).len() as u8
    }
}

impl BinRead for DynamicInteger {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut bytes = vec![];
        let mut is_terminated = false;

        for _ in 0..5 {
            let byte = <u8>::read_options(reader, endian, ())?;
            bytes.push(byte);

            // Evaluate the continue bit
            if ((byte & 0b1000_0000) >> 7) == 0 {
                is_terminated = true;
                break;
            }
        }

        // If it's greater than 5 bytes and hasn't terminated yet, throw an error.
        if !is_terminated {
            panic!("[FATAL] Invalid dynamic integer: {bytes:?}");
        }

        Ok(Self(read_dynamic_integer(bytes).unwrap()))
    }
}

impl BinWrite for DynamicInteger {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let bytes = write_dynamic_integer(self.0);
        bytes.write_options(writer, endian, args)?;
        Ok(())
    }
}

impl fmt::Debug for DynamicInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for DynamicInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::ops::Deref for DynamicInteger {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DynamicInteger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{binrw, io::Cursor, BinWriterExt};

    #[test]
    fn core_read_known_values() {
        assert_eq!(read_dynamic_integer(vec![0x64]), Some(100));
        assert_eq!(read_dynamic_integer(vec![0x81, 0x08]), Some(136));
        assert_eq!(read_dynamic_integer(vec![0x82, 0x40]), Some(320));
        assert_eq!(read_dynamic_integer(vec![0x86, 0x0E]), Some(782));
        assert_eq!(read_dynamic_integer(vec![0x86, 0x0F]), Some(783));
        assert_eq!(read_dynamic_integer(vec![0xCE, 0x15]), Some(10005));
        assert_eq!(read_dynamic_integer(vec![0xCF, 0x6C]), Some(10220));
        assert_eq!(read_dynamic_integer(vec![0xD6, 0x66]), Some(11110));
        assert_eq!(read_dynamic_integer(vec![0xD9, 0x12]), Some(11410));
        assert_eq!(read_dynamic_integer(vec![0xDE, 0x4E]), Some(12110));
        assert_eq!(read_dynamic_integer(vec![0xDE, 0x58]), Some(12120));
        assert_eq!(read_dynamic_integer(vec![0x81, 0xAB, 0x7A]), Some(22010));
        assert_eq!(read_dynamic_integer(vec![0x81, 0xAF, 0x0A]), Some(22410));
        assert_eq!(
            read_dynamic_integer(vec![0x93, 0xBC, 0xD0, 0x30]),
            Some(40839216)
        );
        assert_eq!(
            read_dynamic_integer(vec![0x85, 0x91, 0xA1, 0xE4, 0x75]),
            Some(1378382453)
        );
        assert_eq!(
            read_dynamic_integer(vec![0x8F, 0xFF, 0xFF, 0xFF, 0x7F]),
            Some(-1)
        ); // =/= 4294967295
        assert_eq!(
            read_dynamic_integer(vec![0x8F, 0xFF, 0xFF, 0xFF, 0x35]),
            Some(-75)
        );
        assert_eq!(
            read_dynamic_integer(vec![0x8F, 0xFF, 0xFF, 0xFF, 0x21]),
            Some(-95)
        ); // =/= 4294967201
        assert_eq!(
            read_dynamic_integer(vec![0x8C, 0x93, 0xFC, 0x86, 0x5F]),
            Some(-1031863457)
        ); // =/= 3263103839
        assert_eq!(
            read_dynamic_integer(vec![0x88, 0x80, 0x80, 0x80, 0x08]),
            Some(-2147483640)
        );
    }

    // Regex Replace Read (VSCode)
    // assert_eq!\([\s\n]*read_dynamic_integer\((vec!\[.+?\])\),[\s\n]*Some\((-?\d+)\)[\s\n]*\);(.+)?
    // assert_eq!(write_dynamic_integer($2), $1);$3
    #[test]
    fn core_write_known_values() {
        assert_eq!(write_dynamic_integer(100), vec![0x64]);
        assert_eq!(write_dynamic_integer(136), vec![0x81, 0x08]);
        assert_eq!(write_dynamic_integer(320), vec![0x82, 0x40]);
        assert_eq!(write_dynamic_integer(782), vec![0x86, 0x0E]);
        assert_eq!(write_dynamic_integer(783), vec![0x86, 0x0F]);
        assert_eq!(write_dynamic_integer(10005), vec![0xCE, 0x15]);
        assert_eq!(write_dynamic_integer(10220), vec![0xCF, 0x6C]);
        assert_eq!(write_dynamic_integer(11110), vec![0xD6, 0x66]);
        assert_eq!(write_dynamic_integer(11410), vec![0xD9, 0x12]);
        assert_eq!(write_dynamic_integer(12110), vec![0xDE, 0x4E]);
        assert_eq!(write_dynamic_integer(12120), vec![0xDE, 0x58]);
        assert_eq!(write_dynamic_integer(22010), vec![0x81, 0xAB, 0x7A]);
        assert_eq!(write_dynamic_integer(22410), vec![0x81, 0xAF, 0x0A]);
        assert_eq!(
            write_dynamic_integer(40839216),
            vec![0x93, 0xBC, 0xD0, 0x30]
        );
        assert_eq!(
            write_dynamic_integer(1378382453),
            vec![0x85, 0x91, 0xA1, 0xE4, 0x75]
        );
        assert_eq!(
            write_dynamic_integer(-1),
            vec![0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
        ); // =/= 4294967295
        assert_eq!(
            write_dynamic_integer(-75),
            vec![0x8F, 0xFF, 0xFF, 0xFF, 0x35]
        );
        assert_eq!(
            write_dynamic_integer(-95),
            vec![0x8F, 0xFF, 0xFF, 0xFF, 0x21]
        ); // =/= 4294967201
        assert_eq!(
            write_dynamic_integer(-1031863457),
            vec![0x8C, 0x93, 0xFC, 0x86, 0x5F]
        ); // =/= 3263103839
        assert_eq!(
            write_dynamic_integer(-2147483640),
            vec![0x88, 0x80, 0x80, 0x80, 0x08]
        );
    }

    // Compare edge cases & errors
    #[test]
    fn core_edge_cases() {
        // 0x80 =/= 128, it's a continue byte
        assert_eq!(
            read_dynamic_integer(vec![0x88, 0x80, 0x80, 0x80, 0x08]),
            Some(-2147483640)
        );
        assert_eq!(
            write_dynamic_integer(-2147483640),
            vec![0x88, 0x80, 0x80, 0x80, 0x08]
        );

        assert_eq!(read_dynamic_integer(vec![0x81, 0x00]), Some(128));
        assert_eq!(write_dynamic_integer(128), vec![0x81, 0x00]);
    }

    #[binrw]
    #[br(big)]
    struct TestStructure(DynamicInteger);

    #[test]
    fn read_success() {
        let mut reader = Cursor::new(b"\x81\xAF\x0A");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, 22410);

        let mut reader = Cursor::new(b"\x81\x00");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, 128);

        let mut reader = Cursor::new(b"\x8F\xFF\xFF\xFF\x7F");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, -1);
    }

    #[test]
    fn write_success() {
        let data = TestStructure(DynamicInteger(22410));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x81\xAF\x0A");

        let data = TestStructure(DynamicInteger(128));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x81\x00");

        let data = TestStructure(DynamicInteger(-1));
        let mut writer = Cursor::new(Vec::<u8>::new());
        writer.write_le(&data).unwrap();
        assert_eq!(writer.into_inner(), b"\x8F\xFF\xFF\xFF\x7F");
    }

    #[test]
    #[should_panic]
    fn struct_edge_case_5_null() {
        let mut reader = Cursor::new(b"\x80\x80\x80\x80\x80");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, 128);
    }

    #[test]
    #[should_panic]
    fn struct_edge_case_1_null() {
        let mut reader = Cursor::new(b"\x80");
        let data = TestStructure::read(&mut reader).unwrap();
        assert_eq!(*data.0, 128);
    }
}
