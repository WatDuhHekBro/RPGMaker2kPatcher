use std::collections::HashMap;

use crate::{
    structs::{common::LcfCommand, LcfDataBase, LcfMapUnit},
    types::DynamicInteger,
    util::patch_operations,
};
use serde::{Deserialize, Serialize};

// NOTE: Both PatchDialogue and PatchText are special options to assume certain paths to avoid repeating constant numbers in the path.
// - Maps: Header #81 (Map Events) -> ID #5 (Pages) -> ID #52 (Commands)
// - Database: Header #25 (Global Events) -> ID #22 (Commands)
// Anything outside that uses different fields that aren't specialized for these common paths.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Patch {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dialogue: Vec<PatchDialogue>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<PatchText>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub splice_commands: Vec<PatchSpliceCommands>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub database_vocabulary: Vec<PatchDatabaseVocabulary>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub append_page: Vec<PatchMapAppendPage>,
}

// I have two ideas on how to add arbitrary patching if I ever need it:
// - "header = 21"
// - "path = [21, 123]"

impl Patch {
    // No idea why adding this function causes trim_dialogue_ending_newline() to get called twice, but let's just not.
    // Just call trim_dialogue_ending_newline() explicitly each time you deserialize a Patch.
    /*pub fn new(dialogue: Option<Vec<PatchDialogue>>, text: Option<Vec<PatchText>>) -> Patch {
        let mut patch = Patch {
            dialogue,
            text,
            splice_commands: None,
        };
        patch.trim_dialogue_ending_newline();
        patch
    }*/

    pub fn generate_from_map(
        map: &LcfMapUnit,
        character_names: &HashMap<i32, String>,
        map_name: &String,
        game_title: &String,
    ) -> Patch {
        patch_operations::generate_patch_from_map(map, character_names, map_name, game_title)
    }

    pub fn generate_from_database(database: &LcfDataBase, game_title: &String) -> Patch {
        patch_operations::generate_patch_from_database(database, game_title)
    }

    // NOTE: You should run this after immediately reading it from the TOML string so the dialogue string is consistent.
    pub fn trim_dialogue_ending_newline(&mut self) {
        for dialogue in &mut self.dialogue {
            let char_that_should_be_newline_original = &dialogue.original.pop();
            let char_that_should_be_newline_patched = &dialogue.patched.pop();

            if let Some(c) = char_that_should_be_newline_original {
                let c = *c;

                if c != '\n' {
                    println!("WARNING: Character of original line should end with newline! Found '{c}' instead!\n{}", dialogue.original);
                    dialogue.original.push(c);
                }
            }
            if let Some(c) = char_that_should_be_newline_patched {
                let c = *c;

                if c != '\n' {
                    //println!("WARNING: Character of patched line should end with newline! Found '{c}' instead!\n{}", dialogue.patched);
                    dialogue.should_use_custom_line_wrapping = true;
                    dialogue.patched.push(c);
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

            // Ignore the code below, "ignore_overflow" should NOT affect the lines,
            // because you already determine that via the trailing newline.
            // -----
            // Regardless of the auto-detection above, user's choice overrides the line wrap setting
            /*if let Some(ignore_overflow) = dialogue.ignore_overflow {
                dialogue.should_use_custom_line_wrapping = !ignore_overflow;
            }*/
        }
    }

    /**
     * Returns two things:
     * - A HashMap of all the Dialogue associated with an event-page pair
     * - A Vec of all the keys in order
     */
    pub fn get_ordered_dialogue(&self) -> HashMap<(i32, Option<i32>), Vec<&PatchDialogue>> {
        let mut table = HashMap::new();

        for dialogue in &self.dialogue {
            let key = (dialogue.event, dialogue.page);

            // Create entry if it hasn't reached this key yet
            if !table.contains_key(&key) {
                table.insert(key, Vec::new());
            }

            // Then work off the existing offsets table.
            let list = table
                .get_mut(&key)
                .expect("get_ordered_dialogue() HashMap should exist by this point!");

            list.push(dialogue);
        }

        table
    }

    pub fn get_sorted_dialogue_keys<'a>(
        table: &'a HashMap<(i32, Option<i32>), Vec<&'a PatchDialogue>>,
    ) -> Vec<&'a (i32, Option<i32>)> {
        let mut keys: Vec<&(i32, Option<i32>)> = table.keys().collect();
        keys.sort_by(|a, b| a.cmp(b));
        keys
    }

    /**
     * Returns two things:
     * - A HashMap of all the Text associated with an event-page pair
     * - A Vec of all the keys in order
     */
    pub fn get_ordered_text(&self) -> HashMap<(i32, Option<i32>), Vec<&PatchText>> {
        let mut table = HashMap::new();

        for text in &self.text {
            let key = (text.event, text.page);

            // Create entry if it hasn't reached this key yet
            if !table.contains_key(&key) {
                table.insert(key, Vec::new());
            }

            // Then work off the existing offsets table.
            let list = table
                .get_mut(&key)
                .expect("get_ordered_dialogue() HashMap should exist by this point!");

            list.push(text);
        }

        table
    }

    pub fn get_sorted_text_keys<'a>(
        table: &'a HashMap<(i32, Option<i32>), Vec<&'a PatchText>>,
    ) -> Vec<&'a (i32, Option<i32>)> {
        let mut keys: Vec<&(i32, Option<i32>)> = table.keys().collect();
        keys.sort_by(|a, b| a.cmp(b));
        keys
    }

    pub fn extract_text(&self, character_names: &HashMap<i32, String>) -> String {
        patch_operations::extract_text(&self, character_names)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchDialogue {
    pub event: i32,
    pub page: Option<i32>,
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
    // Just to make the patch format as clean as possible:
    // -----
    // Some(true) = true
    // Some(false) = false
    // None = false
    pub has_portrait: Option<bool>,
    pub ignore_overflow: Option<bool>,
    // The actual value of ignore_overflow
    // - User input overrides the setting
    // - But if not set by the user, it checks whether or not the string ends with a trailing newline, indicating if the patch was machine-generated
    // - This field is only used internally by the program
    // - Defaults to false, so it should be the opposite of "should_ignore_overflow"
    // Check if there's a trailing newline
    // This determines whether or not to use custom line wrapping
    #[serde(skip)]
    pub should_use_custom_line_wrapping: bool,
    // Helpful field to decipher character name variables (e.g. "\n[1]"), unused during actual patching
    pub character: Option<String>,
    pub original: String,
    pub patched: String,
}

// Specifically for replacing text, not for anything else, so it's not named "Replace".
#[derive(Debug, Deserialize, Serialize)]
pub struct PatchText {
    pub event: i32,
    pub page: Option<i32>,
    pub command: i32,
    pub has_portrait: Option<bool>,
    // NOTE: While Text can't do anything with overflow since it's one command only, this field is still used to suppress warnings in "report.html".
    pub ignore_overflow: Option<bool>,
    pub original: String,
    pub patched: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchSpliceCommands {
    pub event: i32,
    pub page: Option<i32>,
    pub replace_commands_from: i32,
    pub replace_commands_to: i32,
    pub commands: Vec<LcfCommand>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchDatabaseVocabulary {
    pub id: i32,
    pub original: String,
    pub patched: String,
}

impl PatchDatabaseVocabulary {
    pub fn convert_to_hashmap(vocab_list: &Vec<PatchDatabaseVocabulary>) -> HashMap<i32, &String> {
        let mut map = HashMap::new();

        for vocab in vocab_list {
            map.insert(vocab.id, &vocab.patched);
        }

        map
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PatchMapAppendPage {
    pub event: i32,
    pub name: Option<String>,
    // The nice thing about Rust's TOML serde implementation is that you can
    // restrict keys to numbers only, throwing an error otherwise.
    pub headers: HashMap<i32, Vec<u8>>,
    pub commands: Vec<LcfCommand>,
}
