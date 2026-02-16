// The following code is concerned with generating HTML previews of all the dialogue in a map.
// It is NOT concerned with the text fields, since those (usually) aren't a concern w.r.t. dialogue box length.
// It uses direct string replacement because it does NOT need advanced features of HTML templating engines.
// -----
// NOTE: Although you *could* try and represent color in the HTML previews...
// Should you? Should you really spend all the extra effort for that tiny gain?
// The HTML dialogue preview is a tool to easily see text overflows.
// Beyond that, anything else is severely diminishing returns.

use crate::{
    dialogue::core::Dialogue,
    structs::{
        patch::{PatchDialogue, PatchText},
        Patch,
    },
    util::{
        constants::{DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT, DIALOGUE_BOX_MAX_LENGTH_PORTRAIT},
        patch_operations,
    },
};
use std::collections::HashMap;

pub const CSS_STRING: &str = include_str!("style.css");
// Variables: $MAP_NAME$ and $DIALOGUE_ENTRIES$
const HTML_PREVIEW_TEMPLATE: &str = include_str!("template.html");
// NOTE: All templates should call '.replacen("\n", "", 1)' in order to remove the leading newline!
const HTML_DIALOGUE_LIST_TEMPLATE: &str = r#"
			<div class="dialogue-list">
				<h2>$DIALOGUE_LIST_TITLE$</h2>
$DIALOGUE_LIST_ENTRIES$
			</div>"#;
const HTML_DIALOGUE_BOX_TEMPLATE: &str = r#"
				<div class="dialogue">
$DIALOGUE_BOX_ENTRIES$
				</div>"#;
const HTML_DIALOGUE_BOX_PORTRAIT_TEMPLATE: &str = r#"
				<div class="dialogue portrait">
$DIALOGUE_BOX_ENTRIES$
				</div>"#;
const HTML_DIALOGUE_BOX_LINE_TEMPLATE: &str = r#"
					<code>$DIALOGUE_BOX_LINE_TEXT$</code>"#;

#[derive(Debug)]
pub struct OverflowEntry {
    map_name: String,
    event: i32,
    page: Option<i32>,
    area: OverflowEntryType,
    html: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OverflowEntryType {
    Dialogue,
    Text,
}

pub fn generate_html_preview(
    dialogues_table: &HashMap<(i32, Option<i32>), Vec<&PatchDialogue>>,
    texts_table: &HashMap<(i32, Option<i32>), Vec<&PatchText>>,
    map_name: &String,
    character_names: &HashMap<i32, String>,
    // Extremely janky mutable reference in order to not have to parse Dialogue all over again
    overflow_list: &mut Vec<OverflowEntry>,
) -> String {
    let mut html_dialogue_groups: Vec<String> = Vec::new();

    for key in Patch::get_sorted_dialogue_keys(dialogues_table) {
        let dialogues = dialogues_table
            .get(key)
            .expect("Patch::get_sorted_dialogue_keys() didn't generate a valid key?!");
        let mut html_dialogue_boxes_list: Vec<String> = Vec::new();

        for dialogue in dialogues {
            let has_portrait = dialogue.has_portrait.unwrap_or(false);
            let ignore_overflow = dialogue.ignore_overflow.unwrap_or(false);

            // Generate the lines that would be in the resulting binaries
            let (parsed_dialogue, error_message) = Dialogue::get_split_lines(
                &dialogue.patched,
                has_portrait,
                character_names,
                dialogue.should_use_custom_line_wrapping,
            );
            // Then parse THAT in order to generate HTML preview lines
            let dialogue = Dialogue::from(
                parsed_dialogue.join("\n"),
                has_portrait,
                character_names,
                dialogue.should_use_custom_line_wrapping,
            );

            if !ignore_overflow {
                // May as well print out errors to the console as well if you're already running the check dialogue function
                if let Some(error_message) = error_message {
                    eprintln!("{error_message}- See the generated \"report.html\" for details.");
                }
            }

            let (html, has_overflow) = generate_html_dialogue_box_preview(
                &dialogue.processed_lines_pretty,
                has_portrait,
                ignore_overflow,
            );

            if has_overflow {
                let (event, page) = key;

                // The performance penalty of cloning strings only when needed is negligible
                overflow_list.push(OverflowEntry {
                    map_name: map_name.to_string(),
                    event: *event,
                    page: *page,
                    area: OverflowEntryType::Dialogue,
                    html: html.clone(),
                });
            }

            html_dialogue_boxes_list.push(html);
        }

        let html_dialogue_list_title = {
            let (event, page) = key;

            if let Some(page) = page {
                format!("Event #{event} Page #{page}")
            } else {
                format!("Event #{event}")
            }
        };
        html_dialogue_groups.push(
            HTML_DIALOGUE_LIST_TEMPLATE
                .replacen("\n", "", 1)
                .replace("$DIALOGUE_LIST_TITLE$", &html_dialogue_list_title)
                .replace(
                    "$DIALOGUE_LIST_ENTRIES$",
                    &html_dialogue_boxes_list.join("\n"),
                ),
        );
    }

    for key in Patch::get_sorted_text_keys(texts_table) {
        let texts = texts_table
            .get(key)
            .expect("Patch::get_sorted_text_keys() didn't generate a valid key?!");
        let mut html_dialogue_boxes_list: Vec<String> = Vec::new();

        for text in texts {
            let has_portrait = text.has_portrait.unwrap_or(false);
            let ignore_overflow = text.ignore_overflow.unwrap_or(false);
            let parsed_dialogue =
                patch_operations::clean_escaped_text(&text.patched, character_names);

            let (html, has_overflow) =
                generate_html_text_box_preview(&parsed_dialogue, has_portrait, ignore_overflow);

            if has_overflow {
                let (event, page) = key;

                // The performance penalty of cloning strings only when needed is negligible
                overflow_list.push(OverflowEntry {
                    map_name: map_name.to_string(),
                    event: *event,
                    page: *page,
                    area: OverflowEntryType::Text,
                    html: html.clone(),
                });
            }

            html_dialogue_boxes_list.push(html);
        }

        let html_dialogue_list_title = {
            let (event, page) = key;

            if let Some(page) = page {
                format!("Event #{event} Page #{page} (Text)")
            } else {
                format!("Event #{event} (Text)")
            }
        };
        html_dialogue_groups.push(
            HTML_DIALOGUE_LIST_TEMPLATE
                .replacen("\n", "", 1)
                .replace("$DIALOGUE_LIST_TITLE$", &html_dialogue_list_title)
                .replace(
                    "$DIALOGUE_LIST_ENTRIES$",
                    &html_dialogue_boxes_list.join("\n"),
                ),
        );
    }

    HTML_PREVIEW_TEMPLATE
        .replace("$MAP_NAME$", map_name)
        .replace("$DIALOGUE_ENTRIES$", &html_dialogue_groups.join("\n"))
}

// This generates one instance of HTML_DIALOGUE_BOX_TEMPLATE, more modular that way
// Assume by this point that you've already done necessary line wrapping and cleaned out special characters.
pub fn generate_html_dialogue_box_preview(
    cleaned_lines: &Vec<String>,
    has_portrait: bool,
    ignore_overflow: bool,
) -> (String, bool) {
    let dialogue_box_length = match has_portrait {
        true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
        false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
    };
    let mut html_lines: Vec<String> = Vec::new();
    let mut has_overflow = false;

    // CSS will automatically mark lines 5+ as red
    // You simply need to add characters beyond the 38/50 length into a <span> to mark as red
    for line in cleaned_lines {
        let length = line.chars().count();

        let html_line = if length > dialogue_box_length {
            if !ignore_overflow {
                has_overflow = true;
            }

            // Query the correct index with Unicode strings
            // https://stackoverflow.com/a/72589022
            let (index, _) = line
                .char_indices()
                .nth(dialogue_box_length)
                .expect(&format!(
                    "There should be at least {dialogue_box_length} entries for this substring!"
                ));

            let line_in_bounds = &line[0..index];
            let line_out_of_bounds = &line[index..];

            &format!("{line_in_bounds}<span>{line_out_of_bounds}</span>")
        } else {
            line
        };

        html_lines.push(
            HTML_DIALOGUE_BOX_LINE_TEMPLATE
                .replacen("\n", "", 1)
                .replace("$DIALOGUE_BOX_LINE_TEXT$", html_line),
        );
    }

    let template = if has_portrait {
        HTML_DIALOGUE_BOX_PORTRAIT_TEMPLATE
    } else {
        HTML_DIALOGUE_BOX_TEMPLATE
    };

    let html = template
        .replacen("\n", "", 1)
        .replace("$DIALOGUE_BOX_ENTRIES$", &html_lines.join("\n"));

    (html, has_overflow)
}

pub fn generate_html_text_box_preview(
    line: &String,
    has_portrait: bool,
    ignore_overflow: bool,
) -> (String, bool) {
    let dialogue_box_length = match has_portrait {
        true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
        false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
    };
    let mut has_overflow = false;

    let length = line.chars().count();

    let html_line = if length > dialogue_box_length {
        if !ignore_overflow {
            has_overflow = true;
        }

        // Query the correct index with Unicode strings
        // https://stackoverflow.com/a/72589022
        let (index, _) = line
            .char_indices()
            .nth(dialogue_box_length)
            .expect(&format!(
                "There should be at least {dialogue_box_length} entries for this substring!"
            ));

        let line_in_bounds = &line[0..index];
        let line_out_of_bounds = &line[index..];

        &format!("{line_in_bounds}<span>{line_out_of_bounds}</span>")
    } else {
        line
    };

    let template = if has_portrait {
        HTML_DIALOGUE_BOX_PORTRAIT_TEMPLATE
    } else {
        HTML_DIALOGUE_BOX_TEMPLATE
    };

    let html = template.replacen("\n", "", 1).replace(
        "$DIALOGUE_BOX_ENTRIES$",
        &HTML_DIALOGUE_BOX_LINE_TEMPLATE
            .replacen("\n", "", 1)
            .replace("$DIALOGUE_BOX_LINE_TEXT$", html_line),
    );

    (html, has_overflow)
}

pub fn generate_overflow_html_preview(overflow_list: &Vec<OverflowEntry>) -> String {
    let mut html_dialogue_groups: Vec<String> = Vec::new();
    let mut html_dialogue_boxes_list: Vec<String> = Vec::new();

    // NOTE: Due to the Vec, you'll need to keep track of last_x variables to group dialogue together.
    let mut last_map_name = String::new();
    let mut last_event = -1;
    let mut last_page: Option<i32> = None;
    let mut last_area = OverflowEntryType::Dialogue;
    let mut has_encountered_first_group = false;

    for overflow_entry in overflow_list {
        let OverflowEntry {
            map_name,
            event,
            page,
            area,
            html,
        } = overflow_entry;

        if (last_map_name != *map_name)
            || (last_event != *event)
            || (last_page != *page)
            || (last_area != *area)
        {
            if has_encountered_first_group {
                let html_dialogue_list_title = {
                    let area_str = match last_area {
                        OverflowEntryType::Dialogue => "Dialogue",
                        OverflowEntryType::Text => "Text",
                    };

                    if let Some(last_page) = last_page {
                        format!(
                            "{last_map_name}: Event #{last_event} Page #{last_page} ({area_str})"
                        )
                    } else {
                        format!("{last_map_name}: Event #{last_event} ({area_str})")
                    }
                };
                html_dialogue_groups.push(
                    HTML_DIALOGUE_LIST_TEMPLATE
                        .replacen("\n", "", 1)
                        .replace("$DIALOGUE_LIST_TITLE$", &html_dialogue_list_title)
                        .replace(
                            "$DIALOGUE_LIST_ENTRIES$",
                            &html_dialogue_boxes_list.join("\n"),
                        ),
                );
                html_dialogue_boxes_list = Vec::new();
            }

            has_encountered_first_group = true;
        }

        last_map_name = map_name.to_string();
        last_event = *event;
        last_page = *page;
        last_area = *area;

        html_dialogue_boxes_list.push(html.to_string());
    }

    HTML_PREVIEW_TEMPLATE
        .replace("$MAP_NAME$", "Overflow Report")
        .replace("$DIALOGUE_ENTRIES$", &html_dialogue_groups.join("\n"))
}
