use crate::{
    structs::{LcfCommandList, ListEntry, ListEntryHeaderGeneric, Patch},
    types::{
        double_byte_counted::DoubleByteCounted, ByteCounted, DynamicInteger, FileTerminatedList,
        NullTerminatedList, PascalString, PreallocatedList,
    },
    util::{generate_toml_database, generate_toml_patch, patch_operations},
};
use binrw::binrw;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.
// -----
// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big, magic = b"\x0BLcfDataBase")]
pub struct LcfDataBase(pub FileTerminatedList<LcfDataBaseHeader>);

impl LcfDataBase {
    pub fn get_events(&self) -> Option<&Vec<LcfDataBaseGlobalEvent>> {
        for header in &**self {
            if let LcfDataBaseHeader::GlobalEvents(events) = header {
                return Some(events);
            }
        }

        None
    }

    pub fn get_events_mut(&mut self) -> Option<&mut Vec<LcfDataBaseGlobalEvent>> {
        for header in &mut **self {
            if let LcfDataBaseHeader::GlobalEvents(events) = header {
                return Some(events);
            }
        }

        None
    }

    /*pub fn get_event(&self, id: i32) -> Option<&LcfDataBaseGlobalEvent> {
        self.get_events().and_then(|events| {
            for event in events {
                if event.id == id {
                    return Some(event);
                }
            }

            None
        })
    }*/

    pub fn get_event_mut(&mut self, id: i32) -> Option<&mut LcfDataBaseGlobalEvent> {
        self.get_events_mut().and_then(|events| {
            for event in events {
                if event.id == id {
                    return Some(event);
                }
            }

            None
        })
    }

    pub fn extract_character_names(&self) -> HashMap<i32, String> {
        let mut character_names = HashMap::new();

        for header in &**self {
            if let LcfDataBaseHeader::Characters(characters) = header {
                let characters = &***characters;

                for character in characters {
                    for character_header in &*character.headers {
                        if let LcfDataBaseCharacterHeader::Name1(name) = character_header {
                            character_names.insert(*character.id, name.0.clone());

                            // Name header found
                            break;
                        }
                    }
                }

                // No need to continue the loop to the other top-level headers once found
                break;
            }
        }

        character_names
    }

    pub fn generate_toml_database(&self) -> String {
        generate_toml_database(&self)
    }

    pub fn generate_toml_patch(&self, game_title: &String) -> String {
        let patch = Patch::generate_from_database(&self, game_title);
        generate_toml_patch(&patch)
    }

    pub fn apply_patch(&mut self, patch: &Patch) {
        patch_operations::apply_patch_database(self, patch);
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
    #[brw(magic = 11u8)]
    Characters(ByteCounted<PreallocatedList<LcfDataBaseCharacter>>),
    #[brw(magic = 12u8)]
    Skills(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 13u8)]
    Items(ByteCounted<PreallocatedList<LcfDataBaseDoubleText>>),
    #[brw(magic = 14u8)]
    Enemies(ByteCounted<PreallocatedList<LcfDataBaseDoubleText>>),
    #[brw(magic = 15u8)]
    EnemyGroups(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 16u8)]
    Terrain(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 17u8)]
    Attributes(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 18u8)]
    Conditions(ByteCounted<PreallocatedList<LcfDataBaseConditions>>),
    #[brw(magic = 19u8)]
    BattleAnimations(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 20u8)]
    Chipsets(ByteCounted<PreallocatedList<LcfDataBaseDoubleText>>),
    #[brw(magic = 21u8)]
    Vocabulary(ByteCounted<NullTerminatedList<LcfDataBaseVocab>>),
    #[brw(magic = 22u8)]
    System(ByteCounted<NullTerminatedList<LcfDataBaseVocab>>),
    #[brw(magic = 23u8)]
    Switches(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 24u8)]
    Variables(ByteCounted<PreallocatedList<LcfDataBaseSingleText>>),
    #[brw(magic = 25u8)]
    GlobalEvents(ByteCounted<PreallocatedList<LcfDataBaseGlobalEvent>>),
    Generic(ListEntryHeaderGeneric),
}

pub type LcfDataBaseSingleText = ListEntry<LcfDataBaseSingleTextHeader>;

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseSingleTextHeader {
    #[brw(magic = 1u8)]
    Text(PascalString),
    Generic(ListEntryHeaderGeneric),
}

pub type LcfDataBaseDoubleText = ListEntry<LcfDataBaseDoubleTextHeader>;

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseDoubleTextHeader {
    #[brw(magic = 1u8)]
    TextA(PascalString),
    #[brw(magic = 2u8)]
    TextB(PascalString),
    Generic(ListEntryHeaderGeneric),
}

pub type LcfDataBaseCharacter = ListEntry<LcfDataBaseCharacterHeader>;

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseCharacterHeader {
    #[brw(magic = 1u8)]
    Name1(PascalString),
    #[brw(magic = 2u8)]
    Name2(PascalString),
    #[brw(magic = 3u8)]
    Name3(PascalString),
    #[brw(magic = 15u8)]
    Face(PascalString),
    #[brw(magic = 67u8)]
    Type(PascalString),
    Generic(ListEntryHeaderGeneric),
}

pub type LcfDataBaseConditions = ListEntry<LcfDataBaseConditionsHeader>;

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseConditionsHeader {
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 51u8)]
    Start1(PascalString),
    #[brw(magic = 52u8)]
    Start2(PascalString),
    #[brw(magic = 55u8)]
    Finish(PascalString),
    Generic(ListEntryHeaderGeneric),
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub struct LcfDataBaseVocab {
    pub id: DynamicInteger,
    pub text: PascalString,
}

pub type LcfDataBaseGlobalEvent = ListEntry<LcfDataBaseGlobalEventHeader>;

impl LcfDataBaseGlobalEvent {
    pub fn get_commands(&self) -> Option<&LcfCommandList> {
        for entry in &self.headers.0 {
            if let LcfDataBaseGlobalEventHeader::Commands(commands) = entry {
                return Some(commands);
            }
        }

        None
    }

    pub fn get_commands_mut(&mut self) -> Option<&mut LcfCommandList> {
        for entry in &mut self.headers.0 {
            if let LcfDataBaseGlobalEventHeader::Commands(commands) = entry {
                return Some(commands);
            }
        }

        None
    }
}

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfDataBaseGlobalEventHeader {
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 21u8)]
    Commands(DoubleByteCounted<LcfCommandList>),
    Generic(ListEntryHeaderGeneric),
}
