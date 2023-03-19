use std::sync::mpsc::Receiver;

use crate::{
    time_of_day,
    url_tools::{Routes, Urls},
    windows::client_windows::{GenericWindow, Loglet},
    MagicError,
};
use serde_derive::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

#[derive(Default)]
pub struct SessionInfo {
    pub key: String,
    pub session_time: SessionTime,
    pub is_logged_in: bool,
    pub access_rights: Vec<String>,
    pub display: GenericWindow,
    pub receiver: Option<Receiver<(SessionTime, Loglet)>>,
}

impl SessionInfo {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }

    pub fn new(receiver: Option<Receiver<(SessionTime, Loglet)>>) -> Self {
        Self {
            key: String::default(),
            session_time: SessionTime::default(),
            is_logged_in: bool::default(),
            access_rights: Vec::default(),
            display: GenericWindow::default(),
            receiver,
        }
    }

    pub fn session_time_to_session_info(
        session_time: SessionTime,
    ) -> Result<SessionInfo, MagicError> {
        Ok(SessionInfo {
            key: session_time.key.clone(),
            session_time,
            is_logged_in: true,
            access_rights: Vec::default(),
            display: GenericWindow::default(),
            receiver: None,
        })
    }

    pub fn consume(&mut self, si: SessionInfo) {
        //! Takes in a Session Information and properly appends all necessary information
        //! without overriding the already present maybe Option<Receiver<>>
        self.key = si.key;
        self.session_time = si.session_time;
        self.is_logged_in = si.is_logged_in;
        self.access_rights = si.access_rights;
        self.display = si.display;
    }

    pub fn display_namae(mut self, name: &str) -> Self {
        // Change the name of the display
        // Easy of Access function
        self.display.namae(name);
        self
    }

    fn is_session_over(time: &TimeTime) -> bool {
        let one = String::from("1");
        if time.hour < one && time.minute < one && time.second < one {
            false
        } else {
            true
        }
    }

    pub fn try_update(&mut self) {
        //! Check for updates from: (Session Time Updater Thread)
        if let Some(receiver) = &self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok((st, loglet)) => {
                        // update user login status
                        self.is_logged_in = Self::is_session_over(&st.time);
                        self.session_time = st;
                        self.display.log.push(loglet);
                    }
                    Err(_) => break,
                }
            }
        } else {
            println!("NO RECEIVER")
        }
    }
    pub fn block_update(&mut self) {
        if let Some(receiver) = &self.receiver {
            let (st, loglet) = receiver.recv().unwrap();
            // update user login status
            self.is_logged_in = Self::is_session_over(&st.time);
            self.session_time = st;
            self.display.log.push(loglet);
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct SessionTime {
    pub key: String,
    pub time: TimeTime,
}

#[derive(Deserialize, Serialize, Default)]
pub struct TimeTime {
    pub hour: String,
    pub minute: String,
    pub second: String,
}

impl TimeTime {
    pub fn formatted(&self) -> String {
        format!(
            "Session Time Left: ({}:{} -- {})",
            self.hour, self.minute, self.second
        )
    }
}

#[derive(Serialize)]
struct Key<'a> {
    key: &'a String,
}

fn fetch_session_time(key: &String) -> Result<ureq::Response, ureq::Error> {
    ureq::post(&Urls::default(Routes::SessionTimeLeft)).send_json(Key { key })
}

pub fn current_session_time(
    sender: &Sender<(SessionTime, Loglet)>,
    validation_key: String,
) -> Result<String, MagicError> {
    // Get the sesssion time from the server
    let response = fetch_session_time(&validation_key)?;

    // Convert response to time object
    let session_time: SessionTime = response.into_json()?;

    // Create information to be sent by (Sender)
    let loglet = Loglet::new("Update", &session_time.time.formatted(), &time_of_day());
    let validation_key = session_time.key.clone();

    // Send
    sender.send((session_time, loglet)).unwrap();

    // Return Key to be used on next iteration of calling scope's loop
    Ok(validation_key)
}
