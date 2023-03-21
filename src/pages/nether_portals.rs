use crate::{
    images::Imager,
    pages::portals::{NetherPortalText, NetherPortals},
    thread_tools::Communicator,
    time_of_day,
    url_tools::{Routes, Urls},
    windows::{client_windows::Loglet, error_messages::ErrorMessage},
    MagicError,
};
use std::{
    sync::mpsc::Sender,
    {collections::HashMap, future::Future},
};
use tokio::runtime::Runtime;

// Custom Convenience Types
type NetherPortalTextBunch = HashMap<String, NetherPortalText>;

fn check_promises() {}

fn batch_fetch_nether_portal_text(offset: i32, limit: i32) -> Result<ureq::Response, ureq::Error> {
    let route = Urls::default(Routes::GetNetherPortalBunch);
    let url = &format!("{}offset={}&limit={}", route, offset, limit);

    ureq::get(url).call()
}

fn estimate_nether_portals_count() -> Result<i32, MagicError> {
    // Takes a relative count from the database through the Rest API

    // Fetch Request
    let response = ureq::get(&Urls::default(Routes::EstimatedAmountNetherPortals)).call()?;

    // Convert Request To i32
    let estimate = serde_json::from_reader(response.into_reader())?;

    Ok(estimate)
}

fn response_to_nether_portal_bunch(
    response: ureq::Response,
) -> Result<NetherPortalTextBunch, MagicError> {
    let reader = response.into_reader();
    let bunch = serde_json::from_reader(reader)?;
    Ok(bunch)
}

fn sync_fetch_all_nether_portals(limit: i32) -> Result<NetherPortalTextBunch, MagicError> {
    let offset = -1; // Starting point for scrapping the db table

    // The One-True Overlord of Nether Portal Bunchs: Portahlus Bunchaccous!
    let mut master_bunch = NetherPortalTextBunch::new();

    // Loop until you have all bunchs from rest api
    loop {
        // Get a bunch of nether portals from Rest API as response
        let response = batch_fetch_nether_portal_text(offset, limit)?;

        // Convert Response
        let bunch: NetherPortalTextBunch = response_to_nether_portal_bunch(response)?;

        // if the bunch is empty or bunch is less than(<limit) limit, Fetching is finished
        if bunch.is_empty() || (bunch.len() as i32) < limit {
            master_bunch.extend(bunch);
            break;
        }

        // Append to Master Bunch
        master_bunch.extend(bunch);
    }

    // Return the master to its harvest!
    Ok(master_bunch)
}

async fn fetch_all_nether_portals() -> Result<NetherPortalTextBunch, MagicError> {
    let estimate = estimate_nether_portals_count()?;
    let limit = 5; // Buffer Size

    let nether_portal_text_bunchs;

    // If estimate is larger(>100) than 100: True => Do multithreaded; False => Sync
    nether_portal_text_bunchs = if estimate > 100 {
        NetherPortalTextBunch::new()
        // TODO
    } else if estimate > 0 {
        sync_fetch_all_nether_portals(limit)?
    } else {
        // There are no Rows in the database.
        // The Big Sadge (because there are no entrys.. because no one uses this lol)
        NetherPortalTextBunch::new()
    };

    Ok(nether_portal_text_bunchs)
}

async fn send_nether_portal_texts(
    nether_portal_sender: Sender<NetherPortalText>,
) -> Result<(), MagicError> {
    let mut nether_portal_text_bunchs = fetch_all_nether_portals().await?;

    for (_, nether_portal_text) in nether_portal_text_bunchs.drain() {
        nether_portal_sender.send(nether_portal_text)?
    }

    Ok(())
}

fn download_nether_portals(
    nether_portal_sender: Sender<NetherPortalText>,
    imager_sender: Sender<Imager>,
    err_msg_sender: Sender<Loglet>,
    runtime: &Runtime,
) {
    //let nether_portal_sender = nether_portal_comm.downloader_sender_clone();
    //let err_msg_sender = err_msg.sender_clone();
    runtime.spawn(async move {
        let result = send_nether_portal_texts(nether_portal_sender).await;
        if let Err(error) = result {
            err_msg_sender
                .send(Loglet::new("Error", &error.to_string(), &time_of_day()))
                .unwrap();
        }
    });
}

pub fn nether_portals_page<F: Future>(
    nether_portals: &mut NetherPortals<F>,
    err_msg: &ErrorMessage,
    runtime: &Runtime,
) {
    download_nether_portals(
        nether_portals.npt_sender_clone(),
        nether_portals.imager_sender_clone(),
        err_msg.sender_clone(),
        runtime,
    );
    check_promises();
}

// SPromise
// You have a value
// when you need a new value
//
// 1) Run the future
// 2) Give sender to the future
// 3) Pass future to a thread/tokio
// 4) Use a spinner in place of the loading data
// 5) Once promise if fulfilled, take() from Option
//     and give it to SPromise.value
//      making SPromise.some_value as None
