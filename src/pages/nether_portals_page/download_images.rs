use super::portals::NetherPortal;
use crate::images::{ImageDetails, ImageDetailsList, Imager};
use crate::url_tools::{Routes, Urls};
use crate::windows::client_windows::Loglet;
use crate::MagicError;

use std::sync::mpsc::Sender;
use tokio::runtime::Runtime;

fn to_image_details(resp: ureq::Response) -> Result<ImageDetailsList, MagicError> {
    let image_details = serde_json::from_reader(resp.into_reader())?;

    Ok(image_details)
}

fn image_details_list(true_name: &String) -> Result<ureq::Response, MagicError> {
    //! Downloads meta details about the image from a api/server
    let url: String = Urls::default(Routes::GetNetherPortalImageNames);
    let result = ureq::get(&format!("{}?true_name={}", url, true_name)).call()?;

    Ok(result)
}

// DO THESE TINGS FIRST
pub fn get_image_details_list(true_name: String) -> Result<ImageDetailsList, MagicError> {
    // Calls a bunch of functions to finally produce a HashMap of struct ImageDetails, Keyed with the name of the image
    let response = image_details_list(&true_name)?;
    let image_details_list = to_image_details(response)?;
    println!("The length of the list is: {}", image_details_list.len());
    Ok(image_details_list)
}

// After image_details_list is downloaded; DO THESE TINGS
fn download_image(true_name: &String) -> Result<ureq::Response, MagicError> {
    //! Download image specified by the name from the server
    let url = Urls::default_i(Routes::GetNetherPortalImage);
    let response = ureq::get(&format!("{}?name={}", url, true_name)).call()?;

    Ok(response)
}
fn to_retained_image(response: ureq::Response) -> Result<egui_extras::RetainedImage, MagicError> {
    //! Convert response to struct RetainedImage
    let url = "Your Mom.";
    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes)?;
    let retained_image = egui_extras::image::RetainedImage::from_image_bytes(url, &bytes)?;
    Ok(retained_image)
}

fn get_imager(image_details: ImageDetails) -> Result<Imager, MagicError> {
    //! Calls a bunch of functions to finally produce a struct Imager
    println!("Did get imager run?");

    let name = image_details.name.clone();
    let response = download_image(&name)?;
    let image = to_retained_image(response)?;

    let imager = Imager::new(name, Some(image_details), Some(image));
    Ok(imager)
}

pub fn execute_futures(np: &mut NetherPortal, runtime: &Runtime, err_msg_sender: Sender<Loglet>) {
    println!(
        "inside execute futures, the length is :|{}|",
        np.images_ref().len()
    );

    let mut list_of_futures = Vec::new();
    np.images_mut().iter_mut().for_each(|(_, spromise_imager)| {
        // Create Senders for async/thread communication
        let imager_sender = spromise_imager.take_sender().unwrap();
        let em_sender = err_msg_sender.clone();
        // Clone the struct ImageDetails from inside struct Imager; It has to be 'Send'-able, so plain refs dont work
        let image_details = spromise_imager
            .ref_value()
            .image_details_ref()
            .as_ref()
            .unwrap()
            .clone();
        // Create a function to be performed asyncronously/threaded and spawn a tokio thread

        let fut = move || match get_imager(image_details) {
            Ok(imager) => {
                println!("Image Sent!");
                imager_sender.send(imager);
            }
            Err(err) => {
                em_sender.send(Loglet::err(err)).unwrap();
                println!("Did we run into an error?");
            }
        };
        list_of_futures.push(fut);
    });
    for task in list_of_futures.into_iter() {
        runtime.spawn(async move { task() });
    }
}

// Move to a different file
// Every time a person selects a NetherPortal to be displayed

// We need to check for images, if there are no images, then load them in
// The position of what NetherPortal is being view is stored the (NetherPortals!) master struct
// You can get it with (fn get_ow_pos())/(fn get_neth_pos())

// check the len of the BTree<SPromise<Imager>>
// on 0, load in all its related images and create any necessary datastructures

use crate::eframe_tools::display_retained_image;
use crate::images::ImagerList;
use crate::pages::nether_portals_page::portals::NetherPortals;
use crate::thread_tools::SPromise;
use crate::HandleError;
use eframe::egui::Ui;
use egui_extras::RetainedImage;

fn some_retained_image(np: &mut NetherPortal) -> Option<&RetainedImage> {
    np.set_image_pos();
    let key = np.image_pos_ref();
    let retained_image = np
        .images_ref()
        .get(key)?
        .spromise_ref()
        .as_ref()?
        .ready()?
        .image_ref()
        .as_ref()?;

    Some(retained_image)
}

fn display_ow_image_helper(nps: &mut NetherPortals) -> Option<&RetainedImage> {
    let position = &nps.get_ow_pos()?;
    let np = nps.overworld_mut().get_mut(position)?;
    let retained_image = some_retained_image(np)?;

    Some(retained_image)
}

fn display_nether_image_helper(nps: &mut NetherPortals) -> Option<&RetainedImage> {
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
    if ui.button("Downloader Button").clicked() {
        if let Some((np, position)) = should_we_reload_ow_images(nps) {
            println!("Position Overworld: {}.", position);
            load_images(np, position, runtime, err_msg_sender.clone());
        }

        if let Some((np, position)) = should_we_reload_nether_images(nps) {
            println!("Position Nether: {}.", position);
            load_images(np, position, runtime, err_msg_sender);
        }
    }

    match display_ow_image_helper(nps) {
        Some(retained_image) => {
            display_retained_image(retained_image, ui);
        }
        None => {
            ui.spinner();
        }
    }
    match display_nether_image_helper(nps) {
        Some(retained_image) => {
            display_retained_image(retained_image, ui);
        }
        None => {
            ui.spinner();
        }
    }
}

// temp func to save logic
fn should_we_reload_nether_images(nps: &mut NetherPortals) -> Option<(&mut NetherPortal, String)> {
    let position = nps.get_neth_pos()?;
    // put (nps.=> to images) in the function signature, that way i dont have to rewrite it twice
    let np = nps.nether_mut().get_mut(&position)?;
    let images = np.images_mut();
    if images.len() != 0 {
        println!("THE LENGTH OF IMAGES IN NETHER: |{}|", images.len());
        // No load/reload necessary
        return None;
    }

    Some((np, position))
}
fn should_we_reload_ow_images(nps: &mut NetherPortals) -> Option<(&mut NetherPortal, String)> {
    let position = nps.get_ow_pos()?;
    let np = nps.overworld_mut().get_mut(&position)?;
    let images = np.images_mut();

    if images.len() != 0 {
        return None;
    }

    Some((np, position))
}

fn merge_image_details_to_nether_portals(
    imager_list: &mut ImagerList,
    mut image_details_list: ImageDetailsList,
) {
    image_details_list.drain().for_each(|(_, image_details)| {
        let name = image_details.name.clone();
        let imager = Imager::new(name.clone(), Some(image_details), None);
        imager_list.insert(name, SPromise::create_promise(imager));
    });
}

pub fn load_images(
    np: &mut NetherPortal,
    position: String,
    runtime: &Runtime,
    err_msg_sender: Sender<Loglet>,
) -> Option<()> {
    // get a list of names for images to be downloaded and its related details
    let image_details_list = get_image_details_list(position.clone())
        .send_error(err_msg_sender.clone())
        .ok()?;
    println!(
        "Does this ever proc? POSITION: -> |{}|; List LENGTH -> |{}|",
        position,
        image_details_list.len()
    );
    merge_image_details_to_nether_portals(np.images_mut(), image_details_list);
    execute_futures(np, runtime, err_msg_sender);

    Some(())
}

// Testing Function
pub fn load_images_as_queue(
    np: &mut NetherPortal,
    runtime: &Runtime,
    err_msg_sender: Sender<Loglet>,
) {
    let mut list_of_tasks = Vec::new();
    for (_key, sp_imager) in np.images_mut().iter_mut() {
        // what will happen if i use unwrap here?
        let sender = sp_imager.take_sender().unwrap();
        let err_msg_sender_c = err_msg_sender.clone();
        let image_details = sp_imager
            .ref_value()
            .image_details_ref()
            .as_ref()
            .unwrap()
            .clone();
        list_of_tasks.push(move || match get_imager(image_details) {
            Ok(imager) => {
                println!("image sent!");
                sender.send(imager);
            }
            Err(err) => err_msg_sender_c.send(Loglet::err(err)).unwrap(),
        });
    }

    for task in list_of_tasks.into_iter() {
        runtime.spawn(async move { task() });
    }
}
