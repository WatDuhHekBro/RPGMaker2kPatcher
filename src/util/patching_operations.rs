use crate::{
    structs::{
        patch::{Dialogue, Text},
        LcfCommonCommand, LcfMapUnit, Patch,
    },
    types::{DynamicInteger, DynamicIntegerArray, PascalString},
    util::constants::*,
};
use std::collections::HashMap;

pub fn generate_patch_from_map(map: &LcfMapUnit, map_name: Option<&String>) -> Patch {
    let mut dialogue: Vec<Dialogue> = vec![];
    let mut text: Vec<Text> = vec![];

    // Loop through all commands
    if let Some(events) = map.get_events() {
        for event in events {
            if let Some(pages) = event.get_pages() {
                for page in pages {
                    if let Some(commands) = page.get_commands() {
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
                                if let Some(indent_written_into_patch) = &indent_written_into_patch
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

pub fn apply_patch(map: &mut LcfMapUnit, patch: &Patch) {
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
                let commands = map
                    .get_event(*event)
                    .expect("Event should exist!")
                    .get_page(*page)
                    .expect("Page should exist!")
                    .get_commands()
                    .expect("Commands should exist!");
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

            let commands = &mut map
                .get_event_mut(*event)
                .expect("Event should exist!")
                .get_page_mut(*page)
                .expect("Page should exist!")
                .get_commands_mut()
                .expect("Commands should exist!");

            // Get explicit indent if available or assume previous indent
            let command_index = *command_index as usize;
            let start_index = ((command_index as isize) + (offsets[command_index])).max(0) as usize;

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
            let mut patched_commands: Vec<LcfCommonCommand> = Vec::new();
            let patched_lines = patched.split("\n").collect::<Vec<&str>>();
            let patched_lines_count = patched_lines.len();
            let mut is_first_line = true;

            if patched_lines.len() > 4 {
                println!("ERROR: More than 4 lines of dialogue found!");
            }

            for line in patched_lines {
                if is_first_line {
                    patched_commands.push(LcfCommonCommand {
                        code: DynamicInteger(COMMAND_DIALOGUE_START),
                        indent: DynamicInteger(previous_indent),
                        text: PascalString::from(line),
                        parameters: DynamicIntegerArray(Vec::new()),
                    });

                    is_first_line = false;
                } else {
                    patched_commands.push(LcfCommonCommand {
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
                let commands = map
                    .get_event(*event)
                    .expect("Event should exist!")
                    .get_page(*page)
                    .expect("Page should exist!")
                    .get_commands()
                    .expect("Commands should exist!");
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

            let commands = &mut map
                .get_event_mut(*event)
                .expect("Event should exist!")
                .get_page_mut(*page)
                .expect("Page should exist!")
                .get_commands_mut()
                .expect("Commands should exist!")
                .0;

            // Get explicit indent if available or assume previous indent
            let command_index = *command_index as usize;
            let start_index = ((command_index as isize) + (offsets[command_index])).max(0) as usize;

            // Replace text
            commands[start_index].text = PascalString::from(patched);
        }
    }
}
