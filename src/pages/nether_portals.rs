use crate::{
    images::Imager,
    pages::portals::{NetherPortalText, NetherPortals},
    thread_tools::Communicator,
};
use std::future::Future;

fn check_promises() {}

fn batch_fetch() {}
fn fetch_all_nether_portals(
    nether_portal_comm: Communicator<NetherPortalText>,
    imager_comm: Communicator<Imager>,
) {
    std::thread::spawn(|| {});
}

pub fn nether_portals_page<F: Future>(nether_portals: &mut NetherPortals<F>) {
    // fetch_all_nether_portals();
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
