// Note: 0x33 contains redundant byte count, immediately followed by 0x34 which contains the commands
// -----
// Wrapper: Byte Count Length (DynamicInteger) (DISCARD), Byte Count (DynamicInteger) (DISCARD), 0x34 (52)
// Byte Count (DynamicInteger), Vec<LcfMapUnitCommand> (null-terminated by a 4-set of zeroes)

use crate::{
    structs::map::LcfMapUnitCommand,
    types::{DynamicInteger, DynamicIntegerArray, PascalString},
    util::constants::*,
};
use binrw::{
    io::{Cursor, Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MapCommandsWrapper(pub Vec<LcfMapUnitCommand>);

impl BinRead for MapCommandsWrapper {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let _ = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ); // 0x33 byte count length (DISCARD)
        let _ = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ); // 0x33 byte count (DISCARD)
        let _ = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ); // 0x34 identifier (DISCARD)

        // Unless you can read the amount of bytes of a struct, just operate on a separate pool of bytes
        let mut bytes = Vec::<u8>::new();
        let byte_count = DynamicInteger::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);

        for _ in 0..*byte_count {
            let byte = <u8>::read_options(reader, endian, ()).expect(ERROR_BINRW_READ);
            bytes.push(byte);
        }

        // This entire section will now operate on this section of bytes
        let mut reader = Cursor::new(&bytes);
        let mut commands = Vec::<LcfMapUnitCommand>::new();

        loop {
            // No idea why this doesn't work, so just read each field manually.
            //let command = LcfMapUnitCommand::read(&mut reader).unwrap();
            let code =
                DynamicInteger::read_options(&mut reader, endian, ()).expect(ERROR_BINRW_READ);
            let indent =
                DynamicInteger::read_options(&mut reader, endian, ()).expect(ERROR_BINRW_READ);
            let text = PascalString::read_options(&mut reader, endian, ()).expect(ERROR_BINRW_READ);
            let parameters =
                DynamicIntegerArray::read_options(&mut reader, endian, ()).expect(ERROR_BINRW_READ);

            let command = LcfMapUnitCommand {
                code,
                indent,
                text,
                parameters,
            };

            if command.is_terminating() {
                break;
            } else {
                commands.push(command);
            }
        }

        Ok(MapCommandsWrapper(commands))
    }
}

impl BinWrite for MapCommandsWrapper {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let mut subsection_writer = Cursor::new(Vec::<u8>::new());
        // Write the initial command bytes
        subsection_writer.write_be(&self.0).unwrap();
        // Add a null-terminating command set
        0u32.write_options(&mut subsection_writer, endian, args)?;

        // Then count the bytes for the main slice
        let bytes = subsection_writer.into_inner();
        let bytes_count = DynamicInteger(bytes.len().try_into().unwrap());
        //println!("{:02X?}", bytes);

        // First write 0x33 as the redundant byte count ID
        // NOTE: Actually you DON'T, it's already there from the binrw magic number.
        //0x33u8.write_options(writer, endian, args)?;
        // Then write the size of the bytes count (I know)
        bytes_count.size().write_options(writer, endian, args)?;
        // Then the byte count (I know)
        bytes_count.write_options(writer, endian, args)?;
        // Then 0x34
        0x34u8.write_options(writer, endian, args)?;
        // Then the byte count again (Yep... I know)
        bytes_count.write_options(writer, endian, args)?;
        // Then finally write the sub-section into the main section of bytes
        bytes.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl std::ops::Deref for MapCommandsWrapper {
    type Target = Vec<LcfMapUnitCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapCommandsWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
