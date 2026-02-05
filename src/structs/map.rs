use std::collections::HashMap;

use crate::{
    structs::{LcfCommandList, ListEntry, ListEntryHeaderGeneric, Patch},
    types::{
        double_byte_counted::DoubleByteCounted, ByteCounted, NullTerminatedList, PascalString,
        PreallocatedList,
    },
    util::{generate_toml_map, generate_toml_patch, patch_operations},
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
#[brw(big, magic = b"\x0ALcfMapUnit")]
pub struct LcfMapUnit(pub NullTerminatedList<LcfMapUnitHeader>);

impl LcfMapUnit {
    pub fn get_events(&self) -> Option<&Vec<LcfMapUnitEvent>> {
        for header in &**self {
            if let LcfMapUnitHeader::Events(events) = header {
                return Some(events);
            }
        }

        None
    }

    pub fn get_events_mut(&mut self) -> Option<&mut Vec<LcfMapUnitEvent>> {
        for header in &mut **self {
            if let LcfMapUnitHeader::Events(events) = header {
                return Some(events);
            }
        }

        None
    }

    /*pub fn get_event(&self, id: i32) -> Option<&LcfMapUnitEvent> {
        self.get_events().and_then(|events| {
            for event in events {
                if event.id == id {
                    return Some(event);
                }
            }

            None
        })
    }*/

    pub fn get_event_mut(&mut self, id: i32) -> Option<&mut LcfMapUnitEvent> {
        self.get_events_mut().and_then(|events| {
            for event in events {
                if event.id == id {
                    return Some(event);
                }
            }

            None
        })
    }

    pub fn generate_toml_map(&self) -> String {
        generate_toml_map(&self)
    }

    pub fn generate_toml_patch(
        &self,
        character_names: &HashMap<i32, String>,
        map_name: Option<&String>,
    ) -> String {
        let patch = Patch::generate_from_map(&self, character_names, map_name);
        generate_toml_patch(&patch)
    }

    pub fn apply_patch(&mut self, patch: &Patch) {
        patch_operations::apply_patch_map(self, patch);
    }
}

impl std::ops::Deref for LcfMapUnit {
    type Target = Vec<LcfMapUnitHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LcfMapUnit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfMapUnitHeader {
    #[brw(magic = 32u8)]
    Panorama(PascalString),
    #[brw(magic = 81u8)]
    Events(ByteCounted<PreallocatedList<LcfMapUnitEvent>>),
    Generic(ListEntryHeaderGeneric),
}

pub type LcfMapUnitEvent = ListEntry<LcfMapUnitEventHeader>;

// For some reason "LcfMapUnitEvent" doesn't expand here.
impl ListEntry<LcfMapUnitEventHeader> {
    pub fn get_pages(&self) -> Option<&Vec<LcfMapUnitPage>> {
        for entry in &self.headers.0 {
            if let LcfMapUnitEventHeader::Pages(pages) = entry {
                return Some(pages);
            }
        }

        None
    }

    pub fn get_pages_mut(&mut self) -> Option<&mut Vec<LcfMapUnitPage>> {
        for entry in &mut self.headers.0 {
            if let LcfMapUnitEventHeader::Pages(pages) = entry {
                return Some(pages);
            }
        }

        None
    }

    /*pub fn get_page(&self, id: i32) -> Option<&LcfMapUnitPage> {
        self.get_pages().and_then(|pages| {
            for page in pages {
                if page.id == id {
                    return Some(page);
                }
            }

            None
        })
    }*/

    pub fn get_page_mut(&mut self, id: i32) -> Option<&mut LcfMapUnitPage> {
        self.get_pages_mut().and_then(|pages| {
            for page in pages {
                if page.id == id {
                    return Some(page);
                }
            }

            None
        })
    }
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfMapUnitEventHeader {
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 5u8)]
    Pages(ByteCounted<PreallocatedList<LcfMapUnitPage>>),
    Generic(ListEntryHeaderGeneric),
}

pub type LcfMapUnitPage = ListEntry<LcfMapUnitPageHeader>;

impl LcfMapUnitPage {
    pub fn get_commands(&self) -> Option<&LcfCommandList> {
        for entry in &self.headers.0 {
            if let LcfMapUnitPageHeader::Commands(commands) = entry {
                return Some(commands);
            }
        }

        None
    }

    pub fn get_commands_mut(&mut self) -> Option<&mut LcfCommandList> {
        for entry in &mut self.headers.0 {
            if let LcfMapUnitPageHeader::Commands(commands) = entry {
                return Some(commands);
            }
        }

        None
    }
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfMapUnitPageHeader {
    #[brw(magic = 21u8)]
    Name(PascalString),
    // 0x33 contains redundant byte count, immediately followed by 0x34 which contains the commands
    #[brw(magic = 51u8)]
    Commands(DoubleByteCounted<LcfCommandList>),
    Generic(ListEntryHeaderGeneric),
}
