use super::portals::{NetherPortal, NetherPortals};
use crate::eframe_tools::ModalMachine;
use crate::images::{ImageDetails, ImageDetailsList, Imager, ImagerList};
use crate::thread_tools::SPromise;
use crate::url_tools::{Routes, Urls};
use crate::windows::client_windows::Loglet;
use crate::HandleError;
use crate::MagicError;
use egui_extras::RetainedImage;
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

    let name = image_details.name.clone();
    let response = download_image(&name)?;
    let image = to_retained_image(response)?;

    let imager = Imager::new(name, Some(image_details), Some(image));
    Ok(imager)
}

pub fn some_retained_image(np: &mut NetherPortal) -> Option<&RetainedImage> {
    // A image will not be selected initially, so make sure to set one before you try to display it/get a key! because it will be blank if you dont...
    np.init_img_pos();

    // Get the position/key
    let key = np.image_pos_ref();
    println!("another image pos: |{}|", key);

    // dot into BTree of (struct Imager) -> into a specific (struct SPromise) -> check if its done downloading -> get a reference to the image
    let retained_image = np
        .images_ref()
        .get(key)?
        .spromise_ref()?
        .ready()?
        .image_ref()?;

    // return the contained image as a reference
    Some(retained_image)
}

pub fn should_we_reload_nether_images(
    nps: &mut NetherPortals,
) -> Option<(&mut NetherPortal, String)> {
    let position = nps.get_neth_pos()?;
    // put (nps.=> to images) in the function signature, that way i dont have to rewrite it twice
    let np = nps.nether_mut().get_mut(&position)?;
    let images = np.images_mut();
    if images.len() != 0 {
        // No load/reload necessary
        return None;
    }

    Some((np, position))
}

pub fn should_we_reload_ow_images(nps: &mut NetherPortals) -> Option<(&mut NetherPortal, String)> {
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

fn execute_futures(np: &mut NetherPortal, runtime: &Runtime, err_msg_sender: Sender<Loglet>) {
    let mut list_of_futures = Vec::new();

    //np.images_mut().iter_mut().for_each(|(_, spromise_imager)| {
    for (_, spromise_imager) in np.images_mut().iter_mut() {
        || -> Option<()> {
            // Create Senders for async/thread communication
            let imager_sender = spromise_imager.take_sender()?;
            let em_sender = err_msg_sender.clone();

            // Clone the struct ImageDetails from inside struct Imager; It has to be 'Send'-able, so plain refs dont work
            let image_details = spromise_imager
                .ref_value()
                .image_details_ref()
                .unwrap()
                .clone();

            // Create a closure to be performed asyncronously/threaded and spawn a tokio thread
            let fut = move || match get_imager(image_details.clone()) {
                Ok(imager) => {
                    println!("Image Sent!");
                    imager_sender.send(imager);
                }
                Err(err) => {
                    em_sender.send(Loglet::err(err)).unwrap();
                    // the sender has to be used otherwise it will through a panic if its dropped before use...
                    imager_sender.send(Imager::new(
                        image_details.name.clone(),
                        Some(image_details),
                        None,
                    ))
                }
            };

            list_of_futures.push(fut);
            Some(())
        }();
    }
    //});

    // Then spawn a thread for the futures. Let ToKIoOOoOO #ThePRIMEagen do its magic
    for task in list_of_futures.into_iter() {
        runtime.spawn(async move { task() });
    }
}

fn get_image_details_list(true_name: String) -> Result<ImageDetailsList, MagicError> {
    // Calls a bunch of functions to finally produce a HashMap of struct ImageDetails, Keyed with the name of the image
    let response = image_details_list(&true_name)?;
    let image_details_list = to_image_details(response)?;
    Ok(image_details_list)
}

pub fn load_images(
    np: &mut NetherPortal,
    position: String,
    runtime: &Runtime,
    err_msg_sender: Sender<Loglet>,
) -> Option<()> {
    // Get image names from the server; They are to be used to download the exact image needed
    let image_details_list = get_image_details_list(position.clone())
        .send_error(err_msg_sender.clone())
        .ok()?;

    // SHOVE those values into NetherPortals
    merge_image_details_to_nether_portals(np.images_mut(), image_details_list);

    // Use all those SHOVED values to Execute image calls as a async/thread
    execute_futures(np, runtime, err_msg_sender);

    Some(())
}

pub fn reload_image_mm(nps: &mut NetherPortals, realm: &crate::Realm, id: String) -> Option<()> {
    // Get the chosen nether portal key
    let position = nps.realm_pos(realm)?;

    // Chosen realm is just a convienence variable
    let chosen_realm = nps.realm_ref(realm).get(&position)?;

    // Take all the image keys and convert to an arrary
    let options: Vec<String> = chosen_realm.images_ref().keys().cloned().collect();

    // Get the current image selected
    let selected_option = options.first()?.to_owned();

    // Create a struct ModalMachine to assign
    let mm = ModalMachine::new(
        selected_option,
        options,
        "Overworld Images List".to_string(),
    );

    // Assign the current ModalMachine
    nps.set_image_modal(realm, mm);
    println!("Finished!");

    Some(())
}
