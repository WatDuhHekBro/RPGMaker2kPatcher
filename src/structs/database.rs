use crate::{
    structs::{
        patch::{Dialogue, Text},
        Patch,
    },
    types::{DynamicInteger, DynamicIntegerArray, PascalString, U8Array},
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

// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big, magic = b"\x0ALcfDataBase")]
// Vec<LcfDataBaseHeader> (null-terminated)
pub struct LcfDataBase(pub Vec<LcfDataBaseHeader>);

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
