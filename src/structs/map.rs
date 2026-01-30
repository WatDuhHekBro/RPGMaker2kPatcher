use crate::{
    types::{DynamicInteger, DynamicIntegerArray, PascalString, U8Array},
    util::{generate_toml_patch, generate_toml_representation},
    wrappers::{
        MapCommandsWrapper, MapEventHeadersWrapper, MapEventsWrapper, MapMainWrapper,
        MapPageHeadersWrapper, MapPagesWrapper,
    },
};
use binrw::binrw;
use serde::{Deserialize, Serialize};

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big, magic = b"\x0ALcfMapUnit")]
pub struct LcfMapUnit {
    pub headers: MapMainWrapper, // Wrapper: Vec<LcfMapUnitHeader> (null-terminated)
}

impl LcfMapUnit {
    pub fn get_events(&self) -> Option<&MapEventsWrapper> {
        self.headers.get_events()
    }

    pub fn get_event(&self, id: i32) -> Option<&MapEventHeadersWrapper> {
        self.get_events().and_then(|events| events.get_event(id))
    }

    pub fn generate_toml_representation(&self) -> String {
        generate_toml_representation(&self)
    }

    pub fn generate_toml_patch(&self) -> String {
        generate_toml_patch(&self)
    }
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfMapUnitHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big)]
pub struct LcfMapUnitEvent {
    pub id: DynamicInteger,
    pub headers: MapEventHeadersWrapper, // Vec<LcfMapUnitEventHeader> (null-terminated)
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfMapUnitEventHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 5u8)]
    Pages(MapPagesWrapper), // Wrapper: Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitPage>
    Generic(LcfMapUnitEventHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfMapUnitEventHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big)]
pub struct LcfMapUnitPage {
    pub id: DynamicInteger,
    pub headers: MapPageHeadersWrapper, // Vec<LcfMapUnitPageHeader> (null-terminated)
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfMapUnitPageHeader {
    // Stop if the next byte read is 0x00, that indicates the end.
    #[brw(magic = 0u8)]
    End,
    #[brw(magic = 21u8)]
    Name(PascalString),
    #[brw(magic = 51u8)]
    // 0x33 contains redundant byte count, immediately followed by 0x34 which contains the commands
    Commands(MapCommandsWrapper),
    // Wrapper: Byte Count Length (DynamicInteger) (DISCARD), Byte Count (DynamicInteger) (DISCARD), 0x34 (52)
    // Byte Count (DynamicInteger), # of Pages Count (DynamicInteger), Vec<LcfMapUnitCommand> (null-terminated by a 4-set of zeroes)
    Generic(LcfMapUnitPageHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfMapUnitPageHeaderGeneric {
    pub id: DynamicInteger,
    pub value: U8Array,
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big)]
pub struct LcfMapUnitCommand {
    pub event: DynamicInteger,           // 1st
    pub indent: DynamicInteger,          // 2nd
    pub text: PascalString,              // 3rd
    pub parameters: DynamicIntegerArray, // 4th
}

impl LcfMapUnitCommand {
    pub fn is_terminating(&self) -> bool {
        self.event.0 == 0
            && self.indent.0 == 0
            && self.text.is_empty()
            && self.parameters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command_should_terminate() {
        let command = LcfMapUnitCommand {
            event: DynamicInteger(0),
            indent: DynamicInteger(0),
            text: PascalString::from(""),
            parameters: DynamicIntegerArray(vec![]),
        };
        assert_eq!(command.is_terminating(), true);
    }
}
