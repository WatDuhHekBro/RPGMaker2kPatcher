use crate::{structs::map::*, types::PascalString, wrappers::*};

//TMP = output_events.push_str(&format!("\n"));

pub fn generate_toml_representation(map: &LcfMapUnit) -> String {
    let mut output = String::from("# This binary representation has been serialized for easier debugging/patching. It will not be used by the program to actually patch anything, so feel free to delete this.\n\n");

    ////////////
    // Header //
    ////////////
    let mut output_header = String::from("[header]\ntype = 'LcfMapUnit'\n");
    let mut output_events = String::new();

    // This disgusting nesting could probably be done more elegantly... but oh well.
    for header in &map.headers.0 {
        match header {
            LcfMapUnitHeader::End => {}
            LcfMapUnitHeader::Panorama(name) => {
                output_header.push_str(&format!("32 = '{}'\n", get_safe_single_quoted_string(name)))
            }
            LcfMapUnitHeader::Generic(LcfMapUnitHeaderGeneric { id, value }) => {
                output_header.push_str(&format!("{id} = {value}\n"));
            }
            LcfMapUnitHeader::Events(MapEventsWrapper(events)) => {
                for event in events {
                    let mut output_current_event = format!("[event.{}]\n", event.id);
                    let mut output_pages = String::new();

                    for event_header in &event.headers.0 {
                        match event_header {
                            LcfMapUnitEventHeader::End => {}
                            LcfMapUnitEventHeader::Name(name) => output_current_event.push_str(
                                &format!("1 = '{}'\n", get_safe_single_quoted_string(name)),
                            ),
                            LcfMapUnitEventHeader::Generic(LcfMapUnitEventHeaderGeneric {
                                id,
                                value,
                            }) => output_current_event.push_str(&format!("{id} = {value}\n")),
                            LcfMapUnitEventHeader::Pages(MapPagesWrapper(pages)) => {
                                for page in pages {
                                    let mut output_current_page =
                                        format!("[event.{}.page.{}]\n", event.id, page.id);

                                    for page_header in &page.headers.0 {
                                        match page_header {
                                            LcfMapUnitPageHeader::End => {}
                                            LcfMapUnitPageHeader::Name(name) => output_current_page
                                                .push_str(&format!(
                                                    "21 = '{}'\n",
                                                    get_safe_single_quoted_string(name)
                                                )),
                                            LcfMapUnitPageHeader::Generic(
                                                LcfMapUnitPageHeaderGeneric { id, value },
                                            ) => output_current_page
                                                .push_str(&format!("{id} = {value}\n")),
                                            LcfMapUnitPageHeader::Commands(MapCommandsWrapper(
                                                commands,
                                            )) => {
                                                if commands.is_empty() {
                                                    output_current_page
                                                        .push_str(&format!("commands = []\n"));
                                                } else {
                                                    output_current_page
                                                        .push_str(&format!("commands = [\n"));
                                                    let mut index = 0;

                                                    for LcfMapUnitCommand {
                                                        event,
                                                        indent,
                                                        text,
                                                        parameters,
                                                    } in commands
                                                    {
                                                        output_current_page.push_str(&format!("\t[{event}, {indent}, '{}', {parameters}], #{index}\n", get_safe_single_quoted_string(text)));
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
                }
            }
        }
    }

    output.push_str(&output_header);
    output.push_str("\n");
    output.push_str(&output_events);

    output
}

pub fn generate_toml_patch(data: &LcfMapUnit) -> String {
    let mut output = String::new();

    output
}

// TODO: Find an actual safe method
fn get_safe_single_quoted_string(text: &PascalString) -> String {
    text.to_string().replace("'", "\\'")
}
