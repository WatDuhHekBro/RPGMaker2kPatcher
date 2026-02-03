use crate::{
    structs::{
        patch::{Dialogue, Text},
        Patch,
    },
    types::{DynamicInteger, DynamicIntegerArray, PascalString, U8Array},
    util::{constants::*, generate_toml_map, generate_toml_patch},
    wrappers::{
        MapCommandsWrapper, MapEventHeadersWrapper, MapEventsWrapper, MapMainWrapper,
        MapPageHeadersWrapper, MapPagesWrapper,
    },
};
use binrw::binrw;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.

// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

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
    pub fn get_events_mut(&mut self) -> Option<&mut MapEventsWrapper> {
        self.headers.get_events_mut()
    }

    pub fn get_event(&self, id: i32) -> Option<&MapEventHeadersWrapper> {
        self.get_events().and_then(|events| events.get_event(id))
    }

    pub fn get_event_mut(&mut self, id: i32) -> Option<&mut MapEventHeadersWrapper> {
        self.get_events_mut()
            .and_then(|events| events.get_event_mut(id))
    }

    pub fn generate_toml_map(&self) -> String {
        generate_toml_map(&self)
    }

    pub fn generate_toml_patch(&self, map_name: Option<&String>) -> String {
        let patch = Patch::generate_from_map(&self, map_name);
        generate_toml_patch(&patch)
    }

    pub fn apply_patch(&mut self, patch: &Patch) {
        // Since Array.splice is a dynamic function, you need to adjust for things that'll change the index.
        // The offset tracked will be different for every event-page pair.
        // Targeted command index also must be in order, if you want this simplified offset method to work (and not have to use a HashMap of references).
        // -----
        // Or just forget the above. Use the HashMap method to keep track of where everything is.
        // This method provides resilience against out-of-order TOML. The patch order isn't dependent on user-edited TOML.
        // -----
        // HashMap<(event, page), Vec<(original_index), shifted_index>>
        // Make sure the offsets is NOT unsigned, you need those negative numbers!
        let mut offsets_table: HashMap<(i32, i32), Vec<isize>> = HashMap::new();

        if let Some(dialogues) = &patch.dialogue {
            for dialogue in dialogues {
                let Dialogue {
                    event,
                    page,
                    command: command_index,
                    indent: explicitly_defined_indent,
                    original,
                    patched,
                } = dialogue;
                let key = (*event, *page);

                // Create offset entry if it hasn't worked on this key yet
                if !offsets_table.contains_key(&key) {
                    let commands = &self
                        .get_event(*event)
                        .expect("Event should exist!")
                        .get_page(*page)
                        .expect("Page should exist!")
                        .get_commands()
                        .expect("Commands should exist!")
                        .0;
                    let commands_len = commands.len();
                    let mut offsets: Vec<isize> = Vec::with_capacity(commands.len());

                    for _ in 0..commands_len {
                        offsets.push(0);
                    }

                    offsets_table.insert(key, offsets);
                }

                // Then work off the existing offsets table.
                let offsets = offsets_table
                    .get_mut(&key)
                    .expect("Offsets HashMap should exist by this point!");

                let commands = &mut self
                    .get_event_mut(*event)
                    .expect("Event should exist!")
                    .get_page_mut(*page)
                    .expect("Page should exist!")
                    .get_commands_mut()
                    .expect("Commands should exist!");

                // Get explicit indent if available or assume previous indent
                let command_index = *command_index as usize;
                let start_index =
                    ((command_index as isize) + (offsets[command_index])).max(0) as usize;

                let previous_indent = {
                    if let Some(indent) = explicitly_defined_indent {
                        indent.0
                    } else {
                        if start_index > 0 {
                            commands[start_index - 1].indent.0
                        } else {
                            0
                        }
                    }
                };

                // Generate patched commands
                let mut patched_commands: Vec<LcfMapUnitCommand> = Vec::new();
                let patched_lines = patched.split("\n").collect::<Vec<&str>>();
                let patched_lines_count = patched_lines.len();
                let mut is_first_line = true;

                if patched_lines.len() > 4 {
                    println!("ERROR: More than 4 lines of dialogue found!");
                }

                for line in patched_lines {
                    if is_first_line {
                        patched_commands.push(LcfMapUnitCommand {
                            code: DynamicInteger(COMMAND_DIALOGUE_START),
                            indent: DynamicInteger(previous_indent),
                            text: PascalString::from(line),
                            parameters: DynamicIntegerArray(Vec::new()),
                        });

                        is_first_line = false;
                    } else {
                        patched_commands.push(LcfMapUnitCommand {
                            code: DynamicInteger(COMMAND_DIALOGUE_CONTINUE),
                            indent: DynamicInteger(previous_indent),
                            text: PascalString::from(line),
                            parameters: DynamicIntegerArray(Vec::new()),
                        });
                    }
                }

                // Splice
                let original_lines = original.split("\n").collect::<Vec<&str>>();
                let original_lines_count = original_lines.len();
                let stop_index = start_index + original_lines_count;
                let splice_range = start_index..stop_index;
                commands.splice(splice_range, patched_commands);

                // Then update indexes
                // ----- Add -----
                // [0, 1, [2, 3, 4], [5, 6], 7, 8, 9] => index=2, len=3->4
                // [0, 1, [2, 3, 4, 5], [6, 7], 8, 9] => index=5+1, len=2
                // [+0 +0 +0 +0 +0  +1  +1 +1  +1 +1] => start new offset at index=5
                // ----- Remove -----
                // [0, 1, [2, 3, 4], [5, 6], 7, 8, 9] => index=2, len=3->1
                // [0, 1, [2], [3, 4], 5, 6, 7, 8, 9] => index=5-2, len=2
                // [+0 +0 +0  +0 +0  -2 -2  -2 -2 -2]
                // ----- Map0150 -----
                // [0, 1, [2, 3, 4], [5, 6, 7, 8], 9, 10, 11, 12, 13, 14, [15, 16], 17, [18], 19, [20, 21, 22], [23, 24, 25, 26], 27, [28, 29], 30, 31, ...]
                //                  +1
                // [0, 1, [2, 3, 4, 5], [6, 7, 8, 9], 10, 11, 12, 13, 14, 15, [16, 17], 18, [19], 20, [21, 22, 23], [24, 25, 26, 27], 28, [29, 30], 31, ...]
                //                                                                      +1
                // [0, 1, [2, 3, 4, 5], [6, 7, 8, 9], 10, 11, 12, 13, 14, 15, [16, 17, 18], 19, [20], 21, [22, 23, 24], [25, 26, 27, 28], 29, [30, 31], ...]
                //                                                                                                 -1
                // [0, 1, [2, 3, 4, 5], [6, 7, 8, 9], 10, 11, 12, 13, 14, 15, [16, 17, 18], 19, [20], 21, [22, 23], [24, 25, 26, 27], 28, [29, 30], 31, ...]
                // But stop_index=25. You set offsets from index=25. But the original_index=23. So it's 23+2 starting at 25, not 24.
                // Should you use the original_index?
                let length_difference =
                    (patched_lines_count as isize) - (original_lines_count as isize);

                for offsets_index in command_index..offsets.len() {
                    offsets[offsets_index] += length_difference;
                }

                //println!("{offsets:?} <== {length_difference}");
            }
        }

        if let Some(texts) = &patch.text {
            for text in texts {
                let Text {
                    event,
                    page,
                    command: command_index,
                    original: _,
                    patched,
                } = text;
                let key = (*event, *page);

                // Create offset entry if it hasn't worked on this key yet
                if !offsets_table.contains_key(&key) {
                    let commands = &self
                        .get_event(*event)
                        .expect("Event should exist!")
                        .get_page(*page)
                        .expect("Page should exist!")
                        .get_commands()
                        .expect("Commands should exist!")
                        .0;
                    let commands_len = commands.len();
                    let mut offsets: Vec<isize> = Vec::with_capacity(commands.len());

                    for _ in 0..commands_len {
                        offsets.push(0);
                    }

                    offsets_table.insert(key, offsets);
                }

                // Then work off the existing offsets table.
                let offsets = offsets_table
                    .get_mut(&key)
                    .expect("Offsets HashMap should exist by this point!");

                let commands = &mut self
                    .get_event_mut(*event)
                    .expect("Event should exist!")
                    .get_page_mut(*page)
                    .expect("Page should exist!")
                    .get_commands_mut()
                    .expect("Commands should exist!");

                // Get explicit indent if available or assume previous indent
                let command_index = *command_index as usize;
                let start_index =
                    ((command_index as isize) + (offsets[command_index])).max(0) as usize;

                // Replace text
                commands[start_index].text = PascalString::from(patched);
            }
        }
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
    pub code: DynamicInteger, // 1st
    // If I remember correctly, the indent is mostly just for viewing it in an editor (EasyRPG Editor or the official RPGMaker2k)
    pub indent: DynamicInteger,          // 2nd
    pub text: PascalString,              // 3rd
    pub parameters: DynamicIntegerArray, // 4th
}

impl LcfMapUnitCommand {
    pub fn is_terminating(&self) -> bool {
        self.code.0 == 0 && self.indent.0 == 0 && self.text.is_empty() && self.parameters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command_should_terminate() {
        let command = LcfMapUnitCommand {
            code: DynamicInteger(0),
            indent: DynamicInteger(0),
            text: PascalString::from(""),
            parameters: DynamicIntegerArray(vec![]),
        };
        assert_eq!(command.is_terminating(), true);
    }
}
