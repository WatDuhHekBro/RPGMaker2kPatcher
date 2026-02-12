// Utility structure to handle advanced dialogue operations that need a proper parse tree
// Basically replaces any `dialogue.split("\n")` calls with this custom line splitter

use crate::util::constants::{
    DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT, DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
};

#[derive(Debug)]
pub struct Dialogue {
    // Don't keep track of indexes of dialogue length, because it's redundant
    // You may as well just count fragment rendered character length
    fragments: Vec<DialogueFragment>,
    has_portrait: bool,
    //has_existing_newlines: bool,
    //is_out_of_bounds: bool,
    // Don't automatically process lines, because depending on the need, you don't need both processed versions.
    //pub processed_lines: Vec<String>,
    //pub processed_lines_pretty: Vec<String>,
}

// TODO: See whether or not you need...
// - Original text index
// - Original text index for current line
//     - Current line index
// - Display index
/*#[derive(Debug)]
pub struct DialogueFragmentWrapper {
    pub fragment: DialogueFragment,
    // A 1-index
    pub display_index: Range<i32>,
}*/

// https://rpgmaker.net/tutorials/43/
// https://www.yanfly.moe/wiki/Category:Text_Codes_(MV)
// https://www.francelettekindnessadventure.com/color-codes---rpg-maker.html
#[derive(Debug)]
pub enum DialogueFragment {
    /////////////////
    // Normal Text //
    /////////////////
    Normal(String),
    // These should be grouped together so newlines don't awkwardly split it in the middle.
    Punctuation(char),
    Space,
    Newline,
    /////////////
    // Control //
    /////////////
    Delay,        // \.
    BigDelay,     // \|
    AutoContinue, // \^
    Backslash,    // \\
    UnknownControl(char),
    ///////////////////////
    // Control w/ Number //
    ///////////////////////
    // The original character used should be readily available just in case there's a difference between uppercase and lowercase
    SetColor(char, u32),
    SetSpeed(char, u32),
    CharacterName(char, u32),
    Variable(char, u32),
    UnknownControlWithNumber(char, u32),
}

impl DialogueFragment {
    pub fn to_string(&self) -> String {
        match &self {
            // Normal Text
            DialogueFragment::Normal(fragment_text) => fragment_text.to_string(),
            DialogueFragment::Punctuation(character) => character.to_string(),
            DialogueFragment::Space => String::from(" "),
            DialogueFragment::Newline => String::new(),
            // Control
            DialogueFragment::Delay => String::from(r"\."),
            DialogueFragment::BigDelay => String::from(r"\|"),
            DialogueFragment::AutoContinue => String::from(r"\^"),
            DialogueFragment::Backslash => String::from(r"\\"),
            DialogueFragment::UnknownControl(character) => format!("\\{character}"),
            // Control w/ Number
            DialogueFragment::SetColor(character, number) => format!("\\{character}[{number}]"),
            DialogueFragment::SetSpeed(character, number) => format!("\\{character}[{number}]"),
            DialogueFragment::CharacterName(character, number) => {
                format!("\\{character}[{number}]")
            }
            DialogueFragment::Variable(character, number) => format!("\\{character}[{number}]"),
            DialogueFragment::UnknownControlWithNumber(character, number) => {
                format!("\\{character}[{number}]")
            }
        }
    }

    pub fn to_string_display(&self) -> String {
        match &self {
            // Normal Text
            DialogueFragment::Normal(fragment_text) => fragment_text.to_string(),
            DialogueFragment::Punctuation(character) => character.to_string(),
            DialogueFragment::Space => String::from(" "),
            DialogueFragment::Newline => String::new(),
            // Control
            DialogueFragment::Delay => String::new(),
            DialogueFragment::BigDelay => String::new(),
            DialogueFragment::AutoContinue => String::new(),
            DialogueFragment::Backslash => String::from(r"\\"),
            DialogueFragment::UnknownControl(_) => String::new(),
            // Control w/ Number
            DialogueFragment::SetColor(_, _) => String::new(),
            DialogueFragment::SetSpeed(_, _) => String::new(),
            DialogueFragment::CharacterName(character, number) => {
                format!("\\{character}[{number}]")
            }
            DialogueFragment::Variable(character, number) => format!("\\{character}[{number}]"),
            DialogueFragment::UnknownControlWithNumber(_, _) => String::new(),
        }
    }
}

pub enum ParsingMode {
    Normal,
    Control,
    ControlWithNumber(ParsingModeProgress),
}

#[derive(Clone, Copy)]
pub enum ParsingModeProgress {
    Start,
    Main,
}

#[derive(PartialEq)]
pub enum ControlWithNumberType {
    SetColor(char),
    SetSpeed(char),
    CharacterName(char),
    Variable(char),
    Unknown(char),
}

impl Dialogue {
    fn parse_into_fragments<S: AsRef<str>>(text: S) -> Vec<DialogueFragment> {
        let mut parsed: Vec<DialogueFragment> = Vec::new();
        let mut mode = ParsingMode::Normal;
        let mut tmp_text: String = String::new();
        let mut tmp_number: u32 = 0;
        let mut tmp_type: ControlWithNumberType = ControlWithNumberType::Unknown('?');
        let mut chars_iterator = text.as_ref().chars().peekable();

        while let Some(character) = chars_iterator.next() {
            match mode {
                ParsingMode::Normal => match character {
                    '\\' => {
                        if !tmp_text.is_empty() {
                            parsed.push(DialogueFragment::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        mode = ParsingMode::Control
                    }
                    '\n' => {
                        if !tmp_text.is_empty() {
                            parsed.push(DialogueFragment::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        parsed.push(DialogueFragment::Newline)
                    }
                    ' ' => {
                        if !tmp_text.is_empty() {
                            parsed.push(DialogueFragment::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        parsed.push(DialogueFragment::Space)
                    }
                    '.' | ',' | '?' | '!' | ':' | ';' => {
                        if !tmp_text.is_empty() {
                            parsed.push(DialogueFragment::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        parsed.push(DialogueFragment::Punctuation(character))
                    }
                    _ => {
                        tmp_text.push(character);
                    }
                },
                ParsingMode::Control => match character {
                    'c' | 'C' => {
                        tmp_type = ControlWithNumberType::SetColor(character);
                        mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Start);
                    }
                    's' | 'S' => {
                        tmp_type = ControlWithNumberType::SetSpeed(character);
                        mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Start);
                    }
                    'n' | 'N' => {
                        tmp_type = ControlWithNumberType::CharacterName(character);
                        mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Start);
                    }
                    'v' | 'V' => {
                        tmp_type = ControlWithNumberType::Variable(character);
                        mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Start);
                    }
                    '.' => {
                        parsed.push(DialogueFragment::Delay);
                        mode = ParsingMode::Normal;
                    }
                    '|' => {
                        parsed.push(DialogueFragment::BigDelay);
                        mode = ParsingMode::Normal;
                    }
                    '^' => {
                        parsed.push(DialogueFragment::AutoContinue);
                        mode = ParsingMode::Normal;
                    }
                    '\\' => {
                        parsed.push(DialogueFragment::Backslash);
                        mode = ParsingMode::Normal;
                    }
                    // You don't know if an unknown control type is of format "\x" or "\x[123]", so test both
                    _ => {
                        // You need to peek at the next character and not consume it
                        // This is so you don't have to repeat the same DialogueFragment::Normal logic
                        let next_character = chars_iterator.peek();
                        let mut is_next_character_with_number = false;

                        if let Some(next_character) = next_character {
                            if *next_character == '[' {
                                is_next_character_with_number = true;
                            }
                        }

                        if is_next_character_with_number {
                            tmp_type = ControlWithNumberType::Unknown(character);
                            mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Start);
                        } else {
                            parsed.push(DialogueFragment::UnknownControl(character));
                            mode = ParsingMode::Normal;
                        }
                    }
                },
                ParsingMode::ControlWithNumber(progress) => match progress {
                    ParsingModeProgress::Start => {
                        if character == '[' {
                            tmp_number = 0;
                            mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Main)
                        }
                        /*else if let ControlWithNumberType::Unknown(character) = tmp_type {
                            parsed.push(DialogueFragment::UnknownControl(character));
                            mode = ParsingMode::Normal;
                        }*/
                        else {
                            panic!("Invalid \\x[#] pattern.");
                        }
                    }
                    ParsingModeProgress::Main => {
                        if character.is_digit(10) {
                            let digit: u8 = character as u8 - 0x30;
                            let digit = digit as u32;

                            // 3 -> 38 ===> 3 * 10 + 8
                            tmp_number = tmp_number * 10 + digit;
                        } else if character == ']' {
                            let fragment_type = match tmp_type {
                                ControlWithNumberType::SetColor(character) => {
                                    DialogueFragment::SetColor(character, tmp_number)
                                }
                                ControlWithNumberType::SetSpeed(character) => {
                                    DialogueFragment::SetSpeed(character, tmp_number)
                                }
                                ControlWithNumberType::CharacterName(character) => {
                                    DialogueFragment::CharacterName(character, tmp_number)
                                }
                                ControlWithNumberType::Variable(character) => {
                                    DialogueFragment::Variable(character, tmp_number)
                                }
                                ControlWithNumberType::Unknown(character) => {
                                    DialogueFragment::UnknownControlWithNumber(
                                        character, tmp_number,
                                    )
                                }
                            };
                            parsed.push(fragment_type);
                            mode = ParsingMode::Normal;
                        } else {
                            panic!("Invalid \\x[#] pattern.");
                        }
                    }
                },
            }
        }

        // Do I need to check if the buffer has been flushed?
        if !tmp_text.is_empty() {
            parsed.push(DialogueFragment::Normal(tmp_text));
        }

        parsed
    }

    // If there's an existing newline anywhere in the original text, preserve those, don't auto-split
    fn get_already_split_lines_if_exists(&self, is_pretty: bool) -> Option<Vec<String>> {
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut has_existing_newlines = false;

        for fragment in &self.fragments {
            match fragment {
                DialogueFragment::Newline => {
                    lines.push(current_line);
                    current_line = String::new();
                    has_existing_newlines = true;
                }
                fragment => {
                    if is_pretty {
                        current_line.push_str(&fragment.to_string_display());
                    } else {
                        current_line.push_str(&fragment.to_string());
                    }
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if has_existing_newlines {
            Some(lines)
        } else {
            None
        }
    }

    // Only Dialogue::from() is available, not Dialogue::from_lines(),
    // because Dialogue will automatically determine if there's any existing newlines.
    pub fn from<S: AsRef<str>>(text: S, has_portrait: bool) -> Dialogue {
        Dialogue {
            fragments: Dialogue::parse_into_fragments(text),
            has_portrait,
        }
    }

    pub fn render_to_auto_wrapped_lines(&self, is_pretty: bool) -> Vec<String> {
        let existing_lines = self.get_already_split_lines_if_exists(is_pretty);
        if let Some(existing_lines) = existing_lines {
            return existing_lines;
        }

        let line_length_limit = match self.has_portrait {
            true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
            false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
        };

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        // You need to keep track of the actual display length separately if it's the raw string
        let mut current_line_displayed_length: usize = 0;

        for fragment in &self.fragments {
            // All line wrap operations go off the assumption of the displayed string
            let new_fragment = fragment.to_string_display();
            // NOTE: You cannot use "new_fragment.len()" because it counts bytes, not actual length!
            // NOTE: "Möglichkeit" should be counted as 11 characters but is counted as 12 characters.
            let new_fragment_displayed_length = new_fragment.chars().count();

            // But whether or not to actually keep it is up to the specific setting
            let actual_fragment = if is_pretty {
                new_fragment
            } else {
                fragment.to_string()
            };

            // Append to current line or push to new line depending on
            // if a new fragment will exceed the current length
            if current_line_displayed_length + new_fragment_displayed_length > line_length_limit {
                // Be sure to clean up any spaces at the end
                lines.push(current_line.trim_end().to_string());

                // Make sure to carry over the current fragment text to the next line and reset the counter
                // ...unless it's a space (do not start the newline with a space)
                current_line = {
                    if actual_fragment != " " {
                        actual_fragment
                    } else {
                        String::new()
                    }
                };
                // Note that because of line rollover, the displayed length is NOT always zero
                // It is whatever the current_line is
                current_line_displayed_length = new_fragment_displayed_length;
            } else {
                current_line.push_str(&actual_fragment);
                current_line_displayed_length += new_fragment_displayed_length;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /*pub fn render_to_auto_wrapped_lines_pretty(&self) -> Vec<String> {
        let existing_lines = self.get_already_split_lines_if_exists();
        if let Some(existing_lines) = existing_lines {
            return existing_lines;
        }

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        let line_length_limit = match self.has_portrait {
            true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
            false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
        };

        for fragment in &self.fragments {
            // Append to current line or push to new line depending on
            // if a new fragment will exceed the current length
            let current_line_length = current_line.len();

            let new_fragment = fragment.to_string_pretty();
            let new_fragment_length = new_fragment.len();

            // Push new line if exceeds bounds
            if current_line_length + new_fragment_length > line_length_limit {
                lines.push(current_line);
                current_line = String::new();
            }
            // Otherwise push to current line
            else {
                current_line.push_str(&new_fragment);
            }
        }

        lines
    }*/
}
