// The following code is concerned with generating HTML previews of all the dialogue in a map.
// It is NOT concerned with the text fields, since those (usually) aren't a concern w.r.t. dialogue box length.
// It uses direct string replacement because it does NOT need advanced features of HTML templating engines.

use crate::util::constants::{
    DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT, DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
};

const HTML_PREVIEW_TEMPLATE: &str = include_str!("template.html");
const HTML_DIALOGUE_BOX_TEMPLATE: &str = r#"
		<div class="dialogue">
			<code class="check">$DIALOGUE_LINE_0$</code>
			<div class="lines">
				<code>$DIALOGUE_LINE_1$</code>
				<code>$DIALOGUE_LINE_2$</code>
				<code>$DIALOGUE_LINE_3$</code>
				<code>$DIALOGUE_LINE_4$</code>
			</div>
			<code class="overflow">$DIALOGUE_LINE_OVERFLOW$</code>
		</div>
        "#;
const NO_BREAK_SPACE: &str = "\u{A0}";

// Assume by this point that you've already done necessary line wrapping and cleaned out special characters.
pub fn generate_html_preview(
    cleaned_lines: Vec<&String>,
    has_portrait: bool,
    map_name: &String,
) -> String {
    let dialogue_box_length = match has_portrait {
        true => DIALOGUE_BOX_MAX_LENGTH_PORTRAIT,
        false => DIALOGUE_BOX_MAX_LENGTH_NON_PORTRAIT,
    };
    let mut html = HTML_PREVIEW_TEMPLATE.replace("$MAP_NAME$", map_name);
    let mut html_dialogue_box = HTML_DIALOGUE_BOX_TEMPLATE.replace(
        "$DIALOGUE_LINE_0$",
        &NO_BREAK_SPACE.repeat(dialogue_box_length),
    );
    let mut index = 1;
    let mut has_overflow = false;

    for line in cleaned_lines {
        if index <= 4 {
            // TODO: Line is OOB of dialogue box
            let text_padding_length = dialogue_box_length - line.len();
            html_dialogue_box = html_dialogue_box.replace(
                &format!("$DIALOGUE_LINE_{index}$"),
                &format!("{line}{}", NO_BREAK_SPACE.repeat(text_padding_length)),
            );
        } else {
            // TODO: 6+ lines case
            html_dialogue_box = html_dialogue_box.replace("$DIALOGUE_LINE_OVERFLOW$", line);
            has_overflow = true;
        }

        index += 1;
    }

    if !has_overflow {
        html_dialogue_box = html_dialogue_box.replace("$DIALOGUE_LINE_OVERFLOW$", "");
    }

    html = html.replace("$DIALOGUE_ENTRIES$", &html_dialogue_box);

    html
}

// This generates one instance of HTML_DIALOGUE_BOX_TEMPLATE, more modular that way
pub fn generate_html_dialogue_box_preview() {
    //
}
