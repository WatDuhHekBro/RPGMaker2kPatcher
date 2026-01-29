use crate::{
    types::{DynamicInteger, PascalString, U8Array, DynamicIntegerArray},
    wrappers::{MapEventHeadersWrapper, MapEventsWrapper, MapMainWrapper, MapPagesWrapper, MapPageHeadersWrapper, MapCommandsWrapper},
};
use binrw::binrw;

// Make sure to place this #[derive(Debug)] below #[binrw], or it'll throw errors for temporary fields.

#[binrw]
#[derive(Debug)]
#[brw(big, magic = b"\x0ALcfMapUnit")]
pub struct LcfMapUnit {
    pub headers: MapMainWrapper, // Wrapper: Vec<LcfMapUnitHeader> (null-terminated)
}

#[binrw]
#[derive(Debug)]
pub enum LcfMapUnitHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    #[brw(magic = 0x20u8)]
    Panorama(PascalString),
    #[brw(magic = 0x51u8)]
    Events(MapEventsWrapper), // Wrapper: Byte Count (DynamicInteger), # of Events Count (DynamicInteger), Vec<LcfMapUnitEvent>
    Generic(LcfMapUnitHeaderGeneric),
}

#[binrw]
#[derive(Debug)]
pub struct LcfMapUnitHeaderGeneric {
    id: DynamicInteger,
    value: U8Array,
}

#[binrw]
#[derive(Debug)]
#[brw(big)]
pub struct LcfMapUnitEvent {
    id: DynamicInteger,
    headers: MapEventHeadersWrapper, // Vec<LcfMapUnitEventHeader> (null-terminated)
}

#[binrw]
#[derive(Debug)]
pub enum LcfMapUnitEventHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 5u8)]
    Events(MapPagesWrapper), // Wrapper: Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitPage>
    Generic(LcfMapUnitEventHeaderGeneric),
}

#[binrw]
#[derive(Debug)]
pub struct LcfMapUnitEventHeaderGeneric {
    id: DynamicInteger,
    value: U8Array,
}

#[binrw]
#[derive(Debug)]
#[brw(big)]
pub struct LcfMapUnitPage {
    id: DynamicInteger,
    headers: MapPageHeadersWrapper, // Vec<LcfMapUnitPageHeader> (null-terminated)
}

#[binrw]
#[derive(Debug)]
pub enum LcfMapUnitPageHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    #[brw(magic = 21u8)]
    Name(PascalString),
    #[brw(magic = 51u8)] // 0x33 contains redundant byte count, immediately followed by 0x34 which contains the commands
    // Wrapper: Byte Count Length (DynamicInteger) (DISCARD), Byte Count (DynamicInteger) (DISCARD), 0x34 (52)
    // Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitCommand> (null-terminated by a 4-set of zeroes)
    Commands(MapCommandsWrapper),
    Generic(LcfMapUnitPageHeaderGeneric),
}

#[binrw]
#[derive(Debug)]
pub struct LcfMapUnitPageHeaderGeneric {
    id: DynamicInteger,
    value: U8Array,
}

#[binrw]
#[derive(Debug)]
#[brw(big)]
pub struct LcfMapUnitCommand {
    pub event: DynamicInteger, // 1st
    pub indent: DynamicInteger, // 2nd
    pub text: PascalString, // 3rd
    pub parameters: DynamicIntegerArray // 4th
}

impl LcfMapUnitCommand {
    pub fn is_terminating(&self) -> bool {
        self.event.size() == 0 &&
        self.indent.size() == 0 &&
        self.text.is_empty() &&
        self.parameters.is_empty()
    }
}
