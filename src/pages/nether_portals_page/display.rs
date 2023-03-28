use eframe::egui::Ui;
use egui_extras::{Column, TableBuilder};
use std::collections::BTreeMap;

use super::portals::{NetherPortal, NetherPortalText, NetherPortals, PortalText};
use crate::{
    increment::Inc,
    url_tools::{Routes, Urls},
    MagicError,
};
use ureq;

fn quick_table(ui: &mut Ui, columns: usize, reset: bool) -> TableBuilder {
    //! Just for settings up a simple table

    // Width of the columns
    let mut max_width = 300.0;
    let mut min_width = 40.0;
    let mut default_width = 300.0;
    let mut default_min_width = 150.0;

    // If the table should be reset, reinitalize its initial widths
    if reset {
        max_width = 70.0;
        min_width = 70.0;
        default_width = 150.0;
        default_min_width = 150.0;
    }

    // Basic settings for each extra column
    let column_settings = Column::initial(150.0)
        .range(default_min_width..=default_width)
        .resizable(true)
        .clip(false);

    // Create table based on settings
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(
            Column::initial(70.0)
                .range(min_width..=max_width)
                .resizable(true)
                .clip(false),
        );

    // If you want more than 1 table, iter through and add more!
    if columns > 0 {
        for _ in 0..columns {
            table = table.column(column_settings)
        }
    }

    // Return the fastest table alive
    table
}

fn portal_text_displayer(displayable_pt: &BTreeMap<String, String>, table: TableBuilder) {
    // Use that table bby!
    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Type");
            });
            header.col(|ui| {
                ui.strong("Details");
            });
        })
        .body(|mut body| {
            // Iter through the BTree & Display each key/element of Nether Portal Text info
            displayable_pt.iter().for_each(|(key, value)| {
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("key: {}", key));
                    });
                    row.col(|ui| {
                        ui.label(format!("value: {}", value));
                    });
                });
            });
        })
}

fn portal_text_displayer_mut(displayable_pt: &mut BTreeMap<String, String>, table: TableBuilder) {
    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("Type");
            });
            header.col(|ui| {
                ui.strong("Details");
            });
        })
        .body(|mut body| {
            displayable_pt.iter_mut().for_each(|(key, value)| {
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("key: {}", key));
                    });
                    row.col(|ui| {
                        ui.add(egui::TextEdit::singleline(value));
                    });
                });
            })
        });
}

fn move_back_or_forth_buttons(nether_portals: &mut NetherPortals, ui: &mut Ui) {
    if ui.button("Go Back").clicked() {
        nether_portals.ow_pos_down();
    }

    if ui.button("Go Forth").clicked() {
        nether_portals.ow_pos_up();
    }
}

fn bool_button(ui: &mut Ui, name: &str, mut reset: bool) -> bool {
    if ui.button(name).clicked() {
        reset = true
    }

    reset
}
fn reset_button(ui: &mut Ui) -> bool {
    let reset = false;
    bool_button(ui, "Reset View", reset)
}
fn mutate_button(ui: &mut Ui, state: bool) -> bool {
    if ui.button("mutate").clicked() {
        !state
    } else {
        state
    }
}

fn reset_all_btree(nether_portals: &mut NetherPortals, ui: &mut Ui) {
    if ui.button("Reset All Text").clicked() {
        nether_portals
            .overworld_mut()
            .iter_mut()
            .for_each(|(_, nether_portal)| nether_portal.set_as_btree());
    }
}

fn reset_this_btree(nether_portals: &mut NetherPortals, key: &String, ui: &mut Ui) {
    if ui.button("Reset This").clicked() {
        nether_portals
            .overworld_mut()
            .get_mut(key)
            .and_then(|nether_portal| {
                nether_portal.set_as_btree();
                None::<&NetherPortals>
            });
    }
}

//fn save_nether_portal(npt: &NetherPortalText) -> Result<ureq::Response, MagicError> {
//    let url = &Urls::default(Routes::UpdateNetherPortalText);
//    let response = ureq::post(url).send_json(npt)?;
//
//    Ok(response)
//}
//
//fn save_all_changes() {
//}
//
//fn save_this(nether_portals: &mut NetherPortals, key: &String) {
//    if let Some(btree) = nether_portals.overworld_ref().get(key) {
//        let pt = PortalText::from_btree(btree);
//        let response = save_nether_portal(npt)
//    }
//}

pub fn displayer(nether_portals: &mut NetherPortals, unique: &mut Inc, ui: &mut Ui) -> Option<()> {
    // If there is no information, leave. There is nothing to display
    if nether_portals.is_overworld_empty() {
        return None;
    }

    // Get the current key chosen. if no key, nothing to show. So leave
    let ow_key = nether_portals.get_ow_pos()?;
    //nether_portals.get_neth_pos()?

    let mut reset = bool::default();
    let mut mutate = nether_portals.get_mutate();
    ui.horizontal(|ui| {
        // buttons to control which direction to seek PT information
        move_back_or_forth_buttons(nether_portals, ui);

        // Allows a Reset of the portal_text_displayer view
        reset = reset_button(ui);

        // Mutate allows for mutations inside PortalTextBTree
        mutate = nether_portals.set_mutate(mutate_button(ui, mutate));

        // Reset changes allowed by mutate
        reset_all_btree(nether_portals, ui);

        // Reset changes allow by mutate to THIS KEY only
        reset_this_btree(nether_portals, &ow_key, ui);

        // Save All Changes allowed by mutate
        //save_all_changes();

        // Save changes allowed by mutate to THIS KEY only
        //save_this();
    });

    //.overworld_ref()
    // Access The current PortalTextBTree
    nether_portals
        .overworld_mut()
        .get_mut(&ow_key)
        .and_then(|display_portal| {
            // Make sure everything is Unique to avoid clashes
            ui.push_id(unique.up_str(), |ui| {
                // Create a table
                let table = quick_table(ui, 1, reset);

                // Display Content
                match mutate {
                    true => portal_text_displayer_mut(&mut display_portal.btree_mut(), table),
                    false => portal_text_displayer(display_portal.btree_ref(), table),
                }
                //portal_text_displayer(display_portal.btree_ref(), table);
            });
            None::<&NetherPortals>
        });
    nether_portals
        .overworld_ref()
        .get("Luke SpawnPoint")
        .and_then(|dp| {
            ui.push_id(unique.up_str(), |ui| {
                let table = quick_table(ui, 1, reset);
                portal_text_displayer(dp.btree_ref(), table);
            });
            None::<&NetherPortals>
        });

    Some(())
}
