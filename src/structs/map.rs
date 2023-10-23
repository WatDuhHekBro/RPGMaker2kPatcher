use crate::types::{events_wrapper::EventsWrapper, DynamicInteger, PascalString, U8Array};
use binrw::binrw;

// Make sure to place this #[derive(Debug)] below #[binrw], or it'll throw errors for temporary fields.

#[binrw]
#[derive(Debug)]
#[brw(big, magic = b"\x0ALcfMapUnit")]
pub struct LcfMapUnit {
    #[br(count = 14)]
    headers: Vec<LcfMapUnitHeader>,
}

#[binrw]
#[derive(Debug)]
enum LcfMapUnitHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    None,
    #[brw(magic = 0x20u8)]
    Panorama(LcfMapUnitHeaderPanorama),
    #[brw(magic = 0x51u8)]
    Events(EventsWrapper),
    Generic(LcfMapUnitHeaderGeneric),
}

#[binrw]
#[derive(Debug)]
struct LcfMapUnitHeaderGeneric {
    id: DynamicInteger,
    //#[bw(calc(DynamicInteger(value.len().try_into().unwrap())))]
    //size: DynamicInteger,
    //#[br(count = *size)]
    //value: Vec<u8>,
    value: U8Array,
}

#[binrw]
#[derive(Debug)]
struct LcfMapUnitHeaderPanorama(PascalString);

#[binrw]
#[derive(Debug)]
struct LcfMapUnitEvents {
    byte_count: DynamicInteger,
    event_count: DynamicInteger,
    // Problem: Even with this method, you don't know what the resulting size of the new DynamicInteger will be.
    // JS code merges separated streams
    // Maybe another custom type which uses a wrapper around a type? Main --> Wrapper w/ byte count --> Generic Subtype
    #[bw(args(byte_count.size(), event_count.size()))]
    test: Test,
}

#[binrw]
#[derive(Debug)]
#[bw(import(asdf: u8, asdf2: u8))]
struct Test {
    #[bw(calc(asdf + asdf2))]
    yeet: u8,
    zxcv: u8,
}

#[binrw]
#[derive(Debug)]
pub struct LcfMapUnitEvent {
    //
}
