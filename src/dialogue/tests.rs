#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::dialogue::core::*;

    fn verify_rendered_dialogue(
        text: &str,
        has_portrait: bool,
        expected_text: &str,
        expected_cleaned_text: &str,
    ) {
        let mut character_names: HashMap<i32, String> = HashMap::new();
        character_names.insert(1, "Kento".into());
        character_names.insert(2, "Cibon".into());
        character_names.insert(3, "Soko".into());

        let dialogue = Dialogue::from(text, has_portrait, &character_names, true);
        //println!("{dialogue:?}");

        let output = dialogue.processed_lines.join("\n");
        assert_eq!(output, expected_text);

        let output_pretty = dialogue.processed_lines_pretty.join("\n");
        assert_eq!(output_pretty, expected_cleaned_text);

        assert!(dialogue.check_if_out_of_bounds().is_none());
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
        let expected_text = text;
        let expected_cleaned_text = r"Ansager: Euer aktueller Rang ist \v[1886]";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_preserve_character_names() {
        let text = r"\c[1]\n[1]:\c[0] Tu das.";
        let expected_text = text;
        let expected_cleaned_text = r"Kento: Tu das.";
        verify_rendered_dialogue(text, true, expected_text, expected_cleaned_text);
    }

    #[test]
    fn should_parse_character_name_length() {
        let text = r"\c[2]\n[2]:\c[0] Nun...\. ich geh dann mal wieder hoch und seh mir weiterhin die leere Landschaft an...";
        let expected_text = r"\c[2]\n[2]:\c[0] Nun...\. ich geh dann mal wieder
hoch und seh mir weiterhin die leere
Landschaft an...";
        let expected_cleaned_text = r"Cibon: Nun... ich geh dann mal wieder
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
    fn can_read_unknown_control() {
        let text = r"\c[10]Ansager:\c[0] Test \p!!1";
        let expected_text = text;
        let expected_cleaned_text = r"Ansager: Test !!1";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }

    #[test]
    fn can_read_unknown_control_with_number() {
        let text = r"\c[10]Ansager:\c[0] Test \p[123]!!1";
        let expected_text = text;
        let expected_cleaned_text = r"Ansager: Test !!1";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }

    #[test]
    fn preserves_original_control_character() {
        let text = r"\c[10]Ansager:\C[0]\S[1]\s[6] Test \N[123]\n[456] and \V[123]\v[456]!!1";
        let expected_text = text;
        let expected_cleaned_text = r"Ansager: Test \N[123]\n[456] and \V[123]\v[456]!!1";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }

    #[test]
    #[should_panic]
    fn should_warn_on_portrait_overflow() {
        let text = r"\c[13]Red-Haired Woman\c[0]:\s[6] Seldan...\. Don't you notice...\. how we're walking in a circle?\. Don't you see...\. how each victory only brings more suffering?\.\.\^ tes";
        let dialogue = Dialogue::from(text, true, &HashMap::new(), true);
        assert!(dialogue.check_if_out_of_bounds().is_none());
    }

    #[test]
    #[should_panic]
    fn should_warn_on_non_portrait_overflow() {
        let text = r"\c[10]Funkdurchsage\c[0]: Ist Ihnen bewusst, was Sie da machen?!\. Das ist blanker SELBSTMORD!\. Selbst mit dem Vel-System werden Sie nicht im Alleingang gegen eine ganze Armee bestehen können! sample text";
        let dialogue = Dialogue::from(text, false, &HashMap::new(), true);
        assert!(dialogue.check_if_out_of_bounds().is_none());
    }

    #[test]
    #[should_panic]
    fn should_warn_on_portrait_existing_line_overflow() {
        let text = r"\c[10]Silkia\c[0]: Anyways.\. Come back
safely,\. this time the assignment certai
won't be as easy as before.";
        let dialogue = Dialogue::from(text, true, &HashMap::new(), true);
        assert!(dialogue.check_if_out_of_bounds().is_none());
    }

    #[test]
    #[should_panic]
    fn should_warn_on_non_portrait_existing_line_overflow() {
        let text = r"\c[10]Announcer:\C[0]\S[1]\s[6] Alright, here comes some sample text inn
it m8?!";
        let dialogue = Dialogue::from(text, false, &HashMap::new(), true);
        assert!(dialogue.check_if_out_of_bounds().is_none());
    }

    #[test]
    fn can_parse_abstract_numbers() {
        let text = r"\>\c[\v[81]]\n[30]\v[95]  \c[\v[82]]\n[31]\v[102]  \c[\v[83]]\n[32]\v[109]  \c[\v[84]]\n[33]\v[116]
\>\c[0]\n[34]\n[35]
\>\c[3](\n[36]\c[3]\n[37]\c[3]\n[38]\c[3]\n[39]\c[3]\n[40]\c[3]\n[41]\c[3]\n[42])";
        let expected_text = text;
        let expected_cleaned_text = r"\n[30]\v[95]  \n[31]\v[102]  \n[32]\v[109]  \n[33]\v[116]
\n[34]\n[35]
(\n[36]\n[37]\n[38]\n[39]\n[40]\n[41]\n[42])";

        // Copied and modified because it'll always be out of the character limit
        let dialogue = Dialogue::from(text, false, &HashMap::new(), true);

        let output = dialogue.processed_lines.join("\n");
        assert_eq!(output, expected_text);

        let output_pretty = dialogue.processed_lines_pretty.join("\n");
        assert_eq!(output_pretty, expected_cleaned_text);
    }

    #[test]
    fn preserves_unnecessary_trailing_newline_for_consistency() {
        let text = r"\c[1]\n[1]\c[0]: Was tut der Nachname schon 
zur Sache? 
";
        let expected_text = text;
        let expected_cleaned_text = r"Kento: Was tut der Nachname schon 
zur Sache? 
";
        verify_rendered_dialogue(text, false, expected_text, expected_cleaned_text);
    }
}
