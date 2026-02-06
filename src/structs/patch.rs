use std::collections::HashMap;

use crate::{
    structs::{common::LcfCommand, LcfDataBase, LcfMapUnit},
    types::DynamicInteger,
    util::patch_operations,
};
use serde::{Deserialize, Serialize};

// NOTE: Both Dialogue and Text are special options to assume certain paths to avoid repeating constant numbers in the path.
// - Maps: Header #81 (Map Events) -> ID #5 (Pages) -> ID #52 (Commands)
// - Database: Header #25 (Global Events) -> ID #22 (Commands)
// Anything outside that uses different fields that aren't specialized for these common paths.
#[derive(Debug, Deserialize, Serialize)]
pub struct Patch {
    // These have to be made optional in order for serde to be able to read the TOML file directly
    pub dialogue: Option<Vec<Dialogue>>,
    pub text: Option<Vec<Text>>,
    pub insert_commands: Option<Vec<InsertCommands>>,
}

impl Patch {
    // No idea why adding this function causes trim_dialogue_ending_newline() to get called twice, but let's just not.
    // Just call trim_dialogue_ending_newline() explicitly each time you deserialize a Patch.
    /*pub fn new(dialogue: Option<Vec<Dialogue>>, text: Option<Vec<Text>>) -> Patch {
        let mut patch = Patch {
            dialogue,
            text,
            insert_commands: None,
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

    pub fn extract_text(&self, character_names: &HashMap<i32, String>) -> String {
        patch_operations::extract_text(&self, character_names)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dialogue {
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
    // Helpful field to decipher character name variables (e.g. "\n[1]"), unused during actual patching
    pub character: Option<String>,
    pub original: String,
    pub patched: String,
}

// Specifically for replacing text, not for anything else, so it's not named "Replace".
#[derive(Debug, Deserialize, Serialize)]
pub struct Text {
    pub event: i32,
    pub page: Option<i32>,
    pub command: i32,
    pub has_portrait: Option<bool>,
    pub original: String,
    pub patched: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InsertCommands {
    pub event: i32,
    pub page: i32,
    pub command: i32,
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
