use std::{collections::BTreeMap, future::Future};

use serde_derive::{Deserialize, Serialize};

use crate::{
    images::Imager,
    thread_tools::{Communicator, SPromise},
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
    nether_portal_comm: Communicator<NetherPortalText>,
    imager_comm: Communicator<Imager>,
}
