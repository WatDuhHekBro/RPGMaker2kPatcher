// The reason this error is thrown in every potential line instead of propagating upwards via "?" is because
// if you used "?" for all of the DynamicInteger::read()'s, you'll only see the error thrown in your main function.
// Not helpful at all for debugging issues.
pub const ERROR_BINRW_READ: &str = "Binary read failed!";
pub const ERROR_MAP_PAGE_NONE: &str =
    "All LcfMapUnit patches MUST have a \"page\" number! It is only optional for the LcfDataBase!";
pub const ERROR_NO_FILE_DATABASE: &str = "You're missing the LcfDataBase file (RPG_RT.ldb) in the target directory! This file is essential for processing!";
pub const ERROR_NO_FILE_MAPTREE: &str = "You're missing the LcfMapTree file (RPG_RT.lmt) in the target directory! This file is essential for generating TOML patches!";

// Dialogue-related commands
pub const COMMAND_DIALOGUE_START: i32 = 10110;
pub const COMMAND_DIALOGUE_CONTINUE: i32 = 20110;
// There is a LOT to unpack here actually, especially for implementing the "is_portrait" field.
// -----
// The reason this command is important is because if you want to implement line auto-wrapping,
// you need to know how many characters you can safely fit into the dialogue box.
// - For boxes with portraits, you can safely display up to 38 characters.
// - For boxes without portraits, you can safely display up to 50 characters.
// -----
// So how exactly does the game engine know if there's a portrait or not?
// - Command 10130 with an empty string ("") will clear any existing portraits.
// - Command 10130 with any non-empty string (e.g. "Z") will attempt to look for an image file in the "FaceSet" folder.
// - Portraits will remain in-place until changed.
//     - If you have multiple lines of dialogue, the portrait will carry over.
//     - Meaning that you don't need this command for every dialogue box.
// - This seems to be the only rule for this command, regardless of indent or parameters.
//   Those don't have a direct impact on the box's final width.
// -----
// Simple, right?
// Wrong. Velsarbor specifically abstracts this out into a global/system event call (command 12330).
// As far as I'm aware, Velsarbor is the only game I've come across to do this, but regardless, you'll
// need to make an exception for this game.
// Other games I've looked at seem to just use the normal method to clear the portrait.
// -----
// How do you make that exception? How do you check which game you're patching?
// You'll need to decompile the LcfMapTree. There under Map ID #0 is the name of the game.
pub const COMMAND_CHANGE_FACE_GRAPHIC: i32 = 10130;

// Text-related commands
pub const COMMAND_MULTIPLE_CHOICE_PROMPT: i32 = 10140;
pub const COMMAND_MULTIPLE_CHOICE_SELECTION: i32 = 20140;
pub const COMMAND_SAVE_POINT_NAME: i32 = 10610;

// Other commands
pub const COMMAND_DECREASE_INDENT: i32 = 10;
pub const COMMAND_BRANCH_IF: i32 = 12010;
pub const COMMAND_BRANCH_ELSE: i32 = 22010;
pub const COMMAND_LOOP: i32 = 12210;
// The solution to the problem above is to make an exception for Velsarbor.
// You need to check the command parameters for the specific event it calls.
// - [0, 118, 0] = Call global event #118 (>|>|>System-Grafik)
// - [0, 119, 0] = Call global event #119 (|>System-Grafik+Face)
// Whenever you see one of these calls, reset the portrait boolean to false. Assume no portrait until otherwise specified.
pub const COMMAND_CALL_GLOBAL_EVENT: i32 = 12330;
pub const COMMAND_TRANSACTION: i32 = 20720;
pub const COMMAND_NO_TRANSACTION: i32 = 20721;

pub const DIALOGUE_BOX_MAX_LENGTH_PORTRAIT: usize = 38;
pub const DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT: usize = 50;
