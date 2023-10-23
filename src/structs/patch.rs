use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Patch<S: Into<String>> {
    pub dialogue: Vec<Dialogue<S>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Dialogue<S: Into<String>> {
    pub event: u16,
    pub page: u16,
    pub command_start: u16,
    pub command_length: u16,
    pub original: S,
    pub patched: S,
}

#[derive(Deserialize, Debug)]
pub struct LegacyPatch {
    dialogue: Vec<LegacyPatchDialogue>,
    other: Vec<LegacyPatchOther>,
}

#[derive(Deserialize, Debug)]
struct LegacyPatchDialogue {
    // A 4-tuple integer array consisting of: [event #, page #, command start, command length]
    // Note that the command length is for the original lines, not the patched lines.
    path: [u16; 4],
    original: String,
    lines: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct LegacyPatchOther {}

impl LegacyPatch {
    pub fn convert_to_toml_patch(self) /*-> Patch*/
    {
        //Patch { dialogue: () }
    }
}
