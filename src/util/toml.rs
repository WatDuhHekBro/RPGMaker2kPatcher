use crate::{
    structs::{
        database::{
            LcfDataBaseCharacterHeader, LcfDataBaseConditionsHeader, LcfDataBaseDoubleTextHeader,
            LcfDataBaseGlobalEventHeader, LcfDataBaseHeader, LcfDataBaseSingleTextHeader,
        },
        map::*,
        maptree::LcfMapTreeMapHeader,
        patch::{PatchDatabaseVocabulary, PatchDialogue, PatchSpliceCommands, PatchText},
        LcfCommand, LcfDataBase, LcfMapTree, ListEntryHeaderGeneric, Patch,
    },
    types::PascalString,
};

// NOTE: Because of the way TOML treats string literals (single quotes), you cannot escape anything at all,
// neither backslashes nor single quotes. So in order to be safe, you must use the multiline literals
// (triple single quotes)! That'll take care of any problems with apostrophes.

const DISCLAIMER_TEXT: &str = "# This binary representation has been serialized for easier debugging/patching.\n# It will not be used by the program to actually patch anything, so feel free to delete this.\n\n";
const HEADER_SIGNPOST_START: &str = "#############\n# Header ";
const HEADER_SIGNPOST_END: &str = " #\n#############\n\n";

pub fn generate_toml_map(map: &LcfMapUnit) -> String {
    let mut output = String::from(DISCLAIMER_TEXT);

    ////////////
    // Header //
    ////////////
    let mut output_header = String::from("[header]\ntype = '''LcfMapUnit'''\n");
    let mut output_events = String::new();

    // This disgusting nesting could probably be done more elegantly... but oh well.
    for header in &**map {
        match header {
            LcfMapUnitHeader::Panorama(name) => {
                output_header.push_str(&format!("32 = '''{name}'''\n"))
            }
            LcfMapUnitHeader::Generic(ListEntryHeaderGeneric { id, value }) => {
                output_header.push_str(&format!("{id} = {value}\n"));
            }
            LcfMapUnitHeader::Events(events) => {
                for event in &***events {
                    let mut output_current_event = format!("[event.{}]\n", event.id);
                    let mut output_pages = String::new();

                    for event_header in &*event.headers {
                        match event_header {
                            LcfMapUnitEventHeader::Name(name) => {
                                output_current_event.push_str(&format!("1 = '''{name}'''\n"))
                            }
                            LcfMapUnitEventHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => output_current_event.push_str(&format!("{id} = {value}\n")),
                            LcfMapUnitEventHeader::Pages(pages) => {
                                for page in &***pages {
                                    let mut output_current_page =
                                        format!("[event.{}.page.{}]\n", event.id, page.id);

                                    for page_header in &*page.headers {
                                        match page_header {
                                            LcfMapUnitPageHeader::Name(name) => output_current_page
                                                .push_str(&format!("21 = '''{name}'''\n")),
                                            LcfMapUnitPageHeader::Generic(
                                                ListEntryHeaderGeneric { id, value },
                                            ) => output_current_page
                                                .push_str(&format!("{id} = {value}\n")),
                                            LcfMapUnitPageHeader::Commands(commands) => {
                                                let commands = &**commands;

                                                if commands.is_empty() {
                                                    output_current_page
                                                        .push_str(&format!("commands = []\n"));
                                                } else {
                                                    output_current_page
                                                        .push_str(&format!("commands = [\n"));
                                                    let mut index = 0;

                                                    for LcfCommand {
                                                        code,
                                                        indent,
                                                        text,
                                                        parameters,
                                                    } in &**commands
                                                    {
                                                        output_current_page.push_str(&format!("\t[{code}, {indent}, '''{text}''', {parameters}], #{index}\n"));
                                                        index += 1;
                                                    }

                                                    output_current_page.push_str(&format!("]\n"));
                                                }
                                            }
                                        }
                                    }

                                    output_current_page.push_str(&format!("\n"));
                                    output_pages.push_str(&output_current_page);
                                }
                            }
                        }
                    }

                    output_current_event.push_str("\n");
                    output_current_event.push_str(&output_pages);
                    output_events.push_str(&output_current_event);
                    output_events.push_str("\n\n");
                }
            }
        }
    }

    // Merge
    output.push_str(&output_header);
    output.push_str("\n\n\n");
    output.push_str(&output_events);

    // Cleanup
    let mut output = output.trim_end().to_string();
    output.push_str("\n");

    output
}

pub fn generate_toml_database(database: &LcfDataBase) -> String {
    let mut output = String::from(DISCLAIMER_TEXT);
    output.push_str("[header]\ntype = '''LcfDataBase'''\n\n");

    // Behold! The disgusting nesting & copy pasting.
    // Maybe I should learn how to create macros...
    for header in &**database {
        match header {
            LcfDataBaseHeader::Characters(characters) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("11");
                output.push_str(HEADER_SIGNPOST_END);

                for character in &***characters {
                    output.push_str(&format!("[character.{}]\n", *character.id));

                    for header in &*character.headers {
                        let line = match header {
                            LcfDataBaseCharacterHeader::Name1(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseCharacterHeader::Name2(PascalString(text)) => {
                                format!("2 = '''{text}'''\n")
                            }
                            LcfDataBaseCharacterHeader::Name3(PascalString(text)) => {
                                format!("3 = '''{text}'''\n")
                            }
                            LcfDataBaseCharacterHeader::Face(PascalString(text)) => {
                                format!("15 = '''{text}'''\n")
                            }
                            LcfDataBaseCharacterHeader::Type(PascalString(text)) => {
                                format!("67 = '''{text}'''\n")
                            }
                            LcfDataBaseCharacterHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Skills(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("12");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[skill.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Items(lcf_double_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("13");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_double_text {
                    output.push_str(&format!("[item.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseDoubleTextHeader::TextA(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseDoubleTextHeader::TextB(PascalString(text)) => {
                                format!("2 = '''{text}'''\n")
                            }
                            LcfDataBaseDoubleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Enemies(lcf_double_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("14");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_double_text {
                    output.push_str(&format!("[enemy.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseDoubleTextHeader::TextA(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseDoubleTextHeader::TextB(PascalString(text)) => {
                                format!("2 = '''{text}'''\n")
                            }
                            LcfDataBaseDoubleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::EnemyGroups(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("15");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[enemy-group.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Terrain(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("16");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[terrain.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Attributes(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("17");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[attribute.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Conditions(conditions) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("18");
                output.push_str(HEADER_SIGNPOST_END);

                for condition in &***conditions {
                    output.push_str(&format!("[condition.{}]\n", *condition.id));

                    for header in &*condition.headers {
                        let line = match header {
                            LcfDataBaseConditionsHeader::Name(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseConditionsHeader::Start1(PascalString(text)) => {
                                format!("51 = '''{text}'''\n")
                            }
                            LcfDataBaseConditionsHeader::Start2(PascalString(text)) => {
                                format!("52 = '''{text}'''\n")
                            }
                            LcfDataBaseConditionsHeader::Finish(PascalString(text)) => {
                                format!("55 = '''{text}'''\n")
                            }
                            LcfDataBaseConditionsHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::BattleAnimations(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("19");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[battle-animation.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Chipsets(lcf_double_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("20");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_double_text {
                    output.push_str(&format!("[chipset.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseDoubleTextHeader::TextA(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseDoubleTextHeader::TextB(PascalString(text)) => {
                                format!("2 = '''{text}'''\n")
                            }
                            LcfDataBaseDoubleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Vocabulary(vocabulary) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("21");
                output.push_str(HEADER_SIGNPOST_END);
                output.push_str("[vocabulary]\n");

                for vocab in &***vocabulary {
                    output.push_str(&format!("{} = '''{}'''\n", *vocab.id, vocab.text));
                }

                output.push('\n');
            }

            LcfDataBaseHeader::System(system) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("22");
                output.push_str(HEADER_SIGNPOST_END);
                output.push_str("[system]\n");

                for vocab in &***system {
                    output.push_str(&format!("{} = '''{}'''\n", *vocab.id, vocab.text));
                }

                output.push('\n');
            }

            LcfDataBaseHeader::Switches(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("23");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[switch.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::Variables(lcf_single_text) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("24");
                output.push_str(HEADER_SIGNPOST_END);

                for entry in &***lcf_single_text {
                    output.push_str(&format!("[variable.{}]\n", *entry.id));

                    for header in &*entry.headers {
                        let line = match header {
                            LcfDataBaseSingleTextHeader::Text(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseSingleTextHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }

                    output.push('\n');
                }
            }

            LcfDataBaseHeader::GlobalEvents(events) => {
                output.push_str(HEADER_SIGNPOST_START);
                output.push_str("25");
                output.push_str(HEADER_SIGNPOST_END);

                for event in &***events {
                    output.push_str(&format!("[event.{}]\n", *event.id));

                    for header in &*event.headers {
                        let line = match header {
                            LcfDataBaseGlobalEventHeader::Name(PascalString(text)) => {
                                format!("1 = '''{text}'''\n")
                            }
                            LcfDataBaseGlobalEventHeader::Commands(commands) => {
                                let commands = &***commands;
                                let mut output_commands = String::new();

                                if commands.is_empty() {
                                    output_commands.push_str(&format!("commands = []\n\n"));
                                } else {
                                    output_commands.push_str(&format!("commands = [\n"));
                                    let mut index = 0;

                                    for LcfCommand {
                                        code,
                                        indent,
                                        text,
                                        parameters,
                                    } in &**commands
                                    {
                                        output_commands.push_str(&format!("\t[{code}, {indent}, '''{text}''', {parameters}], #{index}\n"));
                                        index += 1;
                                    }

                                    output_commands.push_str(&format!("]\n\n"));
                                }

                                output_commands
                            }
                            LcfDataBaseGlobalEventHeader::Generic(ListEntryHeaderGeneric {
                                id,
                                value,
                            }) => format!("{id} = {value}\n"),
                        };
                        output.push_str(&line);
                    }
                }
            }

            LcfDataBaseHeader::Generic(ListEntryHeaderGeneric { id, value }) => {
                output.push_str(&format!("{id} = {value}\n"));
            }
        }
    }

    // Cleanup
    let mut output = output.trim_end().to_string();
    output.push_str("\n");

    output
}

pub fn generate_toml_maptree(maptree: &LcfMapTree) -> String {
    let mut output = String::from(DISCLAIMER_TEXT);
    output.push_str("[header]\ntype = '''LcfMapTree'''\n\n");

    for map in &maptree.0 .0 {
        output.push_str(&format!("[map.{}]\n", *map.id));

        for header in &*map.headers {
            let line = match header {
                LcfMapTreeMapHeader::Name(PascalString(name)) => {
                    format!("1 = '''{name}'''\n")
                }
                LcfMapTreeMapHeader::ParentMap(parent_id) => {
                    let parent_id = ***parent_id;
                    format!("2 = {parent_id}\n")
                }
                LcfMapTreeMapHeader::Generic(ListEntryHeaderGeneric { id, value }) => {
                    format!("{id} = {value}\n")
                }
            };

            output.push_str(&line);
        }

        output.push_str("\n");
    }

    // Cleanup
    let mut output = output.trim_end().to_string();
    output.push_str("\n");

    output
}

// Although the default toml::to_string() actually does what I want quite well, it just isn't quite there yet.
pub fn generate_toml_patch(patch: &Patch) -> String {
    let mut output = String::new();

    if let Some(dialogue) = &patch.dialogue {
        for PatchDialogue {
            event,
            page,
            command,
            indent: explicitly_defined_indent,
            has_portrait,
            character,
            original,
            patched,
        } in dialogue
        {
            output.push_str("[[dialogue]]\n");
            output.push_str(&format!("event = {event}\n"));
            if let Some(page) = page {
                output.push_str(&format!("page = {page}\n"));
            }
            output.push_str(&format!("command = {command}\n"));
            if let Some(indent) = explicitly_defined_indent {
                output.push_str(&format!("indent = {indent}\n"));
            }
            if let Some(has_portrait) = has_portrait {
                output.push_str(&format!("has_portrait = {has_portrait}\n"));
            }
            if let Some(character) = character {
                output.push_str(&format!("character = '''{character}'''\n"));
            }

            // NOTE: This extra newline is to make the dialogue lines pretty for manual editing.
            // Be sure to keep this in mind when reading the patch files!
            output.push_str(&format!("original = '''\n{original}\n'''\n"));
            output.push_str(&format!("patched = '''\n{patched}\n'''\n\n"));
        }
    }

    if let Some(text) = &patch.text {
        for PatchText {
            event,
            page,
            command,
            has_portrait,
            original,
            patched,
        } in text
        {
            output.push_str("[[text]]\n");
            output.push_str(&format!("event = {event}\n"));
            if let Some(page) = page {
                output.push_str(&format!("page = {page}\n"));
            }
            output.push_str(&format!("command = {command}\n"));
            if let Some(has_portrait) = has_portrait {
                output.push_str(&format!("has_portrait = {has_portrait}\n"));
            }
            output.push_str(&format!("original = '''{original}'''\n"));
            output.push_str(&format!("patched = '''{patched}'''\n\n"));
        }
    }

    if let Some(splice_commands) = &patch.splice_commands {
        for PatchSpliceCommands {
            event,
            page,
            replace_commands_from,
            replace_commands_to,
            commands,
        } in splice_commands
        {
            output.push_str("[[splice-commands]]\n");
            output.push_str(&format!("event = {event}\n"));
            if let Some(page) = page {
                output.push_str(&format!("page = {page}\n"));
            }
            output.push_str(&format!(
                "replace_commands_from = {replace_commands_from}\n"
            ));
            output.push_str(&format!("replace_commands_to = {replace_commands_to}\n"));

            if commands.is_empty() {
                output.push_str(&format!("commands = []\n\n"));
            } else {
                output.push_str(&format!("commands = [\n"));

                for LcfCommand {
                    code,
                    indent,
                    text,
                    parameters,
                } in &**commands
                {
                    output.push_str(&format!(
                        "\t[{code}, {indent}, '''{text}''', {parameters}],\n"
                    ));
                }

                output.push_str(&format!("]\n\n"));
            }
        }
    }

    if let Some(database_vocabulary) = &patch.database_vocabulary {
        for PatchDatabaseVocabulary {
            id,
            original,
            patched,
        } in database_vocabulary
        {
            output.push_str("[[database-vocabulary]]\n");
            output.push_str(&format!("id = {id}\n"));
            output.push_str(&format!("original = '''{original}'''\n"));
            output.push_str(&format!("patched = '''{patched}'''\n\n"));
        }
    }

    // Cleanup
    let mut output = output.trim_end().to_string();
    output.push_str("\n");

    output
    //toml::to_string(patch).unwrap()
}
