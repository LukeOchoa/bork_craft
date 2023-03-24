use crate::{
    images::Imager,
    pages::nether_portals_page::{download::download_nether_portals, portals::NetherPortals},
    windows::error_messages::ErrorMessage,
};
use eframe::egui::Ui;
use std::sync::Once;
use tokio::runtime::Runtime;

// Globals
static START: Once = Once::new();

fn check_promises() {}

pub fn display_nether_portals_page(
    nether_portals: &mut NetherPortals,
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

    test_downloader(nether_portals, ui);
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
