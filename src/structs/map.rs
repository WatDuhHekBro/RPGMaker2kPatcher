use crate::{
    types::{DynamicInteger, PascalString, U8Array},
    wrappers::{MapEventHeadersWrapper, MapEventsWrapper, MapMainWrapper},
};
use binrw::binrw;

// Make sure to place this #[derive(Debug)] below #[binrw], or it'll throw errors for temporary fields.

#[binrw]
#[derive(Debug)]
#[brw(big, magic = b"\x0ALcfMapUnit")]
pub struct LcfMapUnit {
    headers: MapMainWrapper, // Wrapper: Vec<LcfMapUnitHeader> (null-terminated)
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
    //#[brw(magic = 5u8)]
    //Events(MapPagesWrapper), // Wrapper: Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitPage>
    /*
    {
        strings: [21],
        bytecount: 51,
        commands: 52
    }
    */
    Generic(LcfMapUnitEventHeaderGeneric),
}

#[binrw]
#[derive(Debug)]
pub struct LcfMapUnitEventHeaderGeneric {
    id: DynamicInteger,
    value: U8Array,
}
