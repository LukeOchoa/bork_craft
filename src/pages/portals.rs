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

impl NetherPortalText {}

type F = Box<dyn Future<Output = ()> + Unpin>;
pub struct NetherPortal {
    text: SPromise<NetherPortalText, F>,
    images: BTreeMap<String, SPromise<Imager, F>>,
}

pub struct NetherPortals {
    nether_portals: BTreeMap<String, NetherPortal>,
    nether_portal_text_comm: Communicator<NetherPortalText>,
    imager_comm: Communicator<Imager>,
}

impl NetherPortals {
    pub fn default() -> Self {
        Self {
            nether_portals: BTreeMap::new(),
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

    // Receivers
    pub fn npt_receiver(&self) -> &Receiver<NetherPortalText> {
        &self.nether_portal_text_comm.downloader_receiver()
    }

    // General
    pub fn is_npt_empty(&self) -> bool {
        //! Checks if nether_portals_text is empty
        self.nether_portals.is_empty()
    }
    pub fn quick(&self) -> String {
        let total = format!("{}", self.nether_portals.is_empty());

        total
    }
    pub fn add_to_nether_portal(
        &mut self,
        key: String,
        some_npt: Option<NetherPortalText>,
        some_imager: Option<Imager>,
    ) -> Option<()> {
        //! SPLIT THIS INTO THRE FUNCTIONS
        //!
        //! Wrapper over HashMap method: "HashMap.insert()".
        //!
        //! Allows for either/both values to be inserted into an existing key.
        //!
        //! If key does not exist a new entry will be created AS LONG AS some_npt AND some_imager ARE NOT None.

        // If a key already exists/ override values
        if self.nether_portals.contains_key(&key) {
            let nether_portal = self.nether_portals.get_mut(&key).unwrap();
            if let Some(npt) = some_npt {
                nether_portal.text.add_value(npt);
            }
            if let Some(imager) = some_imager {
                nether_portal
                    .images
                    .insert(imager.get_name(), SPromise::make_no_promise(imager));
            }
        } else {
            // Set up BTreeMap
            let imager = some_imager?;
            let mut btm = BTreeMap::new();
            btm.insert(imager.get_name(), SPromise::make_no_promise(imager));

            // Compose NetherPortal
            let value = NetherPortal {
                text: SPromise::make_no_promise(some_npt?),
                images: btm,
            };

            // Insertion
            self.nether_portals.insert(key, value);
        }
        None
    }
    pub fn try_update_npt(&mut self) -> Result<(), MagicError> {
        while let Ok(nether_portal_text) = self.npt_receiver().try_recv() {
            self.nether_portals.insert(
                format!("{}", nether_portal_text.id),
                NetherPortal {
                    text: SPromise::test(nether_portal_text),
                    images: BTreeMap::new(),
                },
            );
        }
        Ok(())
    }
}
