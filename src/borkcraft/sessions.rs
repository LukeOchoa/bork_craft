use crate::{windows::client_windows::GenericWindow, MagicError};
use serde_derive::{Deserialize, Serialize};

pub type ThreadSessionInfo = std::sync::Arc<std::sync::Mutex<SessionInfo>>;

#[derive(Default)]
pub struct SessionInfo {
    pub session_time: SessionTime,
    pub is_logged_in: bool,
    pub access_rights: Vec<String>,
    pub display: GenericWindow,
}

impl SessionInfo {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }

    pub fn response_to_session_info(response: ureq::Response) -> Result<SessionInfo, MagicError> {
        let session_time: SessionTime = response.into_json()?;
        Ok(SessionInfo {
            session_time,
            is_logged_in: true,
            access_rights: Vec::default(),
            display: GenericWindow::default(),
        })
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
