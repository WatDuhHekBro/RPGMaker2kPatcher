use crate::structs::{map::*, LcfMapUnit};
use serde::{Deserialize, Serialize};

// Dialogue-related commands
const COMMAND_DIALOGUE_START: i32 = 10110;
const COMMAND_DIALOGUE_CONTINUE: i32 = 20110;
const COMMAND_CHANGE_FACE_GRAPHIC: i32 = 10130;
// Other commands
const COMMAND_MULTIPLE_CHOICE_PROMPT: i32 = 10140;
const COMMAND_MULTIPLE_CHOICE_SELECTION: i32 = 20140;
const COMMAND_SAVE_POINT_NAME: i32 = 10610;

#[derive(Debug, Deserialize, Serialize)]
pub struct Patch {
    pub dialogue: Vec<Dialogue>,
    pub replace: Vec<Replace>,
}

impl Patch {
    pub fn generate_from_map(map: &LcfMapUnit) -> Patch {
        let mut dialogue: Vec<Dialogue> = vec![];
        let mut replace: Vec<Replace> = vec![];

        // Loop through all commands
        if let Some(events) = map.get_events() {
            for event in &events.0 {
                if let Some(pages) = event.headers.get_pages() {
                    for page in &pages.0 {
                        if let Some(commands) = page.headers.get_commands() {
                            let mut command_index = 0;
                            let mut start_index = 0;
                            let mut last_command_code = -1;
                            let mut current_dialogue_text = String::new();

                            for command in &commands.0 {
                                // 10110 as A
                                // 20110 as B
                                // A o --> [A]
                                // A A --> [A] [A]
                                // A B --> [A,B]
                                // A B B --> [A,B,B]
                                // A B A --> [A,B] [A]
                                let current_command_code = command.code.0;

                                let was_single_line_dialogue = last_command_code
                                    == COMMAND_DIALOGUE_START
                                    && current_command_code != COMMAND_DIALOGUE_CONTINUE;

                                let was_dialogue_terminated = last_command_code
                                    == COMMAND_DIALOGUE_CONTINUE
                                    && current_command_code != COMMAND_DIALOGUE_CONTINUE;

                                let is_other_text = current_command_code
                                    == COMMAND_MULTIPLE_CHOICE_PROMPT
                                    || current_command_code == COMMAND_MULTIPLE_CHOICE_SELECTION
                                    || current_command_code == COMMAND_SAVE_POINT_NAME;

                                if was_single_line_dialogue || was_dialogue_terminated {
                                    // NOTE: This ending newline is to make the dialogue lines pretty for manual editing.
                                    // Be sure to keep this in mind when reading the patch files!
                                    //current_dialogue_text.push_str("\n");

                                    dialogue.push(Dialogue {
                                        event: event.id.0,
                                        page: page.id.0,
                                        command: start_index,
                                        original: current_dialogue_text.clone(),
                                        patched: current_dialogue_text.clone(),
                                    });

                                    current_dialogue_text = String::new();
                                }

                                if current_command_code == COMMAND_DIALOGUE_START {
                                    start_index = command_index;
                                    //current_dialogue_text = command.text.0.clone();
                                    current_dialogue_text.push_str(command.text.0.as_str());
                                } else if current_command_code == COMMAND_DIALOGUE_CONTINUE {
                                    current_dialogue_text.push_str("\n");
                                    current_dialogue_text.push_str(command.text.0.as_str());
                                } else if is_other_text {
                                    replace.push(Replace {
                                        event: event.id.0,
                                        page: page.id.0,
                                        command: command_index,
                                        original: command.text.0.clone(),
                                        patched: command.text.0.clone(),
                                    });
                                }

                                last_command_code = current_command_code;
                                command_index += 1;
                            }
                        }
                    }
                }
            }
        }

        // TODO: Testing quotes
        /*replace.push(Replace {
            event: 1,
            page: 2,
            command: 3,
            original: String::from("test"),
            patched: String::from("testout"),
        });*/

        Patch { dialogue, replace }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dialogue {
    pub event: i32,
    pub page: i32,
    pub command: i32,
    //pub is_portrait: bool,
    pub original: String,
    pub patched: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Replace {
    pub event: i32,
    pub page: i32,
    pub command: i32,
    pub original: String,
    pub patched: String,
}

#[derive(Debug, Deserialize)]
pub struct LegacyPatch {
    dialogue: Vec<LegacyPatchDialogue>,
    other: Vec<LegacyPatchOther>,
}

#[derive(Debug, Deserialize)]
struct LegacyPatchDialogue {
    // A 4-tuple integer array consisting of: [event #, page #, command start, command length]
    // Note that the command length is for the original lines, not the patched lines.
    path: [u16; 4],
    original: String,
    lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyPatchOther {}

impl LegacyPatch {
    pub fn convert_to_toml_patch(self) /*-> Patch*/
    {
        //Patch { dialogue: () }
    }
}
