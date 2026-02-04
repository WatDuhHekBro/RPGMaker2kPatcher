use crate::{
    structs::LcfCommonCommandList,
    types::{
        double_byte_counted::DoubleByteCounted, ByteCounted, DynamicInteger, FileTerminatedList,
        NullTerminatedList, PascalString, PreallocatedList, U8Array,
    },
};
use binrw::binrw;
use serde::{Deserialize, Serialize};

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.
// -----
// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big, magic = b"\x0BLcfDataBase")]
pub struct LcfDataBase(pub FileTerminatedList<LcfDataBaseHeader>);

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
    GlobalEvents(ByteCounted<PreallocatedList<LcfDataBaseGlobalEvent>>),
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
    pub headers: NullTerminatedList<LcfDataBaseGlobalEventHeader>,
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseGlobalEventHeader {
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 21u8)]
    Commands(DoubleByteCounted<LcfCommonCommandList>),
    Generic(LcfDataBaseGlobalEventHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBaseGlobalEventHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}
