use crate::structs::{map::*, LcfMapUnit};
use serde::{Deserialize, Serialize};

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
