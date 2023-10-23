use crossterm::{
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::{stdout, Result, Stdout};

pub fn main() -> Result<()> {
    stdout()
        .execute(SetForegroundColor(Color::White))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print("Styled text here.   "))?
        .execute(ResetColor)?;
    stdout()
        .execute(SetForegroundColor(Color::White))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(Print("Styled text here.   \n"))?
        .execute(ResetColor)?;

    do_dialogue_testing();

    Ok(())
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct DataMap {
    test1: String,
    test2: String,
}

fn do_dialogue_testing() {
    let map = toml::from_str::<DataMap>(include_str!("sample.toml")).unwrap();
    let textbox = Dialogue::from(&map.test2);
    println!(
        "{textbox:?}\n\n{}\n\n{}",
        textbox.render_plain(),
        toml::to_string(&map).unwrap()
    );
}

// https://rpgmaker.net/tutorials/43/
#[derive(Debug)]
enum AbstractCharacter {
    Normal(String),
    Space,
    Newline,
    SetColor(u8),
    SetSpeed(u8),
    Delay,        // \.
    BigDelay,     // \|
    AutoContinue, // \^
    Backslash,    // \\
                  //UnknownControl(String),
}

enum ParsingMode {
    Normal,
    Control,
    SetColor(ParsingModeProgress),
    SetSpeed(ParsingModeProgress),
}

#[derive(Clone, Copy)]
enum ParsingModeProgress {
    Start,
    Main,
}

#[derive(Debug)]
struct Dialogue {
    text: Vec<AbstractCharacter>,
}

impl Dialogue {
    fn parse<S: AsRef<str>>(text: S) -> Vec<AbstractCharacter> {
        let mut parsed = Vec::new();
        let mut mode = ParsingMode::Normal;
        let mut tmp_text: String = String::new();
        let mut tmp_number: u8 = 0;

        for character in text.as_ref().chars() {
            match mode {
                ParsingMode::Normal => match character {
                    '\\' => {
                        if !tmp_text.is_empty() {
                            parsed.push(AbstractCharacter::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        mode = ParsingMode::Control
                    }
                    '\n' => {
                        if !tmp_text.is_empty() {
                            parsed.push(AbstractCharacter::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        parsed.push(AbstractCharacter::Newline)
                    }
                    ' ' => {
                        if !tmp_text.is_empty() {
                            parsed.push(AbstractCharacter::Normal(tmp_text));
                            tmp_text = String::new();
                        }

                        parsed.push(AbstractCharacter::Space)
                    }
                    _ => {
                        tmp_text.push(character);
                    }
                },
                ParsingMode::Control => match character {
                    'c' => mode = ParsingMode::SetColor(ParsingModeProgress::Start),
                    's' => mode = ParsingMode::SetSpeed(ParsingModeProgress::Start),
                    '.' => {
                        parsed.push(AbstractCharacter::Delay);
                        mode = ParsingMode::Normal;
                    }
                    '|' => {
                        parsed.push(AbstractCharacter::BigDelay);
                        mode = ParsingMode::Normal;
                    }
                    '^' => {
                        parsed.push(AbstractCharacter::AutoContinue);
                        mode = ParsingMode::Normal;
                    }
                    '\\' => {
                        parsed.push(AbstractCharacter::Backslash);
                        mode = ParsingMode::Normal;
                    }
                    other => panic!("Unknown control character: {other}"),
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
                            parsed.push(AbstractCharacter::SetColor(tmp_number));
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
                            parsed.push(AbstractCharacter::SetSpeed(tmp_number));
                            mode = ParsingMode::Normal;
                        } else {
                            panic!("Invalid \\s[#] pattern.");
                        }
                    }
                },
            }
        }

        parsed
    }

    fn from<S: AsRef<str>>(text: S) -> Dialogue {
        Dialogue {
            text: Dialogue::parse(text),
        }
    }

    /*fn from_lines(lines: &Vec<String>) -> Dialogue {
        Dialogue {
            text: Dialogue::parse(lines.join("\n")),
        }
    }*/

    fn render_plain(&self) -> String {
        let mut text = String::new();

        for character in &self.text {
            match character {
                AbstractCharacter::Normal(fragment) => {
                    text.push_str(fragment);
                }
                AbstractCharacter::Space => {
                    text.push(' ');
                }
                AbstractCharacter::Newline => {
                    text.push('\n');
                }
                AbstractCharacter::Backslash => {
                    text.push('\\');
                }
                _ => (),
            }
        }

        text
    }

    fn print_pretty(&self, stdout: Stdout) -> Result<()> {
        Ok(())
    }
}

/*
            match character {
                AbstractCharacter::Normal(character) => {
                    text.push(character);
                }
                AbstractCharacter::Newline => {
                    text.push('\n');
                }
                AbstractCharacter::SetColor(_color) => (),
                AbstractCharacter::SetSpeed(_speed) => (),
                AbstractCharacter::Delay,
                AbstractCharacter::BigDelay,
                AbstractCharacter::AutoContinue,
                AbstractCharacter::Backslash,
            }
*/
