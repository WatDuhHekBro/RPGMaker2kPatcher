#[cfg(test)]
mod tests {
    use crate::dialogue::core::*;

    fn verify_rendered_dialogue(
        text: &str,
        has_portrait: bool,
        expected_text: &str,
        expected_cleaned_text: &str,
    ) {
        let dialogue = Dialogue::from(text, has_portrait);

        let lines = dialogue.render_to_auto_wrapped_lines();
        let output = lines.join("\n");
        assert_eq!(output, expected_text);

        let lines_pretty = dialogue.render_to_auto_wrapped_lines_pretty();
        let output_pretty = lines_pretty.join("\n");
        assert_eq!(output_pretty, expected_cleaned_text);
    }

    #[test]
    fn preserve_existing_newlines() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\.
Glaube mir,\....es ist besser so 
für mich...\.\.\^";
        let expected_text = text;
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan...
Glaube mir,...es ist besser so 
für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_space() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_space() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. GlaubeZ mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. GlaubeZ
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... GlaubeZ
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_exclamation_mark() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube!mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube!
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube!
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_question_mark() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube?mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube?
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube?
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_period() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube.mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube.
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube.
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_comma() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube,mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube,
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube,
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_colon() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube:mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube:
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube:
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn split_line_at_final_semicolon() {
        let text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube;mir,\....es ist besser so für mich...\.\.\^";
        let expected_text = r"\c[13]Rothaarige Frau\c[0]:\s[6] S...\.Seldan...\. Glaube;
mir,\....es ist besser so für mich...\.\.\^";
        let expected_cleaned_text = r"Rothaarige Frau: S...Seldan... Glaube;
mir,...es ist besser so für mich...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_preserve_variables() {
        let text = r"\c[10]Ansager:\c[0] Euer aktueller Rang ist \v[1886]";
        let expected_text = r"\c[10]Ansager:\c[0] Euer aktueller Rang ist \v[1886]";
        let expected_cleaned_text = r"Ansager: Euer aktueller Rang ist \v[1886]";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_preserve_character_names() {
        let text = r"\c[1]\n[1]:\c[0] Tu das.";
        let expected_text = r"\c[1]\n[1]:\c[0] Tu das.";
        let expected_cleaned_text = r"\n[1]: Tu das.";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_parse_character_name_length() {
        let text = r"\c[2]\n[2]:\c[0] Nun...\. ich geh dann mal wieder hoch und seh mir weiterhin die leere Landschaft an...";
        let expected_text = r"\c[2]\n[2]:\c[0] Nun...\. ich geh dann mal wieder
hoch und seh mir weiterhin die leere
Landschaft an...";
        let expected_cleaned_text = r"\n[2]: Nun... ich geh dann mal wieder
hoch und seh mir weiterhin die leere
Landschaft an...";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_wrap_non_portrait_length() {
        let text = r"\c[10]Ansager:\c[0] Ein Hinweis, da ihr neu seid.\. Ihr habt keine Möglichkeit, eure Ausrüstung neu zusammen zu stellen oder aufzustocken und ihr werdet nicht geheilt.";
        let expected_text = r"\c[10]Ansager:\c[0] Ein Hinweis, da ihr neu seid.\. Ihr habt
keine Möglichkeit, eure Ausrüstung neu zusammen zu
stellen oder aufzustocken und ihr werdet nicht
geheilt.";
        let expected_cleaned_text = r"Ansager: Ein Hinweis, da ihr neu seid. Ihr habt
keine Möglichkeit, eure Ausrüstung neu zusammen zu
stellen oder aufzustocken und ihr werdet nicht
geheilt.";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_warn_on_portrait_overflow() {
        //
    }

    #[test]
    fn should_warn_on_non_portrait_overflow() {
        //
    }

    #[test]
    fn should_warn_on_portrait_existing_line_overflow() {
        //
    }

    #[test]
    fn should_warn_on_non_portrait_existing_line_overflow() {
        //
    }
}
