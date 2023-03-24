use std::{
    collections::BTreeMap,
    future::Future,
    sync::mpsc::{Receiver, Sender},
};

use serde_derive::{Deserialize, Serialize};

//windows::client_windows::Loglet,
use crate::{
    images::Imager,
    thread_tools::{Communicator, SPromise},
    MagicError,
};
use std::mem;

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

impl NetherPortalText {
    pub fn nether_true_name(&self) -> String {
        //! Return new String from nether.true_name's field
        self.nether.true_name.clone()
    }
    pub fn ow_true_name(&self) -> String {
        //! Return new String from overworld.true_name's field
        self.overworld.true_name.clone()
    }
}

type F = Box<dyn Future<Output = ()> + Unpin>;

// Images should be stored by keys with their name
pub struct NetherPortal {
    portal_text: SPromise<PortalText, F>,
    images: BTreeMap<String, SPromise<Imager, F>>,
}

impl NetherPortal {
    pub fn add_portal_text(&mut self, pt: PortalText) {
        self.portal_text = SPromise::make_no_promise(pt);
    }
}

// The Keys of NetherPortals BTreeMap members should be the PortalText.true_name
pub struct NetherPortals {
    overworld: BTreeMap<String, NetherPortal>,
    nether: BTreeMap<String, NetherPortal>,
    nether_portal_text_comm: Communicator<NetherPortalText>,
    imager_comm: Communicator<Imager>, // Imager should be a Vec of Imager(s)
}

impl NetherPortals {
    pub fn default() -> Self {
        Self {
            overworld: BTreeMap::new(),
            nether: BTreeMap::new(),
            nether_portal_text_comm: Communicator::new(),
            imager_comm: Communicator::new(),
        }
    }

    // Senders
    pub fn npt_sender_clone(&self) -> Sender<NetherPortalText> {
        self.nether_portal_text_comm.downloader_sender_clone()
    }
    pub fn imager_sender_clone(&self) -> Sender<Imager> {
        self.imager_comm.downloader_sender_clone()
    }

    // Test Function
    pub fn is_nether_empty(&self) -> bool {
        //! Checks if nether is empty
        self.nether.is_empty()
    }
    pub fn is_overworld_empty(&self) -> bool {
        //! Checks if overworld is empty
        self.overworld.is_empty()
    }
    pub fn quickn(&self) -> String {
        format!(
            "Is this BTreeMap empty?: Answer: |{}|",
            self.is_nether_empty()
        )
    }
    pub fn quicko(&self) -> String {
        format!(
            "Is this BTreeMap empty?: Answer: |{}|",
            self.is_overworld_empty()
        )
    }
    // ================

    // Receivers
    pub fn npt_receiver(&self) -> &Receiver<NetherPortalText> {
        &self.nether_portal_text_comm.downloader_receiver()
    }

    pub fn comsume_npt_helper(
        key: String,
        np_list: &mut BTreeMap<String, NetherPortal>,
        pt: PortalText,
    ) {
        match np_list.contains_key(&key) {
            // If Key exists, UPDATE value
            true => {
                np_list.get_mut(&key).unwrap().add_portal_text(pt);
            }
            false => {
                // if Key DOES NOT exist, INSERT new value
                let overworld_np = NetherPortal {
                    portal_text: SPromise::make_no_promise(pt),
                    images: BTreeMap::new(),
                };
                np_list.insert(key, overworld_np);
            }
        }
    }

    pub fn comsume_npt(&mut self, mut npt: NetherPortalText) {
        //! Given a moved NetherPortalText struct: (npt)
        //!
        //! Take its members and give them to NetherPortals struct

        // Take/Append OverWorld (use mem::take to avoid Partial Move|| maybe rust will update compiler to fix this?)
        let key = npt.ow_true_name();
        let overworld = mem::take(&mut npt.overworld);
        Self::comsume_npt_helper(key, &mut self.overworld, overworld);

        // Take/Append Nether
        let key = npt.nether_true_name();
        let nether = npt.nether;
        Self::comsume_npt_helper(key, &mut self.nether, nether);
    }

    //pub fn add_imager_to_nether_portal(&mut self, key: String, imager: Imager) {
    //    if self.nether
    //}

    pub fn try_update_npt(&mut self) -> Result<(), MagicError> {
        while let Ok(nether_portal_text) = self.npt_receiver().try_recv() {
            self.comsume_npt(nether_portal_text);
        }
        Ok(())
    }
}
