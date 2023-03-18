use crate::{
    eframe_tools::text_edit,
    err_tools::ErrorX,
    sessions::SessionInfo,
    time_of_day,
    url_tools::{Routes, Urls},
    windows::{
        client_windows::{GenericWindow, Loglet},
        error_messages::ErrorMessage,
    },
    MagicError,
};

use serde_derive::Serialize;
use serde_json::to_vec;

use eframe::egui::Ui;
use std::{collections::HashMap, sync::mpsc::Sender};

type AccessRights = Vec<String>;

#[derive(Default, Serialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub session_key: String,
    #[serde(skip_serializing)]
    pub sender: Option<Sender<String>>,
}

impl LoginForm {
    pub fn default() -> Self {
        Default::default()
    }

    pub fn send(&self, key: String) -> Option<()> {
        //! Send key to through Sender<>
        //! Returns None if there is no Sender
        //!
        //! This uses an infinite loop so it is possible it will block the thread
        //! IF session thread is stuck on a network request
        let sender = self.sender.as_ref()?;
        loop {
            if let Ok(_) = sender.send(key.clone()) {
                break;
            }
        }
        Some(())
    }

    pub fn try_send(&self, key: String) -> Result<(), MagicError> {
        let sender = self.sender.as_ref().ok_or(ErrorX::new_box("No Sender"))?;
        sender.send(key)?;
        Ok(())
    }
}

fn show_login_form(ui: &mut Ui, login_form: &mut LoginForm) {
    ui.label("Username");
    text_edit(ui, &mut login_form.username);
    ui.end_row();

    ui.label("Password");
    text_edit(ui, &mut login_form.password);
    ui.end_row();
}

fn handle_login(login_form: &LoginForm) -> Result<SessionInfo, MagicError> {
    // Send LoginForm to Server
    let response =
        ureq::post(&Urls::default(Routes::Login)).send_bytes(&to_vec(&login_form).unwrap())?;

    // Convert Response & Assign data
    let session_time = response.into_json()?;
    let sess_info = SessionInfo::session_time_to_session_info(session_time)?;

    // get access rights
    let response = get_access_rights(&login_form.username, Urls::default(Routes::AccessRights))?;
    // Convert Response
    let access_rights: AccessRights = convert_access_rights_resp(response)?;

    // Combine the response data
    let sess_info = SessionInfo {
        access_rights,
        ..sess_info
    };

    // Send the new key to the looping session thread; This is an endless loop interally... Reconsider in the future
    login_form.send(sess_info.key.clone()).unwrap();
    println!("SENT!");

    Ok(sess_info)
}

fn get_access_rights(username: &String, url: String) -> Result<ureq::Response, ureq::Error> {
    let url = &format!("{}?username={}", url, username);
    let result = ureq::get(url).call();

    result
}

fn convert_access_rights_resp(response: ureq::Response) -> Result<Vec<String>, MagicError> {
    //! The db server sends its response wrapped in a useless hashmap.
    //!
    //! This function converts the response and removes the usefull data/structure.
    let mut hasher: HashMap<String, Vec<String>> = serde_json::from_str(&response.into_string()?)?;
    Ok(hasher.remove("access_rights").unwrap())
}

pub fn login_page(
    session_info: &mut SessionInfo,
    login_form: &mut LoginForm,
    ui: &mut Ui,
    err_msg: &mut ErrorMessage,
) {
    // Show login form to screen
    if !session_info.is_logged_in {
        show_login_form(ui, login_form);
    }

    // On button click, Try to login to by server: Ok() => Update session, Err() => Update Err Log
    if ui.button("Login").clicked() {
        match handle_login(login_form) {
            Ok(si) => {
                session_info.consume(si);
            }
            Err(error) => GenericWindow::push_loglet(
                &mut err_msg.display,
                Loglet::new("Error", &error.to_string(), &time_of_day()),
            ),
        }
    }
}
