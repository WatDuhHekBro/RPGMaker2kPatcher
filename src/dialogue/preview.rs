// The following code is concerned with generating HTML previews of all the dialogue in a map.
// It is NOT concerned with the text fields, since those (usually) aren't a concern w.r.t. dialogue box length.
// It uses direct string replacement because it does NOT need advanced features of HTML templating engines.

use crate::{
    dialogue::core::Dialogue,
    structs::{
        patch::{PatchDialogue, PatchText},
        Patch,
    },
    util::constants::{DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT, DIALOGUE_BOX_MAX_LENGTH_PORTRAIT},
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

pub fn generate_html_preview(
    dialogues_table: &HashMap<(i32, Option<i32>), Vec<&PatchDialogue>>,
    texts_table: &HashMap<(i32, Option<i32>), Vec<&PatchText>>,
    map_name: &String,
    character_names: &HashMap<i32, String>,
) -> String {
    let mut html_dialogue_groups: Vec<String> = Vec::new();

    for key in Patch::get_sorted_dialogue_keys(dialogues_table) {
        let dialogues = dialogues_table
            .get(key)
            .expect("Patch::get_sorted_dialogue_keys() didn't generate a valid key?!");
        let mut html_dialogue_boxes_list: Vec<String> = Vec::new();

        for dialogue in dialogues {
            let has_portrait = dialogue.has_portrait.unwrap_or(false);
            let parsed_dialogue = Dialogue::from(&dialogue.patched, has_portrait, character_names);

            html_dialogue_boxes_list.push(generate_html_dialogue_box_preview(
                &parsed_dialogue.processed_lines_pretty,
                has_portrait,
            ));
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
            let parsed_dialogue = Dialogue::from(&text.patched, has_portrait, character_names);

            html_dialogue_boxes_list.push(generate_html_dialogue_box_preview(
                &parsed_dialogue.processed_lines_pretty,
                has_portrait,
            ));
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
) -> String {
    let dialogue_box_length = match has_portrait {
        true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
        false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
    };
    let mut html_lines: Vec<String> = Vec::new();

    // CSS will automatically mark lines 5+ as red
    // You simply need to add characters beyond the 38/50 length into a <span> to mark as red
    for line in cleaned_lines {
        let length = line.chars().count();

        let html_line = if length > dialogue_box_length {
            let line_in_bounds = &line[0..dialogue_box_length];
            let line_out_of_bounds = &line[dialogue_box_length..];
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

    template
        .replacen("\n", "", 1)
        .replace("$DIALOGUE_BOX_ENTRIES$", &html_lines.join("\n"))
}
