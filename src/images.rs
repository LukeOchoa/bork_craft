use crate::thread_tools::SPromise;
use egui_extras::RetainedImage;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// Move this import into this file later?

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageDetails {
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "True_name")]
    pub true_name: String,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub local_image: Option<String>, // if local_image is "True" then the image was added by the user in the current "program session"(not client/server session)
                                     // maybe at some point if the user request, then this will be used to determin if this data in the struct should be saved to a server
                                     // i need a path if its a local image.
                                     // i could create a path variable or i could simply use local_image as an Option<String> aka Option<Path>
                                     // and when my code looks for a bool value this could be the replacement...
}
impl Clone for ImageDetails {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            true_name: self.true_name.clone(),
            username: self.username.clone(),
            local_image: self.local_image.clone(),
        }
    }
}

pub type ImageDetailsList = HashMap<String, ImageDetails>;
type F = Box<dyn std::future::Future<Output = ()> + Unpin>;
pub type ImagerList = BTreeMap<String, SPromise<Imager, F>>;
//////

#[derive(Default)]
pub struct Imager {
    pub name: String,
    image_details: Option<ImageDetails>,
    image: Option<RetainedImage>,
}

impl Imager {
    pub fn default() -> Self {
        Self {
            name: Default::default(),
            image_details: None,
            image: None,
        }
    }
    pub fn new(
        name: String,
        image_details: Option<ImageDetails>,
        image: Option<RetainedImage>,
    ) -> Self {
        Self {
            name,
            image_details,
            image,
        }
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn image_details_ref(&self) -> Option<&ImageDetails> {
        self.image_details.as_ref()
    }
    pub fn image_ref(&self) -> Option<&RetainedImage> {
        self.image.as_ref()
    }
}
