use crate::pages::nether_portals_page::portals::NetherPortals;
use crate::Loglet;
use eframe::egui::Ui;
use egui_extras::RetainedImage;
use std::sync::mpsc::Sender;
use tokio::runtime::Runtime;

use crate::eframe_tools::display_retained_image;
use crate::pages::nether_portals_page::download_images::*;

pub fn image_from_ow_np(nps: &mut NetherPortals) -> Option<&RetainedImage> {
    //! Get the currently selected RetainedImage from NetherPortal in NetherPortals
    //!
    //! Maybe put this function inside (struct NetherPortals) impl block?
    let position = &nps.get_ow_pos()?;
    let np = nps.overworld_mut().get_mut(position)?;
    let retained_image = some_retained_image(np)?;

    Some(retained_image)
}

pub fn image_from_nether_np(nps: &mut NetherPortals) -> Option<&RetainedImage> {
    let position = &nps.get_neth_pos()?;
    let np = nps.nether_mut().get_mut(position)?;
    let retained_image = some_retained_image(np)?;

    Some(retained_image)
}

pub fn display_nether_portal_images(
    nps: &mut NetherPortals,
    runtime: &Runtime,
    err_msg_sender: Sender<Loglet>,
    ui: &mut Ui,
) {
    // Every pass we check if images need to be loaded/(reloaded? prob change name)
    if let Some((np, position)) = should_we_reload_ow_images(nps) {
        load_images(np, position, runtime, err_msg_sender.clone());
    }

    if let Some((np, position)) = should_we_reload_nether_images(nps) {
        load_images(np, position, runtime, err_msg_sender);
    }

    //
    // If there are images to display, Then display them!
    if let Some(retained_image) = image_from_ow_np(nps) {
        display_retained_image(retained_image, ui);
    } else {
        ui.spinner();
    };

    if let Some(retained_image) = image_from_nether_np(nps) {
        display_retained_image(retained_image, ui);
    } else {
        ui.spinner();
    }
}
