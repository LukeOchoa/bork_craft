use crate::{
    err_tools::ErrorX,
    increment::Inc,
    pages::nether_portals_page::{
        download_npt::download_nether_portals,
        portals::{NetherPortals, PortalText},
    },
    thread_tools::SPromise,
    url_tools::{Routes, Urls},
    windows::{client_windows::Loglet, error_messages::ErrorMessage},
    HandleError, MagicError, StatusCheck,
};
use eframe::egui::Ui;
use std::sync::mpsc::Sender;
use std::sync::Once;
use tokio::runtime::Runtime;

use super::{display::displayer, portals::NetherPortalText};

// Globals
static START: Once = Once::new();

fn check_promises() {}

fn save_nether_portal(npt: NetherPortalText) -> Result<ureq::Response, MagicError> {
    let url = &Urls::default(Routes::UpdateNetherPortalText);
    let response = ureq::post(url).send_json(npt)?;

    Ok(response)
}

//fn save_all_changes() {
//}

fn save_this_change(
    nether_portals: &mut NetherPortals,
    neth_key: &String,
    runtime: &Runtime,
    ow_key: &String,
) -> Result<(), MagicError> {
    // Error Message Maker
    let errmsg = |key: &String| {
        format!(
            "Failed to save struct NetherPortal because of bad key: |{}|",
            key
        )
    };

    // Get Refs to BOTH nether&overworld btrees
    let ow_btree = nether_portals
        .overworld_ref()
        .get(ow_key)
        .ok_or(ErrorX::new_box(&errmsg(ow_key)))?
        .btree_ref();
    let neth_btree = nether_portals
        .nether_ref()
        .get(neth_key)
        .ok_or(ErrorX::new_box(&errmsg(ow_key)))?
        .btree_ref();

    // Convert BOTH to type PortalText structs
    let overworld = PortalText::from_btree(ow_btree)?;
    let nether = PortalText::from_btree(neth_btree)?;

    // Build a NetherPortalText the PortalTexts
    let ow_id = nether_portals.overworld_ref()[ow_key].get_id();
    let neth_id = nether_portals.nether_ref()[neth_key].get_id();
    let id = if ow_id < 0 { neth_id } else { ow_id };

    let npt = NetherPortalText::build_from(id, overworld, nether);

    // Create a notifier
    let (spromise, sender) = SPromise::make_promise();
    nether_portals.set_text_request(spromise);

    // spawn a async thread to handle the request
    runtime.spawn(async move {
        let subfn = || -> Result<(), MagicError> {
            save_nether_portal(npt)?.status_check()?;
            Ok(())
        };
        // Some == Err() & None == Ok(); Result<> doesnt impl Default so i couldn't use it lol
        if let Err(err) = subfn() {
            sender.send(Some(err.to_string()))
        } else {
            sender.send(None)
        }
    });

    Ok(())
}

fn setup_displayables(nether_portals: &mut NetherPortals) {
    //! Iter through each NP and make a key collection from them

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

        // Append to NetherPortals
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

fn is_request_processing(np: &NetherPortals, ui: &mut Ui) -> Result<(), MagicError> {
    // Bind let: if  there is currently a Promise to uphold inside SPromise
    let promise = match np.text_request_ref().spromise_ref() {
        Some(promise) => promise,
        None => return Ok(()),
    };

    // if promise is NOT ready(request not finished), THEN ui.spinner
    // TODO change Option<String> to Result<(), MagicError>
    let some_err = match promise.ready() {
        Some(err) => err,
        None => {
            ui.spinner();
            return Ok(());
        }
    };

    // Some(err) == Err(), None == Successful request!
    match some_err {
        Some(err) => Err(ErrorX::new_box(err)),
        None => Ok(()),
    }
}

fn save_npt(
    nether_portals: &mut NetherPortals,
    runtime: &Runtime,
    ui: &mut Ui,
) -> Result<(), MagicError> {
    // If the request is still processing: True => Ui.Spinner; False => Reset the np.request
    is_request_processing(nether_portals, ui).otherwise(|_| {
        nether_portals.set_text_request(SPromise::make_no_promise(None));
    })?;
    // TODO use .inspect_err() in the future when it is stable (currently unstable only)

    // On button click, Save Changes
    if ui.button("Save This Change").clicked() {
        let ow_key = &nether_portals.get_ow_pos().unwrap();
        let neth_key = &nether_portals.get_neth_pos().unwrap();

        // Set both (NetherPortals.portal_text)s with data from .as_btree
        nether_portals
            .overworld_mut()
            .get_mut(ow_key)
            .unwrap()
            .set_pt()?;
        nether_portals
            .nether_mut()
            .get_mut(neth_key)
            .unwrap()
            .set_pt()?;

        // Execute async request
        save_this_change(nether_portals, neth_key, runtime, ow_key)?;
    }

    Ok(())
}

fn reload_npt(
    nether_portal_sender: Sender<NetherPortalText>,
    err_msg_sender: Sender<Loglet>,
    runtime: &Runtime,
    ui: &mut Ui,
) {
    if ui.button("Reload From DB").clicked() {
        download_nether_portals(nether_portal_sender, err_msg_sender, runtime);
    }
}

// Big Boi Function
pub fn display_nether_portals_page(
    nether_portals: &mut NetherPortals,
    unique: &mut Inc,
    err_msg: &mut ErrorMessage,
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

    // Buttons
    ui.horizontal(|ui| {
        save_npt(nether_portals, runtime, ui).consume_error(err_msg);
        reload_npt(
            nether_portals.npt_sender_clone(),
            err_msg.sender_clone(),
            &runtime,
            ui,
        );
    });

    displayer(nether_portals, unique, ui);

    check_promises();
}
