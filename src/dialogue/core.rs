// Utility structure to handle advanced dialogue operations that need a proper parse tree
// Basically replaces any `dialogue.split("\n")` calls with this custom line splitter

use std::ops::Range;

#[derive(Debug)]
pub struct Dialogue {
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
#[derive(Debug)]
pub struct DialogueFragmentWrapper {
    pub fragment: DialogueFragment,
    // A 1-index
    pub display_index: Range<i32>,
}

// https://rpgmaker.net/tutorials/43/
// https://www.yanfly.moe/wiki/Category:Text_Codes_(MV)
// https://www.francelettekindnessadventure.com/color-codes---rpg-maker.html
#[derive(Debug)]
pub enum DialogueFragment {
    Normal(String),
    // These should be grouped together so newlines don't awkwardly split it in the middle.
    Punctuation(String),
    Space,
    Newline,
    SetColor(u8),
    SetSpeed(u8),
    Delay,        // \.
    BigDelay,     // \|
    AutoContinue, // \^
    Backslash,    // \\
    UnknownControl(String),
}

pub enum ParsingMode {
    Normal,
    Control,
    SetColor(ParsingModeProgress),
    SetSpeed(ParsingModeProgress),
}

#[derive(Clone, Copy)]
pub enum ParsingModeProgress {
    Start,
    Main,
}

impl Dialogue {
    fn parse_into_fragments<S: AsRef<str>>(text: S) -> Vec<DialogueFragment> {
        let mut parsed: Vec<DialogueFragment> = Vec::new();
        let mut mode = ParsingMode::Normal;
        let mut tmp_text: String = String::new();
        let mut tmp_number: u8 = 0;

        for character in text.as_ref().chars() {
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
                    _ => {
                        tmp_text.push(character);
                    }
                },
                ParsingMode::Control => match character {
                    'c' => mode = ParsingMode::SetColor(ParsingModeProgress::Start),
                    's' => mode = ParsingMode::SetSpeed(ParsingModeProgress::Start),
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
                    _ => panic!("Unknown control character: {character}"),
                },
                ParsingMode::SetColor(progress) => match progress {
                    ParsingModeProgress::Start => {
                        if character == '[' {
                            tmp_number = 0;
                            mode = ParsingMode::SetColor(ParsingModeProgress::Main)
                        } else {
                            panic!("Invalid \\c[#] pattern.");
                        }
                    }
                    ParsingModeProgress::Main => {
                        if character.is_digit(10) {
                            let digit = character as u8 - 0x30;

                            // 3 -> 38 ===> 3 * 10 + 8
                            tmp_number = tmp_number * 10 + digit;
                        } else if character == ']' {
                            parsed.push(DialogueFragment::SetColor(tmp_number));
                            mode = ParsingMode::Normal;
                        } else {
                            panic!("Invalid \\c[#] pattern.");
                        }
                    }
                },
                ParsingMode::SetSpeed(progress) => match progress {
                    ParsingModeProgress::Start => {
                        if character == '[' {
                            tmp_number = 0;
                            mode = ParsingMode::SetSpeed(ParsingModeProgress::Main)
                        } else {
                            panic!("Invalid \\s[#] pattern.");
                        }
                    }
                    ParsingModeProgress::Main => {
                        if character.is_digit(10) {
                            let digit = character as u8 - 0x30;

                            // 3 --> 3 * 10 + 8
                            tmp_number = tmp_number * 10 + digit;
                        } else if character == ']' {
                            parsed.push(DialogueFragment::SetSpeed(tmp_number));
                            mode = ParsingMode::Normal;
                        } else {
                            panic!("Invalid \\s[#] pattern.");
                        }
                    }
                },
            }
        }

        // Do I need to check if the buffer has been flushed?
        //if !tmp_text.is_empty()

        parsed
    }

    // Only Dialogue::from() is available, not Dialogue::from_lines(),
    // because Dialogue will automatically determine if there's any existing newlines.
    pub fn from<S: AsRef<str>>(text: S, has_portrait: bool) -> Dialogue {
        Dialogue {
            fragments: Dialogue::parse_into_fragments(text),
            has_portrait,
        }
    }

    pub fn render_to_auto_wrapped_lines(&self) -> Vec<String> {
        vec![]
    }

    pub fn render_to_auto_wrapped_lines_pretty(&self) -> Vec<String> {
        let mut text = String::new();

        for fragment in &self.fragments {
            match fragment {
                DialogueFragment::Normal(fragment_text) => {
                    text.push_str(fragment_text);
                }
                DialogueFragment::Space => {
                    text.push(' ');
                }
                DialogueFragment::Newline => {
                    text.push('\n');
                }
                DialogueFragment::Backslash => {
                    text.push('\\');
                }
                _ => (),
            }
        }

        //text
        vec![]
    }
}
