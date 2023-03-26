use crate::{
    images::Imager,
    increment::Inc,
    pages::nether_portals_page::{
        download::download_nether_portals,
        portals::{NetherPortal, NetherPortals},
    },
    windows::error_messages::ErrorMessage,
};
use eframe::egui::Ui;
use std::sync::Once;
use tokio::runtime::Runtime;

use super::display::{self, portal_text_displayer};

// Globals
static START: Once = Once::new();

fn check_promises() {}

// Nether Portals contains
//  1) Overworld NetherPortal
//  2) Nether NetherPoral
//
// NetherPortal contains
//      portal_text: SPromise<PortalText, F>,
//      as_btree: PortalTextBTree,
//      images: BTreeMap<String, SPromise<Imager, F>>,
//

// 1) Get a &mut (Ref Mut) to OverWorld & Nether
// 2) Check if they empty, on NOT empty
// 2.1) get the value from (SPromise.value)
// convert it to a btree

fn setup_displayables(nether_portals: &mut NetherPortals) {
    // If its empty iter through and convert NP to BTree
    if !nether_portals.is_nether_empty() {
        nether_portals
            .overworld_mut()
            .iter_mut()
            .for_each(|(_, nether_portal)| {
                if nether_portal.is_empty() {
                    nether_portal.set_as_btree()
                }
            });

        // Create a collection of keys to be iterated back&forth over
        let keys: Vec<String> = nether_portals
            .overworld_ref()
            .keys()
            .map(|ref_s| ref_s.to_string())
            .collect();
        // Append
        nether_portals.set_ow_pos(keys);
    }

    //if !nether_portals.is_nether_empty() {
    //    nether_portals
    //        .nether_mut()
    //        .iter_mut()
    //        .for_each(|(_, nether_portal)| {
    //            if nether_portal.is_empty() {
    //                nether_portal.set_as_btree()
    //            }
    //        });

    //    let keys: Vec<String> = nether_portals
    //        .nether_ref()
    //        .keys()
    //        .map(|ref_s| ref_s.to_string())
    //        .collect();
    //    nether_portals.set_neth_pos(keys);
    //}
}
fn move_back_or_forth(nether_portals: &mut NetherPortals, ui: &mut Ui) {
    if ui.button("Go Back").clicked() {
        nether_portals.ow_pos_down();
    }

    if ui.button("Go Forth").clicked() {
        nether_portals.ow_pos_up();
    }
}

pub fn display_nether_portals_page(
    nether_portals: &mut NetherPortals,
    unique: &mut Inc,
    err_msg: &ErrorMessage,
    runtime: &Runtime,
    ui: &mut Ui,
) {
    START.call_once(|| {
        download_nether_portals(
            nether_portals.npt_sender_clone(),
            err_msg.sender_clone(),
            runtime,
        );
    });
    if ui.button("Hot Reload").clicked() {
        setup_displayables(nether_portals);
    }

    if !nether_portals.is_overworld_empty() {
        move_back_or_forth(nether_portals, ui);
        let mut reset = false;
        if ui.button("Reset").clicked() {
            reset = true
        }

        if let Some(key) = nether_portals.get_ow_pos() {
            nether_portals
                .overworld_ref()
                .get(&key)
                .and_then(|display_portal| {
                    portal_text_displayer(display_portal.btree_ref(), reset, unique.up_str(), ui);
                    None::<&NetherPortals>
                });
            nether_portals
                .overworld_ref()
                .get("Luke SpawnPoint")
                .and_then(|dp| {
                    portal_text_displayer(dp.btree_ref(), reset, unique.up_str(), ui);
                    None::<&NetherPortals>
                });
        }
    }

    check_promises();
}

// Test code, delete later
fn test_downloader(nether_portals: &NetherPortals, ui: &mut Ui) {
    // TEST FUNCTIONS
    if nether_portals.is_overworld_empty() {
        ui.spinner();
    } else {
        ui.label(nether_portals.quicko());
    }

    if nether_portals.is_nether_empty() {
        ui.spinner();
    } else {
        ui.label(nether_portals.quickn());
    }
}
// SPromise
// You have a value
// when you need a new value
//
// 1) Run the future
// 2) Give sender to the future
// 3) Pass future to a thread/tokio
// 4) Use a spinner in place of the loading data
// 5) Once promise if fulfilled, take() from Option
//     and give it to SPromise.value
//      making SPromise.some_value as None
