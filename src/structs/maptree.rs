// An incomplete representation of an LcfMapTree.
// But it doesn't matter because I'm not patching this.
// This is just good enough to read the maps section.
// There's an unknown section of bytes at the end of the file I didn't bother parsing.

use crate::{
    structs::{ListEntry, ListEntryHeaderGeneric},
    types::{ByteCounted, DynamicInteger, PascalString, PreallocatedList},
    util::toml::generate_toml_maptree,
};
use binrw::binrw;
use serde::{Deserialize, Serialize};

// Make sure to place this #[derive(Debug, Deserialize, Serialize)] below #[binrw], or it'll throw errors for temporary fields.
// -----
// NOTE: All headers are kept in a Vec instead of a HashMap in order to guarantee
// preserving the original order, remaining as close as possible to the original binary.
// Tradeoff: Uses less efficient helper functions to access common fields.

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big, magic = b"\x0ALcfMapTree")]
pub struct LcfMapTree(pub PreallocatedList<ListEntry<LcfMapTreeMapHeader>>);

impl LcfMapTree {
    pub fn extract_game_title(&self) -> &String {
        for map in &self.0 .0 {
            if map.id == 0 {
                for header in &map.headers.0 {
                    if let LcfMapTreeMapHeader::Name(PascalString(name)) = header {
                        return name;
                    }
                }
            }
        }

        panic!("All LcfMapTree's should start with Map #0 with the game's title! Why doesn't this one have it?");
    }

    pub fn generate_toml_maptree(&self) -> String {
        generate_toml_maptree(self)
    }
}

/*impl std::ops::Deref for LcfMapTree {
    type Target = Vec<LcfMapTreeMapHeader>;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl std::ops::DerefMut for LcfMapTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut (self.0 .0)
    }
}*/

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
pub enum LcfMapTreeMapHeader {
    #[brw(magic = 1u8)]
    Name(PascalString),
    #[brw(magic = 2u8)]
    ParentMap(ByteCounted<DynamicInteger>),
    Generic(ListEntryHeaderGeneric),
}
