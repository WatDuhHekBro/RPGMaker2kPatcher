use crate::structs::{
    map::*,
    patch::{Dialogue, Text},
    LcfCommonCommand, LcfCommonGeneric, Patch,
};

// NOTE: Because of the way TOML treats string literals (single quotes), you cannot escape anything at all,
// neither backslashes nor single quotes. So in order to be safe, you must use the multiline literals
// (triple single quotes)! That'll take care of any problems with apostrophes.

pub fn generate_toml_map(map: &LcfMapUnit) -> String {
    let mut output = String::from("# This binary representation has been serialized for easier debugging/patching.\n# It will not be used by the program to actually patch anything, so feel free to delete this.\n\n");

    ////////////
    // Header //
    ////////////
    let mut output_header = String::from("[header]\ntype = '''LcfMapUnit'''\n");
    let mut output_events = String::new();

    // This disgusting nesting could probably be done more elegantly... but oh well.
    for header in &*map.headers {
        match header {
            LcfMapUnitHeader::Panorama(name) => {
                output_header.push_str(&format!("32 = '''{name}'''\n"))
            }
            LcfMapUnitHeader::Generic(LcfCommonGeneric { id, value }) => {
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
                            LcfMapUnitEventHeader::Generic(LcfCommonGeneric { id, value }) => {
                                output_current_event.push_str(&format!("{id} = {value}\n"))
                            }
                            LcfMapUnitEventHeader::Pages(pages) => {
                                for page in &***pages {
                                    let mut output_current_page =
                                        format!("[event.{}.page.{}]\n", event.id, page.id);

                                    for page_header in &page.headers.0 {
                                        match page_header {
                                            LcfMapUnitPageHeader::Name(name) => output_current_page
                                                .push_str(&format!("21 = '''{name}'''\n")),
                                            LcfMapUnitPageHeader::Generic(LcfCommonGeneric {
                                                id,
                                                value,
                                            }) => output_current_page
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

                                                    for LcfCommonCommand {
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

// Although the default toml::to_string() actually does what I want quite well, it just isn't quite there yet.
pub fn generate_toml_patch(patch: &Patch) -> String {
    let mut output = String::new();

    if let Some(dialogue) = &patch.dialogue {
        for Dialogue {
            event,
            page,
            command,
            indent: explicitly_defined_indent,
            original,
            patched,
        } in dialogue
        {
            output.push_str("[[dialogue]]\n");
            output.push_str(&format!("event = {event}\n"));
            output.push_str(&format!("page = {page}\n"));
            output.push_str(&format!("command = {command}\n"));

            if let Some(indent) = explicitly_defined_indent {
                output.push_str(&format!("indent = {indent}\n"));
            }

            // NOTE: This extra newline is to make the dialogue lines pretty for manual editing.
            // Be sure to keep this in mind when reading the patch files!
            output.push_str(&format!("original = '''\n{original}\n'''\n"));
            output.push_str(&format!("patched = '''\n{patched}\n'''\n\n"));
        }
    }

    if let Some(replace) = &patch.text {
        for Text {
            event,
            page,
            command,
            original,
            patched,
        } in replace
        {
            output.push_str("[[text]]\n");
            output.push_str(&format!("event = {event}\n"));
            output.push_str(&format!("page = {page}\n"));
            output.push_str(&format!("command = {command}\n"));
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
