use crate::{
    images::Imager,
    increment::Inc,
    pages::nether_portals_page::{
        download::download_nether_portals,
        portals::{NetherPortal, NetherPortalBTree, NetherPortals},
    },
    windows::error_messages::ErrorMessage,
};
use eframe::egui::Ui;
use std::sync::Once;
use tokio::runtime::Runtime;

use super::display::displayer;

// Globals
static START: Once = Once::new();

fn check_promises() {}

fn setup_displayables(nether_portals: &mut NetherPortals) {
    // If its empty iter through and convert NP to BTree
    if !nether_portals.is_overworld_empty() {
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
    if !nether_portals.is_nether_empty() {
        nether_portals
            .nether_mut()
            .iter_mut()
            .for_each(|(_, nether_portal)| {
                if nether_portal.is_empty() {
                    nether_portal.set_as_btree()
                }
            });
        let keys: Vec<String> = nether_portals
            .nether_ref()
            .keys()
            .map(|ref_s| ref_s.to_string())
            .collect();
        nether_portals.set_neth_pos(keys);
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

    setup_displayables(nether_portals);

    displayer(nether_portals, unique, ui);

    check_promises();
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
