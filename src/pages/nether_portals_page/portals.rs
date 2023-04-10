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

#[derive(Deserialize, Serialize, Default, Debug)]
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
impl PortalText {
    pub fn to_btree(&self) -> BTreeMap<String, String> {
        // Serialize pt(PortalText) to json
        let pt_as_json = serde_json::to_value(&self).unwrap();

        // Create a btree for storage
        let mut new_btree: BTreeMap<String, String> = BTreeMap::new();

        // Iter through json as an object, Convert each field to a string
        pt_as_json
            .as_object()
            .unwrap()
            .iter()
            .for_each(|(key, value)| {
                new_btree.insert(key.clone(), value.to_string());
            });

        new_btree
    }
    pub fn from_btree(btree: &BTreeMap<String, String>) -> Result<PortalText, MagicError> {
        let portal_text = PortalText {
            xcord: btree["Xcord"].parse::<i32>()?,
            ycord: btree["Ycord"].parse::<i32>()?,
            zcord: btree["Zcord"].parse::<i32>()?,
            locale: btree["Locale"].clone().replace('"', ""),
            owner: btree["Owner"].clone().replace('"', ""),
            notes: btree["Notes"].clone().replace('"', ""),
            true_name: btree["True_Name"].clone().replace('"', ""),
        };

        Ok(portal_text)
    }
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
    pub fn build_from(id: i32, overworld: PortalText, nether: PortalText) -> Self {
        Self {
            id,
            overworld,
            nether,
            username: String::default(),
        }
    }
    //pub fn convert_np(np: NetherPortal) -> NetherPortalText{
    //    let npt = NetherPortalText {
    //        id: 0,
    //        overworld
    //    }
    //}
}

type F = Box<dyn Future<Output = ()> + Unpin>;
type PortalTextBTree = BTreeMap<String, String>;
// Images should be stored by keys with their name
pub struct NetherPortal {
    original_id: i32,
    portal_text: SPromise<PortalText, F>,
    as_btree: PortalTextBTree,
    images: BTreeMap<String, SPromise<Imager, F>>,
    image_position: String,
    image_list_request: SPromise<Option<String>, Box<dyn Future<Output = ()> + Unpin>>,
}

impl NetherPortal {
    // IMAGE FUNCTIONS

    // Getters
    pub fn image_pos_ref(&self) -> &String {
        &self.image_position
    }

    pub fn image_pos_mut(&mut self) -> &mut String {
        &mut self.image_position
    }

    pub fn images_ref(&self) -> &BTreeMap<String, SPromise<Imager, F>> {
        &self.images
    }

    pub fn images_mut(&mut self) -> &mut BTreeMap<String, SPromise<Imager, F>> {
        &mut self.images
    }

    pub fn img_list_req_ref(
        &self,
    ) -> &SPromise<Option<String>, Box<dyn Future<Output = ()> + Unpin>> {
        &self.image_list_request
    }

    // Setters
    pub fn set_image_pos(&mut self) -> Option<()> {
        //! initialize the image position if its empty. None== there is no key;
        if self.image_position == String::default() {
            for (key, imager) in self.images.iter() {
                imager.spromise_ref().as_ref()?.ready()?;
                self.image_position = key.clone();
                return Some(());
            }
        }
        None
    }
}

impl NetherPortal {
    pub fn get_id(&self) -> i32 {
        self.original_id
    }
    pub fn add_portal_text(&mut self, pt: PortalText) {
        self.portal_text = SPromise::make_no_promise(pt);
    }
    pub fn set_pt(&mut self) -> Result<(), MagicError> {
        //! Set portal_text with converted NetherPortal.as_btree values.
        let pt = PortalText::from_btree(&self.as_btree)?;
        self.portal_text.set_value(pt);

        Ok(())
    }

    // BTree Stuff
    pub fn set_as_btree(&mut self) {
        //! Create a Btree from PortalText and set it to (as_btree)'s value
        self.as_btree = self.portal_text.ref_value().to_btree();
    }
    pub fn btree_ref(&self) -> &PortalTextBTree {
        &self.as_btree
    }
    pub fn btree_mut(&mut self) -> &mut PortalTextBTree {
        &mut self.as_btree
    }

    // Checkers
    pub fn is_empty(&self) -> bool {
        self.as_btree.is_empty()
    }
}

#[derive(Default)]
struct Keys {
    keys: Vec<String>,
    index: usize,
}
impl Keys {
    pub fn set_keys(&mut self, keys: Vec<String>) {
        self.keys = keys;
    }
    pub fn set_pos(&mut self, index: usize) {
        self.index = index;
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn current(&self) -> Option<String> {
        // Keys[index] == Gives position inside Vec
        self.keys.get(self.index).cloned()
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

pub type NetherPortalBTree = BTreeMap<String, NetherPortal>;

// The Keys of NetherPortals BTreeMap members should be the PortalText.true_name
pub struct NetherPortals {
    overworld: BTreeMap<String, NetherPortal>,
    ow_position: Keys,
    nether: BTreeMap<String, NetherPortal>,
    nether_position: Keys,

    // Channels
    nether_portal_text_comm: Communicator<NetherPortalText>,
    imager_comm: Communicator<Imager>, // Imager should be a Vec of Imager(s)

    // Misc
    mutate: bool,
    text_request: SPromise<Option<String>, Box<dyn Future<Output = ()> + Unpin>>,
}

impl NetherPortals {
    pub fn default() -> Self {
        Self {
            overworld: BTreeMap::new(),
            nether: BTreeMap::new(),
            nether_portal_text_comm: Communicator::new(),
            imager_comm: Communicator::new(),
            ow_position: Keys::default(),
            nether_position: Keys::default(),
            mutate: bool::default(),
            text_request: SPromise::make_no_promise(None),
        }
    }

    // Getters
    pub fn text_request_ref(
        &self,
    ) -> &SPromise<Option<String>, Box<dyn Future<Output = ()> + Unpin>> {
        &self.text_request
    }
    pub fn text_request_mut(
        &mut self,
    ) -> &mut SPromise<Option<String>, Box<dyn Future<Output = ()> + Unpin>> {
        &mut self.text_request
    }
    pub fn set_text_request(
        &mut self,
        spromise: SPromise<Option<String>, Box<dyn Future<Output = ()> + Unpin>>,
    ) {
        self.text_request = spromise;
    }

    pub fn overworld_mut(&mut self) -> &mut NetherPortalBTree {
        &mut self.overworld
    }
    pub fn overworld_ref(&self) -> &NetherPortalBTree {
        &self.overworld
    }
    pub fn nether_mut(&mut self) -> &mut NetherPortalBTree {
        &mut self.nether
    }
    pub fn nether_ref(&self) -> &NetherPortalBTree {
        &self.nether
    }
    pub fn get_mutate(&self) -> bool {
        self.mutate
    }

    // Keys Getters
    pub fn get_ow_pos(&self) -> Option<String> {
        self.ow_position.current()
    }
    pub fn get_neth_pos(&self) -> Option<String> {
        self.nether_position.current()
    }

    // Setters
    pub fn set_mutate(&mut self, booly: bool) -> bool {
        self.mutate = booly;

        self.mutate
    }

    // Key Setters
    pub fn set_ow_pos(&mut self, keys: Vec<String>) {
        self.ow_position.set_keys(keys);
    }
    pub fn set_neth_pos(&mut self, keys: Vec<String>) {
        self.nether_position.set_keys(keys);
    }

    // Increment&Decrement OverWorld Position
    pub fn ow_pos_up(&mut self) {
        let index = self.ow_position.get_index();
        if index < self.ow_position.len() - 1 {
            self.ow_position.set_pos(index + 1)
        }
    }
    pub fn ow_pos_down(&mut self) {
        let index = self.ow_position.get_index();
        if index > 0 {
            self.ow_position.set_pos(index - 1)
        }
    }

    // Increment&Decrement Nether Position
    pub fn neth_pos_up(&mut self) {
        let index = self.nether_position.get_index();
        if index < self.nether_position.len() - 1 {
            self.nether_position.set_pos(index + 1);
        }
    }
    pub fn neth_pos_down(&mut self) {
        let index = self.nether_position.get_index();
        if index > 0 {
            self.nether_position.set_pos(index - 1);
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
    pub fn quick_portal(&self) -> &PortalText {
        self.overworld
            .get("Luke SpawnPoint")
            .unwrap()
            .portal_text
            .quick_value()
        ////-> &NetherPortalText {
        //let mut string = String::default();
        //self.overworld.iter().for_each(|(key, value)| {
        //    //let y = value.portal_text.quick_value();
        //    //let z = y.quick_inner().as_ref().unwrap().ready().unwrap();
        //    //println!("key:|{}| --- value:|{:?}|", key, y);
        //    if string == String::default() {
        //        string = key.clone();
        //    }
        //});

        //string
        //let x = self.overworld.get(key)
    }
    // ================

    // Receivers
    pub fn npt_receiver(&self) -> &Receiver<NetherPortalText> {
        &self.nether_portal_text_comm.downloader_receiver()
    }

    pub fn consume_npt_helper(
        original_id: i32,
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
                    original_id,
                    portal_text: SPromise::make_no_promise(pt),
                    as_btree: BTreeMap::new(),
                    images: BTreeMap::new(),
                    image_position: String::default(),
                    image_list_request: SPromise::make_no_promise(None),
                };
                np_list.insert(key, overworld_np);
            }
        }
    }

    pub fn consume_npt(&mut self, mut npt: NetherPortalText) {
        //! Given a moved NetherPortalText struct: (npt)
        //!
        //! Take its members and give them to NetherPortals struct

        // Take/Append OverWorld (use mem::take to avoid Partial Move|| maybe rust will update compiler to fix this?)
        let og_id = npt.id;

        //let key = npt.ow_true_name();
        let key = npt.ow_true_name();
        let overworld = mem::take(&mut npt.overworld);
        Self::consume_npt_helper(og_id, key, &mut self.overworld, overworld);

        // Take/Append Nether
        let key = npt.nether_true_name();
        let nether = npt.nether;
        Self::consume_npt_helper(og_id, key, &mut self.nether, nether);
    }

    //pub fn add_imager_to_nether_portal(&mut self, key: String, imager: Imager) {
    //    if self.nether
    //}

    pub fn try_update_npt(&mut self) -> Result<(), MagicError> {
        while let Ok(nether_portal_text) = self.npt_receiver().try_recv() {
            self.consume_npt(nether_portal_text);
        }
        Ok(())
    }
}
