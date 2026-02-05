// The reason this error is thrown in every potential line instead of propagating upwards via "?" is because
// if you used "?" for all of the DynamicInteger::read()'s, you'll only see the error thrown in your main function.
// Not helpful at all for debugging issues.
pub const ERROR_BINRW_READ: &str = "Binary read failed!";
pub const ERROR_MAP_PAGE_NONE: &str =
    "All LcfMapUnit patches MUST have a \"page\" number! It is only optional for the LcfDataBase!";

// Dialogue-related commands
pub const COMMAND_DIALOGUE_START: i32 = 10110;
pub const COMMAND_DIALOGUE_CONTINUE: i32 = 20110;
//pub const COMMAND_CHANGE_FACE_GRAPHIC: i32 = 10130;

// Other commands
pub const COMMAND_MULTIPLE_CHOICE_PROMPT: i32 = 10140;
pub const COMMAND_MULTIPLE_CHOICE_SELECTION: i32 = 20140;
pub const COMMAND_SAVE_POINT_NAME: i32 = 10610;
