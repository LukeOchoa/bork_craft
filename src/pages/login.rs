use crate::{
    eframe_tools::text_edit,
    sessions::{SessionInfo, ThreadSessionInfo},
    try_access,
    url_tools::{Routes, Urls},
    MagicError,
};
use eframe::{egui::Ui, epaint::ahash::HashMap};
use serde_derive::Serialize;
use serde_json::to_vec;

type AccessRights = Vec<String>;

#[derive(Default, Serialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub session_key: String,
}

impl LoginForm {
    pub fn default() -> Self {
        Default::default()
    }
    pub fn new(u: &str, p: &str, s: &str) -> LoginForm {
        let x = String::from;
        LoginForm {
            username: x(u),
            password: x(p),
            session_key: x(s),
        }
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

    // Convert Response
    let sess_info = SessionInfo::response_to_session_info(response)?;

    // get access rights
    let response = get_access_rights(&login_form.username, Urls::default(Routes::AccessRights))?;
    // Convert Response
    let access_rights: AccessRights = convert_access_rights_resp(response)?;

    // Combine the response data
    let sess_info = SessionInfo {
        access_rights,
        ..sess_info
    };

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

pub fn login_page(session_info: &ThreadSessionInfo, login_form: &mut LoginForm, ui: &mut Ui) {
    _ = try_access(session_info, |mut sess_info| {
        if !sess_info.is_logged_in {
            show_login_form(ui, login_form);
        }
        if ui.button("Login").clicked() {
            match handle_login(login_form) {
                Ok(sessinfo) => {
                    *sess_info = sessinfo;
                }
                Err(_error) => {}
            }
        }
    });
}

//let subfn = || -> Result<(), MagicError> {
//    // send LoginForm to Server
//    let response = ureq::post(&Urls::default(Routes::Login))
//        .send_bytes(&to_vec(&login_form).unwrap())?;

//    // Convert Response and assign it to session_info(SessionInfo)
//    *sess_info = SessionInfo::response_to_session_info(response)?;

//    // get access rights
//    let response =
//        get_access_rights(&login_form.username, Urls::default(Routes::AccessRights))?;
//    sess_info.access_rights = convert_access_rights_resp(response)?;

//    Ok(())
//};
