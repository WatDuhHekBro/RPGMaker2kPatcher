use crate::{
    structs::{map::LcfMapUnitCommand, LcfMapUnit},
    types::DynamicInteger,
    util::constants::*,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Patch {
    // These have to be made optional in order for serde to be able to read the TOML file directly
    pub dialogue: Option<Vec<Dialogue>>,
    pub text: Option<Vec<Text>>,
    pub insert_commands: Option<Vec<InsertCommands>>,
}

impl Patch {
    pub fn generate_from_map(map: &LcfMapUnit, map_name: Option<&String>) -> Patch {
        let mut dialogue: Vec<Dialogue> = vec![];
        let mut text: Vec<Text> = vec![];

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
                            let mut last_command_indent = 0;
                            let mut indent_written_into_patch: Option<DynamicInteger> = None;

                            for command in &commands.0 {
                                // 10110 as A
                                // 20110 as B
                                // A o --> [A]
                                // A A --> [A] [A]
                                // A B --> [A,B]
                                // A B B --> [A,B,B]
                                // A B A --> [A,B] [A]
                                let current_command_code = command.code.0;
                                let current_command_indent = command.indent.0;

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
                                    dialogue.push(Dialogue {
                                        event: event.id.0,
                                        page: page.id.0,
                                        command: start_index,
                                        indent: indent_written_into_patch,
                                        original: current_dialogue_text.clone(),
                                        patched: current_dialogue_text.clone(),
                                    });

                                    current_dialogue_text = String::new();
                                    indent_written_into_patch = None;
                                }

                                if current_command_code == COMMAND_DIALOGUE_START {
                                    start_index = command_index;
                                    current_dialogue_text.push_str(command.text.0.as_str());

                                    // Compare the indent for the first line of dialogue
                                    indent_written_into_patch = {
                                        if current_command_indent != last_command_indent {
                                            Some(DynamicInteger(current_command_indent))
                                        } else {
                                            None
                                        }
                                    };

                                    // I don't think dialogue commands have parameters, but it doesn't hurt
                                    // to alert the user if there is any.
                                    if !command.parameters.is_empty() {
                                        println!("WARNING: [map.{map_name:?}.event.{}.page.{}.command.{command_index}] (dialogue) contains an unwritten parameter!", event.id.0, page.id.0);
                                    }
                                } else if current_command_code == COMMAND_DIALOGUE_CONTINUE {
                                    current_dialogue_text.push_str("\n");
                                    current_dialogue_text.push_str(command.text.0.as_str());

                                    // And then just make sure there's no conflicting indent in any continue statements.
                                    if let Some(indent_written_into_patch) =
                                        &indent_written_into_patch
                                    {
                                        if indent_written_into_patch.0 != current_command_indent {
                                            println!("WARNING: [map.{map_name:?}.event.{}.page.{}.command.{command_index}] (dialogue) has a conflicting indent in a DIALOGUE_CONTINUE command?!", event.id.0, page.id.0);
                                        }
                                    }

                                    // I don't think dialogue commands have parameters, but it doesn't hurt
                                    // to alert the user if there is any.
                                    if !command.parameters.is_empty() {
                                        println!("WARNING: [map.{map_name:?}.event.{}.page.{}.command.{command_index}] (dialogue) contains an unwritten parameter!", event.id.0, page.id.0);
                                    }
                                } else if is_other_text {
                                    // Other commands do sometimes have parameters
                                    // But unlike dialogue where you're splicing in new commands,
                                    // patching other text doesn't affect existing parameters.
                                    // So no need to alert the user about parameters here.
                                    text.push(Text {
                                        event: event.id.0,
                                        page: page.id.0,
                                        command: command_index,
                                        original: command.text.0.clone(),
                                        patched: command.text.0.clone(),
                                    });
                                }

                                last_command_code = current_command_code;
                                last_command_indent = current_command_indent;
                                command_index += 1;
                            }
                        }
                    }
                }
            }
        }

        let dialogue = {
            if dialogue.len() > 0 {
                Some(dialogue)
            } else {
                None
            }
        };

        let text = {
            if text.len() > 0 {
                Some(text)
            } else {
                None
            }
        };

        Patch {
            dialogue,
            text,
            insert_commands: None,
        }
    }

    // NOTE: You should run this after immediately reading it from the TOML string so the dialogue string is consistent.
    pub fn trim_dialogue_ending_newline(&mut self) {
        if let Some(dialogues) = &mut self.dialogue {
            for dialogue in dialogues {
                let char_that_should_be_newline_original = &dialogue.original.pop();
                let char_that_should_be_newline_patched = &dialogue.patched.pop();

                if let Some(c) = char_that_should_be_newline_original {
                    if *c != '\n' {
                        println!("WARNING: Character of original line should end with newline! Found '{c}' instead!\n{}", dialogue.original);
                    }
                }
                if let Some(c) = char_that_should_be_newline_patched {
                    if *c != '\n' {
                        println!("WARNING: Character of patched line should end with newline! Found '{c}' instead!\n{}", dialogue.patched);
                    }
                }
                if let None = char_that_should_be_newline_original {
                    println!(
                        "WARNING: No character was popped from original line! Was it empty?\n{}",
                        dialogue.original
                    );
                }
                if let None = char_that_should_be_newline_patched {
                    println!(
                        "WARNING: No character was popped from patched line! Was it empty?\n{}",
                        dialogue.patched
                    );
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dialogue {
    pub event: i32,
    pub page: i32,
    pub command: i32,
    // All the indent seems to do is affect how it shows up in the editor GUI.
    // -----
    // ...but if you want the patched binaries to remain as close as possible to the original,
    // you'll want to keep the indents intact.
    // -----
    // Just assume that the previous indent is identical to the current indent for dialogues.
    // Assume that each page's commands indent starts with zero.
    // -----
    // But there are a few cases where the current indent =/= previous indent.
    // These are the cases that warrant writing an explict indent into the TOML patch.
    // This is basically only for times when you can't figure out the indent from the surrounding context.
    pub indent: Option<DynamicInteger>,
    //pub is_portrait: bool,
    pub original: String,
    pub patched: String,
}

// Specifically for replacing text, not for anything else, so it's not named "Replace".
#[derive(Debug, Deserialize, Serialize)]
pub struct Text {
    pub event: i32,
    pub page: i32,
    pub command: i32,
    pub original: String,
    pub patched: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InsertCommands {
    pub event: i32,
    pub page: i32,
    pub command: i32,
    pub commands: Vec<LcfMapUnitCommand>,
}

/*
Patch Splicing Dump
-------------------
Rpg2kpatcher Splicing Indexes Solution: Hashmap of original index to Option(current index)
    - If 1234567 has dialogue lines 345 and your patched lines take up 3 (reduced by 2), then the key to value are 11,22,33,4-,5-,64,75
    - - for None (Option) means find the nearest neighbor, append to index 3 or smth
    - Append after index 5 now means append after index 3, delete index 4 does nothing, replace index 6 now means replace index 4
Or maybe just create another array as a replacement?

Or maybe splicing solution is, pre-group all same event/page, then use same offset function in js code. Hashmap of vecs. Offsets only work if presorted command list. Do that so you don't have to use a moving ref vec.
*/
