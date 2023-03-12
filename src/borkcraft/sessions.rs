use crate::windows::client_windows::GenericWindow;

#[derive(Default)]
pub struct SessionInfo {
    pub key: String,
    pub time: TimeTime,
    pub is_logged_in: bool,
    pub access_rights: Vec<String>,
    pub display: GenericWindow,
}

#[derive(Default)]
pub struct TimeTime {
    pub hour: String,
    pub minute: String,
    pub second: String,
}

impl SessionInfo {
    pub fn default() -> Self {
        let instance: Self = Default::default();
        instance
    }
}
