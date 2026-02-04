use crate::{
    structs::{
        patch::{Dialogue, Text},
        LcfCommonCommand, Patch,
    },
    types::{
        double_byte_counted::DoubleByteCounted, ByteCounted, DynamicInteger, DynamicIntegerArray,
        PascalString, PreallocatedList, U8Array,
    },
    util::{constants::*, generate_toml_map, generate_toml_patch},
};
use binrw::{
    binrw,
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, BinWriterExt, Endian,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor};

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.
// -----
// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBase(pub Vec<LcfDataBaseHeader>); // null-terminated

impl BinRead for LcfDataBase {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let magic = PascalString::read_be(reader).expect(ERROR_BINRW_READ);

        if *magic != "LcfDataBase" {
            panic!("LcfDataBase read does not start with the \"LcfDataBase\" header!");
        }

        let mut headers = Vec::new();

        loop {
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

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    //#[brw(magic = 11u8)]
    //Characters(LcfDataBaseCharacters),
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
    #[brw(magic = 25u8)]
    GlobalEvents(ByteCounted<PreallocatedList<LcfDataBaseGlobalEvent>>), // Another double byte count
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

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBaseCharacters {}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBaseGlobalEvent {
    pub id: DynamicInteger,
    pub page: LcfDataBaseGlobalEventHeader,
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseGlobalEventHeader {
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 21u8)]
    Commands(DoubleByteCounted<LcfCommonCommand>),
    Generic(LcfDataBaseGlobalEventHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBaseGlobalEventHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}
