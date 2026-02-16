use crate::{
    dialogue::{
        core::Dialogue,
        preview::{self, OverflowEntry},
    },
    structs::{
        database::LcfDataBaseHeader,
        map::LcfMapUnitPageHeader,
        patch::{
            PatchDatabaseVocabulary, PatchDialogue, PatchMapAppendPage, PatchSpliceCommands,
            PatchText,
        },
        LcfCommand, LcfCommandList, LcfDataBase, LcfMapUnit, ListEntry, ListEntryHeaderGeneric,
        Patch,
    },
    types::{
        double_byte_counted::DoubleByteCounted, DynamicInteger, DynamicIntegerArray,
        NullTerminatedList, PascalString, U8Array,
    },
    util::constants::*,
};
use regex::Regex;
use std::{collections::HashMap, sync::LazyLock};

pub fn generate_patch_from_map(
    map: &LcfMapUnit,
    character_names: &HashMap<i32, String>,
    map_name: &String,
    game_title: &String,
) -> Patch {
    let mut dialogue: Vec<PatchDialogue> = vec![];
    let mut text: Vec<PatchText> = vec![];

    // Loop through all commands
    if let Some(events) = map.get_events() {
        for event in events {
            if let Some(pages) = event.get_pages() {
                for page in pages {
                    if let Some(commands) = page.get_commands() {
                        extract_dialogue_and_text_from_commands(
                            commands,
                            &mut dialogue,
                            &mut text,
                            *event.id,
                            Some(*page.id),
                            map_name,
                            character_names,
                            game_title,
                        );
                    }
                }
            }
        }
    }

    Patch {
        dialogue,
        text,
        splice_commands: Vec::new(),
        database_vocabulary: Vec::new(),
        append_page: Vec::new(),
    }
}

pub fn generate_patch_from_database(database: &LcfDataBase, game_title: &String) -> Patch {
    let mut dialogue: Vec<PatchDialogue> = vec![];
    let mut text: Vec<PatchText> = vec![];
    let character_names = database.extract_character_names();

    // Loop through all commands
    if let Some(events) = database.get_events() {
        for event in events {
            if let Some(commands) = event.get_commands() {
                extract_dialogue_and_text_from_commands(
                    commands,
                    &mut dialogue,
                    &mut text,
                    *event.id,
                    None,
                    &String::from("RPG_RT/LcfDataBase"),
                    &character_names,
                    game_title,
                );
            }
        }
    }

    Patch {
        dialogue,
        text,
        splice_commands: Vec::new(),
        database_vocabulary: Vec::new(),
        append_page: Vec::new(),
    }
}

fn extract_dialogue_and_text_from_commands(
    commands: &LcfCommandList,
    dialogue: &mut Vec<PatchDialogue>,
    text: &mut Vec<PatchText>,
    event: i32,
    page: Option<i32>,
    map_name: &String,
    character_names: &HashMap<i32, String>,
    game_title: &String,
) {
    let mut start_index = 0;
    let mut last_command_code = -1;
    let mut current_dialogue_text = String::new();
    // NOTE: Only "dialogue" needs to worry about indents, "text" doesn't touch any other field than the text field
    // Heuristic #1: Assume the last indent if it's the same
    // Heuristic #2: Check if the last command was a branching operation (12010 or 22010)
    // Heuristic #3: Check if the last command was a multiple choice selection (20140)
    let mut last_command_indent = 0;
    let mut indent_written_into_patch: Option<DynamicInteger> = None;
    let mut has_portrait = false;
    //let mut last_portrait_condition_before_branching = false;
    // HashMap<indent level, last portrait condition before branching>
    let mut last_portrait_condition_before_branching: HashMap<i32, bool> = HashMap::new();

    for (command_index, command) in commands.iter().enumerate() {
        let command_index = command_index as i32;
        let current_command_code = *command.code;
        let current_command_indent = *command.indent;

        // 10110 as A
        // 20110 as B
        // A o --> [A]
        // A A --> [A] [A]
        // A B --> [A,B]
        // A B B --> [A,B,B]
        // A B A --> [A,B] [A]

        let was_single_line_dialogue = last_command_code == COMMAND_DIALOGUE_START
            && current_command_code != COMMAND_DIALOGUE_CONTINUE;

        let was_dialogue_terminated = last_command_code == COMMAND_DIALOGUE_CONTINUE
            && current_command_code != COMMAND_DIALOGUE_CONTINUE;

        let is_other_text = current_command_code == COMMAND_MULTIPLE_CHOICE_PROMPT
            || current_command_code == COMMAND_MULTIPLE_CHOICE_SELECTION
            || current_command_code == COMMAND_SAVE_POINT_NAME;

        if was_single_line_dialogue || was_dialogue_terminated {
            // If there's a character name variable present, add a field
            // with the original character's name for ease of use.
            static VAR_CHARACTER_PATTERN: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"\\n\[(\d+?)\]").unwrap());
            let captures = VAR_CHARACTER_PATTERN.captures(&current_dialogue_text);
            let mut character: Option<String> = None;

            // I think this should only match the first occurrence anyway.
            if let Some(captures) = captures {
                let (_full, [character_id]) = captures.extract();
                let character_id = character_id.parse::<i32>();

                if let Ok(character_id) = character_id {
                    character = character_names.get(&character_id).cloned();
                }
            }

            let has_portrait = {
                match has_portrait {
                    true => Some(true),
                    false => None,
                }
            };

            // Regular dialogue section
            dialogue.push(PatchDialogue {
                event,
                page,
                command: start_index,
                indent: indent_written_into_patch,
                has_portrait,
                ignore_overflow: None,
                // Only used when reading patch file
                should_use_custom_line_wrapping: true,
                character,
                original: current_dialogue_text.clone(),
                patched: current_dialogue_text.clone(),
            });

            current_dialogue_text = String::new();
            indent_written_into_patch = None;
        }

        if current_command_code == COMMAND_DIALOGUE_START {
            start_index = command_index;
            current_dialogue_text.push_str(command.text.as_str());

            // Compare the indent for the first line of dialogue
            indent_written_into_patch = {
                // If the indent is the same as the one before it, it can be assumed
                if current_command_indent != last_command_indent {
                    // However, if they differ, that sometimes means that a branching operation was used right before it
                    // 12010 (Branch If) indent+1
                    // 22010 (Branch Else) indent+1
                    let indent_difference = current_command_indent - last_command_indent;

                    let can_infer_increasing_indent = indent_difference == 1
                        && (last_command_code == COMMAND_BRANCH_IF
                            || last_command_code == COMMAND_BRANCH_ELSE
                            || last_command_code == COMMAND_LOOP
                            || last_command_code == COMMAND_MULTIPLE_CHOICE_SELECTION
                            || last_command_code == COMMAND_TRANSACTION
                            || last_command_code == COMMAND_NO_TRANSACTION);
                    let can_infer_decreasing_indent =
                        indent_difference == -1 && last_command_code == COMMAND_DECREASE_INDENT;

                    if can_infer_increasing_indent || can_infer_decreasing_indent {
                        None
                    } else {
                        Some(DynamicInteger(current_command_indent))
                    }
                } else {
                    None
                }
            };

            // I don't think dialogue commands have parameters, but it doesn't hurt
            // to alert the user if there is any.
            if !command.parameters.is_empty() {
                println!("WARNING: [map.{map_name}.event.{event}.page.{page:?}.command.{command_index}] (dialogue) contains an unwritten parameter!");
            }
        } else if current_command_code == COMMAND_DIALOGUE_CONTINUE {
            current_dialogue_text.push('\n');
            current_dialogue_text.push_str(command.text.as_str());

            // And then just make sure there's no conflicting indent in any continue statements.
            if let Some(indent_written_into_patch) = &indent_written_into_patch {
                if indent_written_into_patch != current_command_indent {
                    println!("WARNING: [map.{map_name}.event.{event}.page.{page:?}.command.{command_index}] (dialogue) has a conflicting indent in a DIALOGUE_CONTINUE command?!");
                }
            }

            // I don't think dialogue commands have parameters, but it doesn't hurt
            // to alert the user if there is any.
            if !command.parameters.is_empty() {
                println!("WARNING: [map.{map_name}.event.{event}.page.{page:?}.command.{command_index}] (dialogue) contains an unwritten parameter!");
            }
        } else if is_other_text {
            let has_portrait = {
                match has_portrait {
                    true => Some(true),
                    false => None,
                }
            };

            // Other commands do sometimes have parameters
            // But unlike dialogue where you're splicing in new commands,
            // patching other text doesn't affect existing parameters.
            // So no need to alert the user about parameters here.
            text.push(PatchText {
                event,
                page,
                has_portrait,
                ignore_overflow: None,
                command: command_index,
                original: command.text.clone(),
                patched: command.text.clone(),
            });
        }

        // Check for the current portrait condition
        // -----
        // NOTE: This must be placed AFTER adding any dialogue,
        // because the current dialogue is detected to end when
        // the next (current) command is non-dialogue. This means the
        // next (current) command could be clearing the portrait!
        // -----
        // NOTE: You also must take branching into account!
        // -----
        // has_portrait[0] = false
        // if some_condition
        //     has_portrait[1] = false
        //     if some_other_condition
        //         has_portrait[2] = false
        //         portrait(on)
        //         has_portrait[2] = true
        //         portrait(off)
        //         has_portrait[2] = false
        //     else
        //         has_portrait[2] = false
        //         portrait(on)
        //         has_portrait[2] = true
        //     has_portrait[1] = false
        //     portrait(on)
        //     has_portrait[1] = true
        // else
        //     has_portrait[1] = false
        //     portrait(on)
        //     has_portrait[1] = true
        //     if some_other_condition
        //         has_portrait[1] = true
        //         portrait(off)
        //         has_portrait[1] = false
        //     else
        //         has_portrait[1] = true
        // has_portrait[0] = false
        // -----
        // If the indent decreases, assume that the dev cleared any portrait condition by then.
        // If the indent increases, assume the dev has got it all under control.
        /*if current_command_indent == last_command_indent - 1 {
            has_portrait = false;
        }*/
        // -----
        // Forgot all the above, the more robust way is to keep track of each indent level's portrait condition.
        // -----
        // Never mind. Let's try this again.
        // If the indent increases, store the condition before branching.
        if current_command_indent == last_command_indent + 1 {
            //last_portrait_condition_before_branching = has_portrait;
            last_portrait_condition_before_branching.insert(last_command_indent, has_portrait);
        }
        // And if the indent decreases, rollback to the condition before branching.
        else if current_command_indent == last_command_indent - 1 {
            //has_portrait = last_portrait_condition_before_branching;
            let condition = last_portrait_condition_before_branching.get(&current_command_indent);

            if let Some(condition) = condition {
                has_portrait = *condition;
            } else {
                println!("WARNING: last_portrait HashMap somehow has missing value when decreasing indent?!\n{last_portrait_condition_before_branching:?}");
            }
        }
        // Okay, at this point, I give up. I'll never know the full picture anyway.
        // Theoretically, you could show a portrait before a split, then some choices could clear the portrait,
        // meaning after the split ends you'd have inconsistent results.
        // At this point, best to just leave it to manual editing when necessary.
        // After all, this is only going to be used for automatic line wrapping.
        // No need to work so hard on what is essentially an optional feature.

        // But if it happens to land on a change face graphic command at the same time,
        // then let this override whatever the previous heuristic determined.
        if current_command_code == COMMAND_CHANGE_FACE_GRAPHIC {
            has_portrait = !command.text.is_empty();
        }
        // Special edge case for Velsarbor, the clear portrait command is abstracted away into a specific global event call
        else if game_title == "Velsarbor" && current_command_code == COMMAND_CALL_GLOBAL_EVENT {
            let global_event_id = command.parameters.get(1);

            if let Some(global_event_id) = global_event_id {
                if global_event_id == 118 || global_event_id == 119 {
                    has_portrait = false;
                }
            }
        }

        last_command_code = current_command_code;
        last_command_indent = current_command_indent;
    }
}

pub fn apply_patch_map(
    map: &mut LcfMapUnit,
    patch: &Patch,
    character_names: &HashMap<i32, String>,
) {
    let mut offsets_table: HashMap<(i32, Option<i32>), Vec<isize>> = HashMap::new();

    for dialogue in &patch.dialogue {
        let PatchDialogue {
            event,
            page,
            command: command_index,
            indent: explicitly_defined_indent,
            has_portrait,
            ignore_overflow: _,
            should_use_custom_line_wrapping,
            character: _,
            original,
            patched,
        } = dialogue;
        let event = *event;
        let page = page.expect(ERROR_MAP_PAGE_NONE);
        let key = (event, Some(page));

        let commands = &mut map
            .get_event_mut(event)
            .expect("Event should exist!")
            .get_page_mut(page)
            .expect("Page should exist!")
            .get_commands_mut()
            .expect("Commands should exist!");

        // Get split lines
        let has_portrait = has_portrait.unwrap_or(false);

        let parsed_dialogue = Dialogue::from(
            patched,
            has_portrait,
            character_names,
            *should_use_custom_line_wrapping,
        );

        splice_dialogue_and_update_offsets(
            commands,
            *command_index,
            explicitly_defined_indent,
            original,
            &parsed_dialogue.processed_lines,
            &mut offsets_table,
            key,
        );
    }

    for text in &patch.text {
        let PatchText {
            event,
            page,
            command: command_index,
            has_portrait: _,
            ignore_overflow: _,
            original: _,
            patched,
        } = text;
        let event = *event;
        let page = page.expect(ERROR_MAP_PAGE_NONE);
        let key = (event, Some(page));

        let commands = &mut map
            .get_event_mut(event)
            .expect("Event should exist!")
            .get_page_mut(page)
            .expect("Page should exist!")
            .get_commands_mut()
            .expect("Commands should exist!");

        // Get explicit indent if available or assume previous indent
        let command_index = *command_index as usize;
        let start_index = {
            let offsets = offsets_table.get(&key);

            if let Some(offsets) = offsets {
                ((command_index as isize) + (offsets[command_index])).max(0) as usize
            } else {
                command_index
            }
        };

        // Replace text
        commands[start_index].text = PascalString::from(patched);
    }

    for splice_command in &patch.splice_commands {
        let PatchSpliceCommands {
            event,
            page,
            replace_commands_from,
            replace_commands_to,
            commands: patched_commands,
        } = splice_command;
        let event = *event;
        let page = page.expect(ERROR_MAP_PAGE_NONE);
        let key = (event, Some(page));

        let commands = &mut map
            .get_event_mut(event)
            .expect("Event should exist!")
            .get_page_mut(page)
            .expect("Page should exist!")
            .get_commands_mut()
            .expect("Commands should exist!");

        splice_arbitrary_commands_and_update_offsets(
            commands,
            *replace_commands_from,
            *replace_commands_to,
            patched_commands.clone(),
            &mut offsets_table,
            key,
        );
    }

    for entry in &patch.append_page {
        let PatchMapAppendPage {
            event,
            name,
            headers: headers_map,
            commands,
        } = entry;

        let pages = &mut map
            .get_event_mut(*event)
            .expect("Event should exist!")
            .get_pages_mut()
            .expect("Pages should exist!");
        let mut headers: NullTerminatedList<LcfMapUnitPageHeader> = NullTerminatedList(Vec::new());

        // Push name if it exists
        if let Some(name) = name {
            headers.push(LcfMapUnitPageHeader::Name(PascalString::from(name)));
        }

        // Push the commands list
        headers.push(LcfMapUnitPageHeader::Commands(DoubleByteCounted {
            inner: LcfCommandList(commands.to_vec()),
            next_id: DynamicInteger(52),
        }));

        // Push all of the generic headers
        for (id, bytes) in headers_map {
            headers.push(LcfMapUnitPageHeader::Generic(ListEntryHeaderGeneric {
                id: DynamicInteger(*id),
                value: U8Array(bytes.to_vec()),
            }));
        }

        // You MUST make sure to sort the headers in order or the binary output will differ!
        // -----
        // Disgusting hardcoded numbers because I can't figure out how to extract the magic numbers of LcfMapUnitPageHeader
        headers.sort_by(|a, b| {
            let id_of_a = {
                match a {
                    LcfMapUnitPageHeader::Name(_) => 21,
                    // The extracted ID is only used for comparison purposes, so it doesn't matter if it's 51 or 52
                    LcfMapUnitPageHeader::Commands(_) => 52,
                    LcfMapUnitPageHeader::Generic(header) => *header.id,
                }
            };

            let id_of_b = {
                match b {
                    LcfMapUnitPageHeader::Name(_) => 21,
                    // The extracted ID is only used for comparison purposes, so it doesn't matter if it's 51 or 52
                    LcfMapUnitPageHeader::Commands(_) => 52,
                    LcfMapUnitPageHeader::Generic(header) => *header.id,
                }
            };

            id_of_a.cmp(&id_of_b)
        });

        pages.push(ListEntry {
            id: DynamicInteger((pages.len() as i32) + 1),
            headers,
        });
    }
}

pub fn apply_patch_database(
    database: &mut LcfDataBase,
    patch: &Patch,
    character_names: &HashMap<i32, String>,
) {
    let mut offsets_table: HashMap<(i32, Option<i32>), Vec<isize>> = HashMap::new();

    for dialogue in &patch.dialogue {
        let PatchDialogue {
            event,
            page: _,
            command: command_index,
            indent: explicitly_defined_indent,
            has_portrait,
            ignore_overflow: _,
            should_use_custom_line_wrapping,
            character: _,
            original,
            patched,
        } = dialogue;
        let event = *event;
        let key = (event, None);

        let commands = &mut database
            .get_event_mut(event)
            .expect("Event should exist!")
            .get_commands_mut()
            .expect("Commands should exist!");

        // Get split lines
        let has_portrait = has_portrait.unwrap_or(false);

        let parsed_dialogue = Dialogue::from(
            patched,
            has_portrait,
            character_names,
            *should_use_custom_line_wrapping,
        );

        splice_dialogue_and_update_offsets(
            commands,
            *command_index,
            explicitly_defined_indent,
            original,
            &parsed_dialogue.processed_lines,
            &mut offsets_table,
            key,
        );
    }

    for text in &patch.text {
        let PatchText {
            event,
            page: _,
            command: command_index,
            has_portrait: _,
            ignore_overflow: _,
            original: _,
            patched,
        } = text;
        let event = *event;
        let key = (event, None);

        let commands = &mut database
            .get_event_mut(event)
            .expect("Event should exist!")
            .get_commands_mut()
            .expect("Commands should exist!");

        // Get explicit indent if available or assume previous indent
        let command_index = *command_index as usize;
        let start_index = {
            let offsets = offsets_table.get(&key);

            if let Some(offsets) = offsets {
                ((command_index as isize) + (offsets[command_index])).max(0) as usize
            } else {
                command_index
            }
        };

        // Replace text
        commands[start_index].text = PascalString::from(patched);
    }

    for splice_command in &patch.splice_commands {
        let PatchSpliceCommands {
            event,
            page: _,
            replace_commands_from,
            replace_commands_to,
            commands: patched_commands,
        } = splice_command;
        let event = *event;
        let key = (event, None);

        let commands = &mut database
            .get_event_mut(event)
            .expect("Event should exist!")
            .get_commands_mut()
            .expect("Commands should exist!");

        splice_arbitrary_commands_and_update_offsets(
            commands,
            *replace_commands_from,
            *replace_commands_to,
            patched_commands.clone(),
            &mut offsets_table,
            key,
        );
    }

    let vocab_map = PatchDatabaseVocabulary::convert_to_hashmap(&patch.database_vocabulary);

    for header in &mut **database {
        if let LcfDataBaseHeader::Vocabulary(header) = header {
            for vocab_entry in &mut ***header {
                let patched_vocab_text = vocab_map.get(&vocab_entry.id);

                if let Some(patched_vocab_text) = patched_vocab_text {
                    vocab_entry.text = PascalString::from(*patched_vocab_text);
                }
            }
        }
    }
}

// Since Array.splice is a dynamic function, you need to adjust for things that'll change the index.
// The offset tracked will be different for every event-page pair.
// Targeted command index also must be in order, if you want this simplified offset method to work (and not have to use a HashMap of references).
// -----
// Or just forget the above. Use the HashMap method to keep track of where everything is.
// This method provides resilience against out-of-order TOML. The patch order isn't dependent on user-edited TOML.
// -----
// HashMap<(event, page), Vec<(original_index), shifted_index>>
// Make sure the offsets is NOT unsigned, you need those negative numbers!
fn splice_dialogue_and_update_offsets(
    commands: &mut LcfCommandList,
    command_index: i32,
    explicitly_defined_indent: &Option<DynamicInteger>,
    original: &str,
    patched_lines: &Vec<String>,
    offsets_table: &mut HashMap<(i32, Option<i32>), Vec<isize>>,
    key: (i32, Option<i32>),
) {
    // Create offset entry if it hasn't worked on this key yet
    offsets_table
        .entry(key)
        .or_insert_with(|| vec![0; commands.len()]);

    // Then work off the existing offsets table.
    let offsets = offsets_table
        .get_mut(&key)
        .expect("Offsets HashMap should exist by this point!");

    // Get explicit indent if available or assume previous indent
    let command_index = command_index as usize;
    let start_index = ((command_index as isize) + (offsets[command_index])).max(0) as usize;

    let previous_indent = {
        if let Some(indent) = explicitly_defined_indent {
            indent.0
        } else if start_index > 0 {
            let last_command = &commands[start_index - 1];
            let last_command_indent = last_command.indent.0;
            let last_command_code = *last_command.code;

            if last_command_code == COMMAND_BRANCH_IF
                || last_command_code == COMMAND_BRANCH_ELSE
                || last_command_code == COMMAND_LOOP
                || last_command_code == COMMAND_MULTIPLE_CHOICE_SELECTION
                || last_command_code == COMMAND_TRANSACTION
                || last_command_code == COMMAND_NO_TRANSACTION
            {
                last_command_indent + 1
            } else if last_command_code == COMMAND_DECREASE_INDENT {
                (last_command_indent - 1).max(0)
            } else {
                last_command_indent
            }
        } else {
            0
        }
    };

    // Generate patched commands
    let mut patched_commands: Vec<LcfCommand> = Vec::new();
    let patched_lines_count = patched_lines.len();
    let mut is_first_line = true;

    if patched_lines.len() > 4 {
        println!("ERROR: More than 4 lines of dialogue found!");
    }

    for line in patched_lines {
        if is_first_line {
            patched_commands.push(LcfCommand {
                code: DynamicInteger(COMMAND_DIALOGUE_START),
                indent: DynamicInteger(previous_indent),
                text: PascalString::from(line),
                parameters: DynamicIntegerArray(Vec::new()),
            });

            is_first_line = false;
        } else {
            patched_commands.push(LcfCommand {
                code: DynamicInteger(COMMAND_DIALOGUE_CONTINUE),
                indent: DynamicInteger(previous_indent),
                text: PascalString::from(line),
                parameters: DynamicIntegerArray(Vec::new()),
            });
        }
    }

    // Splice
    let original_lines: Vec<&str> = original.split("\n").collect();
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
    // ----- Issues -----
    // Issue: Targeting moved dialogue is offset -1 more than necessary.
    // You should instead treat the offset table like pointers to new locations.
    // Where exactly does it make sense to point to?
    //
    // [a, b, c, d, e, f, g, h, i, j,[k,  l,  m,  n,] o,  p,  q,  r,  s,  t,  u,  v,  w,  x,  y,  z]
    // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25]
    //
    // [a, b, c, d, e, f, g, h, i, j, o,  p,  q,  r,  s,  t,  u,  v,  w,  x,  y,  z] (new list)
    // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21] (new indexes)
    //                                    -1  -2  -3  -4  -4  -4  -4  -4  -4  -4  -4
    // [a, b, c, d, e, f, g, h, i, j, k,  l,  m,  n,  o,  p,  q,  r,  s,  t,  u,  v,  w,  x,  y,  z]
    // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21] (new target indexes)
    //
    // The rationale behind this new change is that with the old system, targeting "n" would get you to target "j", it just doesn't make sense.
    //
    // [a, b, c, d, e, f]
    // [0, 1, 2, 3, 4, 5]
    //
    // [a, b, c, x, y, z, d, e, f] (new list)
    // [0, 1, 2, 3, 4, 5, 6, 7, 8] (new indexes)
    //          +3 +3 +3
    // [a, b, c, d, e, f]
    // [0, 1, 2, 6, 7, 8] (new target indexes)
    //
    // When splicing in more entries, this seems to be a non-issue still.
    let length_difference = (patched_lines_count as isize) - (original_lines_count as isize);

    // Just in case, if you're splicing in more elements than the original,
    // just extend the array with the existing last offset to avoid index OOB.
    if length_difference > 0 {
        let last_offset = offsets[offsets.len() - 1];

        for _ in 0..length_difference {
            offsets.push(last_offset);
        }
    }

    for (offsets_index, offset) in offsets.iter_mut().enumerate().skip(command_index) {
        // If the length_difference is negative, you need to take the distance from the command_index into account.
        // Event #75, lines 0-2 => 0 (diff = -2), offsets = [0, -1, -2, -2, -2, ...]
        // 0-0 = 0, 0-1 = -1, 0-2 = -2, 0-3 = -3
        if length_difference < 0 {
            let distance_from_original_index = (command_index as isize) - (offsets_index as isize);
            *offset += length_difference.max(distance_from_original_index);
        } else {
            *offset += length_difference;
        }
    }

    //println!("{offsets:?} <== {length_difference}");
}

fn splice_arbitrary_commands_and_update_offsets(
    commands: &mut LcfCommandList,
    replace_commands_from: i32,
    replace_commands_to: i32,
    patched_commands: Vec<LcfCommand>,
    offsets_table: &mut HashMap<(i32, Option<i32>), Vec<isize>>,
    key: (i32, Option<i32>),
) {
    // Create offset entry if it hasn't worked on this key yet
    offsets_table
        .entry(key)
        .or_insert_with(|| vec![0; commands.len()]);

    // Then work off the existing offsets table.
    let offsets = offsets_table
        .get_mut(&key)
        .expect("Offsets HashMap should exist by this point!");

    // Splice
    let splice_out_size = replace_commands_to - replace_commands_from;
    let splice_in_size = patched_commands.len();

    let start_index = ((replace_commands_from as isize) + (offsets[replace_commands_from as usize]))
        .max(0) as usize;
    let stop_index = start_index + (splice_out_size as usize);
    let splice_range = start_index..stop_index;

    commands.splice(splice_range, patched_commands);

    // Then update indexes
    let length_difference = (splice_in_size as isize) - (splice_out_size as isize);

    // Just in case, if you're splicing in more elements than the original,
    // just extend the array with the existing last offset to avoid index OOB.
    if length_difference > 0 {
        let last_offset = offsets[offsets.len() - 1];

        for _ in 0..length_difference {
            offsets.push(last_offset);
        }
    }

    for (offsets_index, offset) in offsets
        .iter_mut()
        .enumerate()
        .skip(replace_commands_from as usize)
    {
        // If the length_difference is negative, you need to take the distance from the command_index into account.
        // Event #75, lines 0-2 => 0 (diff = -2), offsets = [0, -1, -2, -2, -2, ...]
        // 0-0 = 0, 0-1 = -1, 0-2 = -2, 0-3 = -3
        if length_difference < 0 {
            let distance_from_original_index =
                (replace_commands_from as isize) - (offsets_index as isize);
            *offset += length_difference.max(distance_from_original_index);
        } else {
            *offset += length_difference;
        }
    }

    //println!("{offsets:?} <== {length_difference}");
}

pub fn generate_html_preview(
    patch: &Patch,
    map_name: &String,
    character_names: &HashMap<i32, String>,
    // Extremely janky mutable reference in order to not have to parse Dialogue all over again
    overflow_list: &mut Vec<OverflowEntry>,
) -> String {
    preview::generate_html_preview(
        &patch.get_ordered_dialogue(),
        &patch.get_ordered_text(),
        map_name,
        character_names,
        overflow_list,
    )
}

pub fn extract_text(patch: &Patch, character_names: &HashMap<i32, String>) -> String {
    let dialogues_table = patch.get_ordered_dialogue();
    let texts_table = patch.get_ordered_text();
    let mut output_original =
        String::from("#######################\n# Original (Dialogue) #\n#######################\n");
    let mut output_patched =
        String::from("\n######################\n# Patched (Dialogue) #\n######################\n");
    let mut is_patched_identical_to_original = true;

    for key in Patch::get_sorted_dialogue_keys(&dialogues_table) {
        let dialogues = dialogues_table
            .get(key)
            .expect("Invalid Key! extract_text -> Patch::get_sorted_dialogue_keys()");
        let (event, page) = key;

        if let Some(page) = page {
            output_original.push_str(&format!(
                "\n==========[ Event #{event} / Page #{page} ]==========\n\n"
            ));
            output_patched.push_str(&format!(
                "\n==========[ Event #{event} / Page #{page} ]==========\n\n"
            ));
        } else {
            output_original.push_str(&format!("\n==========[ Event #{event} ]==========\n\n"));
            output_patched.push_str(&format!("\n==========[ Event #{event} ]==========\n\n"));
        }

        for dialogue in dialogues {
            let original = &dialogue.original;
            let patched = &dialogue.patched;

            if is_patched_identical_to_original && (original != patched) {
                is_patched_identical_to_original = false;
            }

            output_original.push_str(&clean_escaped_text(original, character_names));
            output_patched.push_str(&clean_escaped_text(patched, character_names));
        }
    }

    if !patch.text.is_empty() {
        output_original
            .push_str("\n###################\n# Original (Text) #\n###################\n");
        output_patched.push_str("\n##################\n# Patched (Text) #\n##################\n");
    }

    for key in Patch::get_sorted_text_keys(&texts_table) {
        let texts = texts_table
            .get(key)
            .expect("Invalid Key! extract_text -> Patch::get_sorted_text_keys()");
        let (event, page) = key;

        if let Some(page) = page {
            output_original.push_str(&format!(
                "\n==========[ Event #{event} / Page #{page} ]==========\n\n"
            ));
            output_patched.push_str(&format!(
                "\n==========[ Event #{event} / Page #{page} ]==========\n\n"
            ));
        } else {
            output_original.push_str(&format!("\n==========[ Event #{event} ]==========\n\n"));
            output_patched.push_str(&format!("\n==========[ Event #{event} ]==========\n\n"));
        }

        for text in texts {
            let original = &text.original;
            let patched = &text.patched;

            if is_patched_identical_to_original && (original != patched) {
                is_patched_identical_to_original = false;
            }

            output_original.push_str(&clean_escaped_text(original, character_names));
            output_patched.push_str(&clean_escaped_text(patched, character_names));
        }
    }

    let mut output = String::new();
    output.push_str(&output_original);

    if !is_patched_identical_to_original {
        output.push_str(&output_patched);
    }

    output
}

// Removes all escape characters and replaces \\n[#] with characters.
pub fn clean_escaped_text(text: &str, character_names: &HashMap<i32, String>) -> String {
    static ESCAPED_EXCLUDING_CHAR_AND_VAR_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\\[^NnVv](:?\[(\d+?)\])?").unwrap());
    static MULTI_SPACE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" {2,}").unwrap());

    let mut output = String::from(ESCAPED_EXCLUDING_CHAR_AND_VAR_PATTERN.replace_all(text, ""));

    for (id, name) in character_names {
        output = output.replace(&format!(r"\n[{id}]"), name);
    }

    let output = output.replace("\n", " ");
    let mut output = MULTI_SPACE_PATTERN.replace_all(&output, " ").to_string();
    output.push('\n');

    output
}
