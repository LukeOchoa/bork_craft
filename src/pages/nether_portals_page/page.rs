use crate::{
    err_tools::ErrorX,
    images::Imager,
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
    let promise = match np.text_request_ref().spromise_ref().as_ref() {
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
    //test_downloading_images(nether_portals, runtime, err_msg.sender_clone(), ui);

    displayer(nether_portals, unique, ui);

    check_promises();
}

use crate::images::ImageDetailsList;
use crate::pages::nether_portals_page::download_images::{
    get_image_details_list, load_images_as_queue,
};
fn test_merge_image_details_to_nether_portals(
    mut image_details_list: ImageDetailsList,
    np: &mut NetherPortals,
) {
    let imager_list = np.nether_mut().get_mut("World Spawn").unwrap().images_mut();
    for (_key, image_details) in image_details_list.drain() {
        let name = image_details.name.clone();
        let imager = Imager::new(name.clone(), Some(image_details), None);
        imager_list.insert(name, SPromise::create_promise(imager));
    }

    //for (key, imager) in np
    //.nether_mut()
    //.get_mut("World Spawn")
    //.unwrap()
    //.images_mut()
    //.iter_mut()
    //{
    //let image_details
    //imager.set_value(Imager::new(, , ))
    //}
}
fn test_downloading_images(
    np: &mut NetherPortals,
    runtime: &Runtime,
    err_msg_sender: Sender<Loglet>,
    ui: &mut Ui,
) {
    if ui.button("Download Images?").clicked() {
        match get_image_details_list("World Spawn".to_string()) {
            Ok(idl) => {
                test_merge_image_details_to_nether_portals(idl, np);
                load_images_as_queue(
                    np.nether_mut().get_mut("World Spawn").unwrap(),
                    runtime,
                    err_msg_sender,
                );
            }
            Err(err) => {
                err_msg_sender.send(Loglet::err(err)).unwrap();
            }
        }
    }

    if ui.button("Display Images").clicked() {
        let nether_portal = np.nether_mut().get_mut("World Spawn").unwrap();
        if *nether_portal.image_pos_mut() == String::default() {
            let (key, _) = nether_portal.images_mut().first_key_value().unwrap();
            *nether_portal.image_pos_mut() = key.clone();
        }
    }

    if let Some(nether_portal) = np.nether_ref().get("World Spawn") {
        let key = nether_portal.image_pos_ref().clone();
        if key != String::default() {
            let r_image = nether_portal
                .images_ref()
                .get(&key)
                .unwrap()
                .spromise_ref()
                .as_ref()
                .unwrap()
                .ready()
                .unwrap()
                .image_ref()
                .as_ref()
                .unwrap();
            display_retained_image(r_image, ui);
        }
    }
}

fn display_retained_image(retained_image: &egui_extras::RetainedImage, ui: &mut eframe::egui::Ui) {
    let mut size = retained_image.size_vec2();
    size *= (ui.available_width() / size.x).min(1.0);
    retained_image.show_size(ui, size);
}
