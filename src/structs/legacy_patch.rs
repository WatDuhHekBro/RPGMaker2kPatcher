use std::collections::HashMap;

use crate::{structs::Patch, util::constants::ERROR_MAP_PAGE_NONE};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LegacyPatch {
    dialogue: Option<Vec<LegacyPatchDialogue>>,
    other: Option<Vec<LegacyPatchOther>>,
}

#[derive(Debug, Deserialize)]
struct LegacyPatchDialogue {
    // A 4-tuple integer array consisting of: [event #, page #, command start, command length]
    // Note that the command length is for the original lines, not the patched lines.
    path: [i32; 4],
    //original: String,
    lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LegacyPatchOther {
    path: [i32; 3],
    //original: String,
    patch: String,
}

impl LegacyPatch {
    pub fn import_lines_to_toml_map_patch(self, patch: &mut Patch, map_name: &String) {
        // 3-tuple identifier [event, page, command]
        let mut patch_map = HashMap::<(i32, i32, i32), String>::new();

        // Build patch map for dialogue
        if let Some(old_dialogues) = self.dialogue {
            for dialogue in old_dialogues {
                let event = dialogue.path[0];
                let page = dialogue.path[1];
                let command = dialogue.path[2];
                let patched_text = dialogue.lines.join("\n");

                patch_map.insert((event, page, command), patched_text);
            }
        }

        // Build patch map for other text
        if let Some(old_other) = self.other {
            for text in old_other {
                let event = text.path[0];
                let page = text.path[1];
                let command = text.path[2];
                let patched_text = text.patch;

                patch_map.insert((event, page, command), patched_text);
            }
        }

        // Import lines to new patch format
        if let Some(dialogues) = &mut patch.dialogue {
            for dialogue in dialogues {
                let event = dialogue.event;
                let page = dialogue.page.expect(ERROR_MAP_PAGE_NONE);
                let command = dialogue.command;
                let key = &(event, page, command);
                let patched_text = patch_map.get(key);

                if let Some(patched_text) = patched_text {
                    dialogue.patched = patched_text.to_string();
                    patch_map.remove(key);
                } else {
                    println!("WARNING: [map.{map_name}.event.{event}.page.{page}.command.{command}] found no equivalent dialogue in its legacy patch!");
                }
            }
        }

        // Import lines to new patch format
        if let Some(texts) = &mut patch.text {
            for text in texts {
                let event = text.event;
                let page = text.page.expect(ERROR_MAP_PAGE_NONE);
                let command = text.command;
                let key = &(event, page, command);
                let patched_text = patch_map.get(key);

                if let Some(patched_text) = patched_text {
                    text.patched = patched_text.to_string();
                    patch_map.remove(key);
                } else {
                    println!("WARNING: [map.{map_name}.event.{event}.page.{page}.command.{command}] found no equivalent dialogue in its legacy patch!");
                }
            }
        }

        if !patch_map.is_empty() {
            println!("WARNING: Some legacy entries weren't used in the conversion process for {map_name}!\n{patch_map:?}");
        }
    }
}
