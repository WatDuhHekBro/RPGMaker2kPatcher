// Utility structure to handle advanced dialogue operations that need a proper parse tree
// Basically replaces any `dialogue.split("\n")` calls with this custom line splitter
// -----
// Default patch files *should* be identical to the original binaries, because
// auto line splitting/wrapping is something you need to opt-in to by putting
// all the text onto one line.
// -----
// Because of unforeseen roadblocks, the parser does not truly parse one character at a time.
// The special case of "\c[\v[123]]" iterates past multiple characters in one round.
// -----
// No idea how to split up the parsing function. It has an ungodly amount of nesting.

use crate::util::constants::{
    DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT, DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug)]
pub struct Dialogue {
    pub has_portrait: bool,
    // The reason to automatically process both lines is because
    // processing both together is O(n) instead of 2 * O(n).
    // Plus, this is its primary function anyway, being essentially just a fancy line splitter.
    pub processed_lines: Vec<String>,
    pub processed_lines_pretty: Vec<String>,
}

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
    SetColor(char, AbstractNumber),
    SetSpeed(char, AbstractNumber),
    CharacterName(char, AbstractNumber),
    Variable(char, AbstractNumber),
    // preview.js: ['c', 'i', 'n', 'p', 's', 'v']
    UnknownControlWithNumber(char, AbstractNumber),
}

impl DialogueFragment {
    pub fn render(&self) -> String {
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
            DialogueFragment::UnknownControl(c) => format!("\\{c}"),
            // Control w/ Number
            DialogueFragment::SetColor(c, number) => format!("\\{c}[{number}]"),
            DialogueFragment::SetSpeed(c, number) => format!("\\{c}[{number}]"),
            DialogueFragment::CharacterName(c, number) => {
                format!("\\{c}[{number}]")
            }
            DialogueFragment::Variable(c, number) => format!("\\{c}[{number}]"),
            DialogueFragment::UnknownControlWithNumber(c, number) => {
                format!("\\{c}[{number}]")
            }
        }
    }

    pub fn render_display(&self, character_name: Option<&String>) -> String {
        match &self {
            // Normal Text
            DialogueFragment::Normal(fragment_text) => fragment_text.to_string(),
            DialogueFragment::Punctuation(c) => c.to_string(),
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
            DialogueFragment::CharacterName(c, number) => {
                // Convert the call to a character name if it exists
                // Otherwise, leave it as-is
                if let Some(character_name) = character_name {
                    character_name.to_string()
                } else {
                    format!("\\{c}[{number}]")
                }
            }
            DialogueFragment::Variable(c, number) => format!("\\{c}[{number}]"),
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
    // Assumption: Only 1 level of nesting
    Nested(ParsingModeProgressNested),
}

#[derive(Clone, Copy)]
pub enum ParsingModeProgressNested {
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

// The character name can be nested with a variable actually
// "\c[\v[123]]"
// So you need to create an abstract number case
// I assume \v[#] is a number
#[derive(Debug)]
pub enum AbstractNumber {
    // "\c[123]"
    Normal(i32),
    // "\c[\v[123]]"
    Variable(i32),
}

impl AbstractNumber {
    fn get_number(&self) -> &i32 {
        match self {
            AbstractNumber::Normal(number) => number,
            AbstractNumber::Variable(number) => number,
        }
    }
}

impl Display for AbstractNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            AbstractNumber::Normal(number) => number.to_string(),
            AbstractNumber::Variable(number) => format!("\\v[{number}]"),
        };
        write!(formatter, "{output}")
    }
}

impl Dialogue {
    // Only Dialogue::from() is available, not Dialogue::from_lines(), because
    // the whole point of Dialogue is to perform the custom line splitting.
    pub fn from<S: AsRef<str> + Display>(
        text: S,
        has_portrait: bool,
        character_names: &HashMap<i32, String>,
    ) -> Dialogue {
        let fragments = Dialogue::parse_into_fragments(text);
        //println!("{fragments:?}");

        let (processed_lines, processed_lines_pretty) =
            Dialogue::render_to_auto_wrapped_lines(&fragments, has_portrait, character_names);

        Dialogue {
            has_portrait,
            processed_lines,
            processed_lines_pretty,
        }
    }

    fn parse_into_fragments<S: AsRef<str> + Display>(text: S) -> Vec<DialogueFragment> {
        let mut parsed: Vec<DialogueFragment> = Vec::new();
        let mut mode = ParsingMode::Normal;
        let mut tmp_text: String = String::new();
        let mut tmp_number: i32 = 0;
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
                        // "\c[\v[123]]"
                        //    ^
                        if character == '[' {
                            tmp_number = 0;
                            let mut is_nested = false;

                            // "\c[\v[123]]"
                            //     ^
                            let next_character = chars_iterator.peek();

                            if let Some(next_character) = next_character {
                                if *next_character == '\\' {
                                    is_nested = true;
                                    chars_iterator.next();
                                }
                            }

                            if is_nested {
                                // "\c[\v[123]]"
                                //      ^
                                let next_character = chars_iterator.peek();

                                if let Some(next_character) = next_character {
                                    if *next_character == 'v' {
                                        chars_iterator.next();
                                    } else {
                                        panic!("Invalid \\x[\\v[#]] pattern for:\n{text}");
                                    }
                                }

                                mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Nested(
                                    ParsingModeProgressNested::Start,
                                ))
                            } else {
                                mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Main)
                            }
                        } else {
                            panic!("Invalid \\x[#] pattern for:\n{text}");
                        }
                    }
                    ParsingModeProgress::Main => {
                        if character.is_digit(10) {
                            let digit: u8 = character as u8 - 0x30;
                            let digit = digit as i32;

                            // 3 -> 38 ===> 3 * 10 + 8
                            tmp_number = tmp_number * 10 + digit;
                        } else if character == ']' {
                            let fragment_type = match tmp_type {
                                ControlWithNumberType::SetColor(character) => {
                                    DialogueFragment::SetColor(
                                        character,
                                        AbstractNumber::Normal(tmp_number),
                                    )
                                }
                                ControlWithNumberType::SetSpeed(character) => {
                                    DialogueFragment::SetSpeed(
                                        character,
                                        AbstractNumber::Normal(tmp_number),
                                    )
                                }
                                ControlWithNumberType::CharacterName(character) => {
                                    DialogueFragment::CharacterName(
                                        character,
                                        AbstractNumber::Normal(tmp_number),
                                    )
                                }
                                ControlWithNumberType::Variable(character) => {
                                    DialogueFragment::Variable(
                                        character,
                                        AbstractNumber::Normal(tmp_number),
                                    )
                                }
                                ControlWithNumberType::Unknown(character) => {
                                    DialogueFragment::UnknownControlWithNumber(
                                        character,
                                        AbstractNumber::Normal(tmp_number),
                                    )
                                }
                            };
                            parsed.push(fragment_type);
                            mode = ParsingMode::Normal;
                        } else {
                            panic!("Invalid \\x[#] pattern for:\n{text}");
                        }
                    }
                    ParsingModeProgress::Nested(nested_progress) => match nested_progress {
                        ParsingModeProgressNested::Start => {
                            if character == '[' {
                                tmp_number = 0;
                                mode = ParsingMode::ControlWithNumber(ParsingModeProgress::Nested(
                                    ParsingModeProgressNested::Main,
                                ))
                            } else {
                                panic!("Invalid \\x[#] pattern for:\n{text}");
                            }
                        }
                        ParsingModeProgressNested::Main => {
                            if character.is_digit(10) {
                                let digit: u8 = character as u8 - 0x30;
                                let digit = digit as i32;

                                // 3 -> 38 ===> 3 * 10 + 8
                                tmp_number = tmp_number * 10 + digit;
                            } else if character == ']' {
                                let fragment_type = match tmp_type {
                                    ControlWithNumberType::SetColor(character) => {
                                        DialogueFragment::SetColor(
                                            character,
                                            AbstractNumber::Variable(tmp_number),
                                        )
                                    }
                                    ControlWithNumberType::SetSpeed(character) => {
                                        DialogueFragment::SetSpeed(
                                            character,
                                            AbstractNumber::Variable(tmp_number),
                                        )
                                    }
                                    ControlWithNumberType::CharacterName(character) => {
                                        DialogueFragment::CharacterName(
                                            character,
                                            AbstractNumber::Variable(tmp_number),
                                        )
                                    }
                                    ControlWithNumberType::Variable(character) => {
                                        DialogueFragment::Variable(
                                            character,
                                            AbstractNumber::Variable(tmp_number),
                                        )
                                    }
                                    ControlWithNumberType::Unknown(character) => {
                                        DialogueFragment::UnknownControlWithNumber(
                                            character,
                                            AbstractNumber::Variable(tmp_number),
                                        )
                                    }
                                };
                                parsed.push(fragment_type);
                                mode = ParsingMode::Normal;

                                // "\c[\v[123]]"
                                //           ^
                                let next_character = chars_iterator.peek();

                                if let Some(next_character) = next_character {
                                    if *next_character == ']' {
                                        chars_iterator.next();
                                    } else {
                                        panic!("Invalid \\x[\\v[#]] pattern for:\n{text}");
                                    }
                                }
                            } else {
                                panic!("Invalid \\x[#] pattern for:\n{text}");
                            }
                        }
                    },
                },
            }
        }

        if !tmp_text.is_empty() {
            parsed.push(DialogueFragment::Normal(tmp_text));
        }

        parsed
    }

    // If there's an existing newline anywhere in the original text, preserve those, don't auto-split
    fn get_already_split_lines_if_exists(
        fragments: &Vec<DialogueFragment>,
        character_names: &HashMap<i32, String>,
    ) -> Option<(Vec<String>, Vec<String>)> {
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        let mut lines_pretty: Vec<String> = Vec::new();
        let mut current_line_pretty = String::new();

        let mut has_existing_newlines = false;

        for fragment in fragments {
            match fragment {
                DialogueFragment::Newline => {
                    lines.push(current_line);
                    current_line = String::new();

                    lines_pretty.push(current_line_pretty);
                    current_line_pretty = String::new();

                    has_existing_newlines = true;
                }
                DialogueFragment::CharacterName(_, character_name_id) => {
                    current_line_pretty.push_str(
                        &fragment
                            .render_display(character_names.get(character_name_id.get_number())),
                    );
                    current_line.push_str(&fragment.render());
                }
                fragment => {
                    current_line_pretty.push_str(&fragment.render_display(None));
                    current_line.push_str(&fragment.render());
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
        if !current_line_pretty.is_empty() {
            lines_pretty.push(current_line_pretty);
        }

        if has_existing_newlines {
            Some((lines, lines_pretty))
        } else {
            None
        }
    }

    pub fn render_to_auto_wrapped_lines(
        fragments: &Vec<DialogueFragment>,
        has_portrait: bool,
        character_names: &HashMap<i32, String>,
    ) -> (Vec<String>, Vec<String>) {
        let line_length_limit = match has_portrait {
            true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
            false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
        };

        let existing_lines =
            Dialogue::get_already_split_lines_if_exists(fragments, character_names);
        if let Some(existing_lines) = existing_lines {
            return existing_lines;
        }

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        let mut lines_pretty: Vec<String> = Vec::new();
        let mut current_line_pretty = String::new();

        // You need to keep track of the actual display length separately if it's the raw string
        let mut current_line_displayed_length: usize = 0;

        for fragment in fragments {
            // All line wrap operations go off the assumption of the displayed string
            let fragment_text_pretty =
                if let DialogueFragment::CharacterName(_, character_name_id) = fragment {
                    fragment.render_display(character_names.get(character_name_id.get_number()))
                } else {
                    fragment.render_display(None)
                };

            // NOTE: You cannot use "new_fragment.len()" because it counts bytes, not actual length!
            // NOTE: "Möglichkeit" should be counted as 11 characters but is counted as 12 characters.
            let fragment_text_displayed_length = fragment_text_pretty.chars().count();

            // But whether or not to actually keep it is up to the specific setting
            let fragment_text = fragment.render();

            // Append to current line or push to new line depending on
            // if a new fragment will exceed the current length
            if current_line_displayed_length + fragment_text_displayed_length > line_length_limit {
                // Be sure to clean up any spaces at the end
                lines.push(current_line.trim_end().to_string());
                lines_pretty.push(current_line_pretty.trim_end().to_string());

                // Make sure to carry over the current fragment text to the next line and reset the counter
                // ...unless it's a space (do not start the newline with a space)
                current_line = {
                    if fragment_text != " " {
                        fragment_text
                    } else {
                        String::new()
                    }
                };
                current_line_pretty = {
                    if fragment_text_pretty != " " {
                        fragment_text_pretty
                    } else {
                        String::new()
                    }
                };
                // Note that because of line rollover, the displayed length is NOT always zero
                // It is whatever the current_line is
                current_line_displayed_length = fragment_text_displayed_length;
            } else {
                current_line.push_str(&fragment_text);
                current_line_pretty.push_str(&fragment_text_pretty);

                current_line_displayed_length += fragment_text_displayed_length;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
        if !current_line_pretty.is_empty() {
            lines_pretty.push(current_line_pretty);
        }

        (lines, lines_pretty)
    }

    // This is not a function that should always be run (only for bulk warnings),
    // so only run it when necessary.
    pub fn check_if_out_of_bounds(&self) -> Option<String> {
        let line_length_limit = match self.has_portrait {
            true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
            false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
        };

        let lines = &self.processed_lines_pretty;

        let mut error_message = String::from("\n[-----]\n");
        error_message.push_str(&lines.join("\n"));
        error_message.push_str("\n[-----]\n");

        let mut is_out_of_bounds = false;
        let mut line_index = 1;

        if lines.len() > 4 {
            error_message.push_str("- ERROR: Contains more than 4 lines!\n");
            is_out_of_bounds = true;
        }

        for line in lines {
            // Don't warn about any more lines than the initial 4, it'd be unnecessary
            if line_index > 4 {
                break;
            }

            if line.chars().count() > line_length_limit {
                error_message.push_str(&format!("- ERROR: Line #{line_index} contains more than {line_length_limit} characters!\n"));
                is_out_of_bounds = true;
            }

            line_index += 1;
        }

        if is_out_of_bounds {
            Some(error_message)
        } else {
            None
        }
    }
}
