use super::portals::NetherPortalText;
use crate::{
    err_tools::ErrorX,
    time_of_day,
    url_tools::{Routes, Urls},
    windows::client_windows::Loglet,
    MagicError,
};

use std::{
    error::Error,
    {collections::HashMap, sync::mpsc::Sender},
};
use tokio::runtime::Runtime;

use std::future::Future;
type NetherPortalTextBunch = HashMap<String, NetherPortalText>;

fn batch_fetch_nether_portal_text(offset: i32, limit: i32) -> Result<ureq::Response, ureq::Error> {
    // TODO Change this...?
    let route = &format!("http://localhost:3001/getnetherportalstextinformation");
    //let route = Urls::default(Routes::GetNetherPortalBunch);
    let url = &format!("{}?orderby={}&limit={}", route, offset, limit);
    println!("url: {}", url);

    ureq::get(url).call()
}

fn estimate_nether_portals_count() -> Result<i32, MagicError> {
    // Takes a relative count from the database through the Rest API

    // Fetch Request
    let response = ureq::get(&Urls::default(Routes::EstimatedAmountNetherPortals)).call()?;

    // Convert to Map<Key: String, Value: i32>
    let estimate: HashMap<String, i32> = serde_json::from_reader(response.into_reader())?;

    // Convert To i32
    let estimate = *estimate.get("count").unwrap();

    // Return
    Ok(estimate)
}

fn response_to_nether_portal_bunch(
    response: ureq::Response,
) -> Result<NetherPortalTextBunch, MagicError> {
    let reader = response.into_reader();
    let bunch = serde_json::from_reader(reader)?;
    Ok(bunch)
}

async fn sync_fetch_all_nether_portals(limit: i32) -> Result<NetherPortalTextBunch, MagicError> {
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

async fn send_nether_portal_texts(
    nether_portal_sender: Sender<NetherPortalText>,
    mut npt_bunchs: NetherPortalTextBunch,
) -> Result<(), MagicError> {
    // send each npt with a channel sender to the main thread's NetherPortals(data struct)
    for (_, nether_portal_text) in npt_bunchs.drain() {
        nether_portal_sender.send(nether_portal_text)?
    }

    Ok(())
}

use std::pin::Pin;
type FetchFunction =
    Pin<Box<dyn Future<Output = Result<HashMap<String, NetherPortalText>, Box<dyn Error>>> + Send>>;

fn get_fetcher_fn(estimate: i32, limit: i32) -> Result<FetchFunction, MagicError> {
    // If the estimate of rows is larger than 100 use multi_threaded fn; TODO
    if estimate > 100 {
        return Err(ErrorX::new_box(
            "Create multi-threaded async func, estimate to large...",
        ));
    }
    // If the estimate is less than 100: use sync fetcher
    if estimate < 100 && estimate > 0 {
        return Ok(Box::pin(sync_fetch_all_nether_portals(limit)));
    }

    // No one uses this app lol, thats why there arent any rows
    Err(ErrorX::new_box("There are no rows in the database"))
}

pub fn download_nether_portals(
    nether_portal_sender: Sender<NetherPortalText>,
    err_msg_sender: Sender<Loglet>,
    runtime: &Runtime,
) {
    runtime.spawn(async move {
        let result = async move {
            // Get estimate of how many rows server has
            let estimate = estimate_nether_portals_count()?;

            // Choose an appropriate function based on estimate
            let npt_fetcher = get_fetcher_fn(estimate, 5)?;

            // Get all Nether_Portal_Text_Bunchs (Ik its 'bunches')
            let npt_bunchs = npt_fetcher.await?;

            // Send them all down the provided channel
            send_nether_portal_texts(nether_portal_sender, npt_bunchs).await?;

            // Return
            Ok::<(), MagicError>(())
        }
        .await;

        // Send any error down the channel
        if let Err(error) = result {
            err_msg_sender
                .send(Loglet::new("Error", &error.to_string(), &time_of_day()))
                .unwrap();
        }
    });
}
