use crate::{
    structs::{
        patch::{Dialogue, Text},
        Patch,
    },
    types::{DynamicInteger, DynamicIntegerArray, PascalString, U8Array},
    util::{constants::*, generate_toml_map, generate_toml_patch},
};
use binrw::{
    BinWriterExt, binrw,
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor};

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.

// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

#[derive(Debug, Deserialize, Serialize)]
//#[brw(big, magic = b"\x0ALcfDataBase")]
// Vec<LcfDataBaseHeader> (null-terminated)
pub struct LcfDataBase(pub Vec<LcfDataBaseHeader>);

impl BinRead for LcfDataBase {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let magic = PascalString::read_be(reader).expect(ERROR_BINRW_READ);

        if magic.0 != "LcfDataBase" {
            panic!("Target file attempted to read as an LcfDataBase, but did not contain the magic number!");
        }

        let mut headers = vec![];

        loop {
            let id = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);

            match id.0 {
                0 => {
                    break;
                }
                11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 23 | 24 | 25 => {
                    let mut bytes = Vec::<u8>::new();
                    let byte_count = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);

                    for _ in 0..*byte_count {
                        let byte = u8::read_be(reader).expect(ERROR_BINRW_READ);
                        bytes.push(byte);
                    }

                    // This entire section will now operate on this section of bytes
                    let mut reader = Cursor::new(&bytes);
                    let mut elements = Vec::<LcfMapUnitEvent>::new();

                    let length = DynamicInteger::read_be(&mut reader).expect(ERROR_BINRW_READ);

                    for _ in 0..*length {
                        let element_id =
                            DynamicInteger::read_be(&mut reader).expect(ERROR_BINRW_READ);
                        let event = LcfMapUnitEvent::read(reader).expect(ERROR_BINRW_READ);
                        elements.push(event);
                    }
                }
                _ => {
                    panic!("Unknown ID {id} in LcfDataBase!");
                }
            }

            //

            let header = LcfDataBaseHeader::read_be(reader).expect(ERROR_BINRW_READ);

            if let LcfDataBaseHeader::End = header {
                break;
            } else {
                headers.push(header);
            }
        }

        Ok(LcfDataBase(headers))
    }
}

impl BinWrite for LcfDataBase {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        _: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<()> {
        PascalString::from("LcfDataBase").write_be(writer)?;
        self.0.write_be(writer)?;
        0u8.write_be(writer)?;
        Ok(())
    }
}

impl std::ops::Deref for LcfDataBase {
    type Target = Vec<LcfDataBaseHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LcfDataBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ByteCount<T: BinRead>(pub T);

impl<T: BinRead> BinRead for ByteCount<T>
where
    T: BinRead,
    for<'a> T::Args<'a>: Default,
{
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _: Endian,
        _: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut bytes = Vec::<u8>::new();
        let byte_count = DynamicInteger::read_be(reader).expect(ERROR_BINRW_READ);

        for _ in 0..*byte_count {
            let byte = u8::read_be(reader).expect(ERROR_BINRW_READ);
            bytes.push(byte);
        }

        // This entire section will now operate on this section of bytes
        let mut reader = Cursor::new(&bytes);
        let inner = T::read_be(&mut reader).expect(ERROR_BINRW_READ);

        Ok(ByteCount(inner))
    }
}

impl<T> BinWrite for ByteCount<T> where T: BinRead + BinWrite + BinWriterExt, {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        let mut subsection_writer = Cursor::new(Vec::<u8>::new());
        // Write the rest of the bytes
        &self.0.write_options(writer, endian, ());
        subsection_writer.write_be(&self.0).unwrap();
        let bytes = subsection_writer.into_inner();

        // Then write the sub-section into the main section of bytes
        DynamicInteger(bytes.len().try_into().unwrap()).write_options(writer, endian, args)?;
        bytes.write_options(writer, endian, args)?;

        Ok(())
    }
}

impl<T: BinRead> std::ops::Deref for ByteCount<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BinRead> std::ops::DerefMut for ByteCount<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    // 11u8 = Characters
    // 12u8 = Skills
    // 13u8 = Items
    // 14u8 = Enemies
    // 15u8 = EnemyGroups
    // 16u8 = Terrain
    // 17u8 = Attributes
    // 18u8 = Conditions
    // 19u8 = BattleAnimations
    // 20u8 = Chipsets
    // 21u8 = Vocabulary
    // 22u8 = System
    // 23u8 = Switches
    // 24u8 = Variables
    // 25u8 = CommonEvents
    //#[brw(magic = 0x20u8)]
    //Panorama(PascalString),
    //#[brw(magic = 0x51u8)]
    //Events(MapEventsWrapper), // Wrapper: Byte Count (DynamicInteger), # of Events Count (DynamicInteger), Vec<LcfMapUnitEvent>
    Generic(LcfDataBaseHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfData {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    // 11u8 = Characters
    // 12u8 = Skills
    // 13u8 = Items
    // 14u8 = Enemies
    // 15u8 = EnemyGroups
    // 16u8 = Terrain
    // 17u8 = Attributes
    // 18u8 = Conditions
    // 19u8 = BattleAnimations
    // 20u8 = Chipsets
    // 21u8 = Vocabulary
    // 22u8 = System
    // 23u8 = Switches
    // 24u8 = Variables
    // 25u8 = CommonEvents
    //#[brw(magic = 0x20u8)]
    //Panorama(PascalString),
    //#[brw(magic = 0x51u8)]
    //Events(MapEventsWrapper), // Wrapper: Byte Count (DynamicInteger), # of Events Count (DynamicInteger), Vec<LcfMapUnitEvent>
    Generic(LcfDataBaseHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBaseHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}
