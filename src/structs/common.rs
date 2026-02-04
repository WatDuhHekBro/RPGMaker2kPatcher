use crate::types::{DynamicInteger, DynamicIntegerArray, PascalString};
use binrw::binrw;
use serde::{Deserialize, Serialize};

#[binrw]
#[derive(Debug, Deserialize, Serialize)]
#[brw(big)]
pub struct LcfCommonCommand {
    pub code: DynamicInteger, // 1st
    // If I remember correctly, the indent is mostly just for viewing it in an editor (EasyRPG Editor or the official RPGMaker2k)
    pub indent: DynamicInteger,          // 2nd
    pub text: PascalString,              // 3rd
    pub parameters: DynamicIntegerArray, // 4th
}
