use std::{collections::BTreeMap, future::Future, sync::mpsc::Sender};

use serde_derive::{Deserialize, Serialize};

use crate::{
    images::Imager,
    thread_tools::{Communicator, SPromise},
    windows::client_windows::Loglet,
};

// Traits
//use crate::New;

#[derive(Deserialize, Serialize, Default)]
pub struct PortalText {
    #[serde(rename = "Xcord")]
    xcord: i32,
    #[serde(rename = "Ycord")]
    ycord: i32,
    #[serde(rename = "Zcord")]
    zcord: i32,
    #[serde(rename = "Locale")]
    locale: String,
    #[serde(rename = "Owner")]
    owner: String,
    #[serde(rename = "Notes")]
    notes: String,
    #[serde(rename = "True_Name")]
    true_name: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct NetherPortalText {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "OverWorld")]
    overworld: PortalText,
    #[serde(rename = "Nether")]
    nether: PortalText,
    #[serde(rename = "Username")]
    username: String,
}
//#[serde(skip_serializing, skip_deserializing)]
//receiver: Receiver<NetherPortalText>,
impl NetherPortalText {}

pub struct NetherPortal<F: Future> {
    text: SPromise<NetherPortalText, F>,
    images: BTreeMap<String, SPromise<Imager, F>>,
}

pub struct NetherPortals<F: Future> {
    nether_portals: BTreeMap<String, NetherPortal<F>>,
    nether_portal_text_comm: Communicator<NetherPortalText>,
    imager_comm: Communicator<Imager>,
}

impl<F: Future> NetherPortals<F> {
    pub fn npt_sender_clone(&self) -> Sender<NetherPortalText> {
        self.nether_portal_text_comm.downloader_sender_clone()
    }
    pub fn imager_sender_clone(&self) -> Sender<Imager> {
        self.imager_comm.downloader_sender_clone()
    }
}
