use crate::increment::Inc;
use crate::option;
use crate::pages::nether_portals_page::portals::NetherPortals;
use crate::HandleOption;
use crate::Realm;
use eframe::egui::Ui;
use egui_extras::RetainedImage;

use crate::eframe_tools::display_retained_image;
use crate::pages::nether_portals_page::download_images::*;

pub fn change_image_by_user_input(nps: &mut NetherPortals, realm: &Realm) -> Option<()> {
    // If there is an event continue, otherwise return
    nps.image_modal_mut(realm).use_event()?;

    // Get the key to the current NetherPortal
    let pos = &nps.realm_pos(realm)?;

    // Get the new image position from ModalMachine
    let new_pos = nps.image_modal_mut(realm).get_selected_option();

    // Get the current NetherPortal
    let np = nps.realm_mut(realm).get_mut(pos)?;

    // Set the position of the newly (user) selected image
    np.img_pos_set(new_pos);

    Some(())
}

pub fn image_from_np<'a>(nps: &'a mut NetherPortals, realm: &Realm) -> Option<&'a RetainedImage> {
    let pos = &nps.realm_pos(realm)?;
    let np = nps.realm_mut(realm).get_mut(pos)?;
    let retained_image = some_retained_image(np)?;

    Some(retained_image)
}
pub fn display_nether_portal_images(nps: &mut NetherPortals, unique: &mut Inc, ui: &mut Ui) {
    // If there are images to display, Then display them!
    let realm = &Realm::Overworld;
    option(|| {
        let retained_image = image_from_np(nps, realm).on_none(|| ui.spinner())?;
        display_retained_image(retained_image, ui);
        Some(())
    });

    // Display the ModalMachine!
    nps.image_modal_mut(realm).modal_machine(unique.up(), ui);

    // If there are images to display, Then display them!
    let realm = &Realm::Nether;
    option(|| {
        let retained_image = image_from_np(nps, realm).on_none(|| ui.spinner())?;
        display_retained_image(retained_image, ui);
        Some(())
    });

    // Display the ModalMachine!
    nps.image_modal_mut(realm).modal_machine(unique.up(), ui);
}
