mod windows;

pub mod borkcraft;
pub mod pages;

pub use borkcraft::*;
use chrono::{Timelike, Utc};

mod increment {
    pub struct Inc {
        counter: i64,
    }
    impl Inc {
        pub fn up(&mut self) -> i64 {
            self.counter = self.counter + 1;
            self.counter
        }
        //pub fn down(&mut self) -> i64 {
        //    self.counter = self.counter - 1;
        //    self.counter
        //}
        pub fn reset(&mut self) -> i64 {
            self.counter = 0;
            self.counter
        }
        pub fn new() -> Inc {
            Inc { counter: 0 }
        }
    }
}

type MagicError = Box<dyn std::error::Error>;

fn _get_tokio_runtime() -> tokio::runtime::Runtime {
    let rt = tokio::runtime::Runtime::new().unwrap();
    _ = rt.enter();
    rt
}

fn _try_access<T>(
    try_me: &std::sync::Arc<std::sync::Mutex<T>>,
    f: impl FnOnce(std::sync::MutexGuard<T>),
) -> Result<(), MagicError> {
    match try_me.try_lock() {
        Ok(access) => {
            f(access);
            Ok(())
            // Do magic
        }
        Err(_) => Err(crate::err_tools::ErrorX::new_box(
            "try_access was used currently, try again later...",
        )),
    }
}

fn _try_access_experimental<T>(
    try_me: &std::sync::Arc<std::sync::Mutex<T>>,
    f: impl FnOnce(std::sync::MutexGuard<T>),
) -> Result<std::sync::Arc<std::sync::Mutex<T>>, MagicError> {
    match try_me.try_lock() {
        Ok(access) => {
            f(access);
            return Ok(std::sync::Arc::clone(try_me));
            // Do magic
        }
        Err(_) => Err(crate::err_tools::ErrorX::new_box(
            "try_access was used currently, try again later...",
        )),
    }
}

fn time_of_day() -> String {
    // "Hour:{} Minute:{} Second:{}",
    let time = Utc::now();
    format!("{}:{} -- {}", time.hour(), time.minute(), time.second())
}

pub mod string_tools {
    fn quick_maker(amount: usize, character: &str) -> String {
        let mut s = String::default();
        for _ in 0..amount {
            s = format!("{}{}", s, character)
        }
        s
    }

    pub fn newliner(amount: usize) -> String {
        quick_maker(amount, "\n")
    }

    pub fn tabber(amount: usize) -> String {
        quick_maker(amount, "\t")
    }
}

pub mod err_tools {
    #[derive(Debug)]
    pub struct ErrorX {
        details: String,
    }

    impl ErrorX {
        pub fn _new(msg: &str) -> ErrorX {
            ErrorX {
                details: msg.to_string(),
            }
        }
        pub fn new_box(msg: &str) -> Box<ErrorX> {
            Box::new(ErrorX {
                details: msg.to_string(),
            })
        }
    }

    impl std::fmt::Display for ErrorX {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.details)
        }
    }

    impl std::error::Error for ErrorX {
        fn description(&self) -> &str {
            &self.details
        }
    }
}

pub mod url_tools {
    use serde::Serialize;

    pub fn to_vec8(cereal: &impl Serialize) -> Vec<u8> {
        serde_json::to_vec(cereal).unwrap()
    }

    pub enum Routes {
        Login,
        Logout,
        AddNetherPortalText,
        UpdateNetherPortalText,
        SaveImageText,
        SaveImage,
        DeleteImage,
        DeleteClientImage,
        GetNetherPortalBunch,
        GetNetherPortalImageNames,
        GetNetherPortalImage,
        AccessRights,
        SessionTimeLeft,
    }
    impl Routes {
        fn make(&self) -> String {
            match self {
                Routes::Login => "/login",
                Routes::Logout => "/logout",
                Routes::AddNetherPortalText => "/addnetherportaltext",
                Routes::UpdateNetherPortalText => "/savenetherportaltextchanges",
                Routes::SaveImageText => "/addnetherportalimagedetails",
                Routes::SaveImage => "/saveimage",
                Routes::DeleteImage => "/deleteimage",
                Routes::DeleteClientImage => "/deleteimagefromclient",
                Routes::GetNetherPortalBunch => "/getnetherportalstextinformation",
                Routes::GetNetherPortalImage => "/getnetherportalimage",
                Routes::GetNetherPortalImageNames => "/getnetherportalimagenames",
                Routes::AccessRights => "/getaccessrights",
                Routes::SessionTimeLeft => "/sessiontimeleft",
            }
            .to_string()
        }
    }
    pub struct Urls {
        url: String,
    }
    impl Urls {
        pub fn default(route: Routes) -> String {
            //! Provides a url with default port number and dns ipv4 thingy
            Urls::new(aws_public_dns(), text_server_port()).url(route)
        }
        pub fn default_i(route: Routes) -> String {
            Urls::new("localhost".to_string(), "1234".to_string()).url(route)
        }
        pub fn new(public_dns: String, port: String) -> Urls {
            Urls {
                url: format!("http://{}:{}", public_dns, port),
            }
        }
        pub fn url(&self, route: Routes) -> String {
            format!("{}{}", self.url, route.make())
        }
    }
    pub fn aws_public_dns() -> String {
        // format!("put ec2 aws ipv4/dns here!")
        // format!("ec2-3-101-115-101.us-west-1.compute.amazonaws.com")
        format!("localhost")
    }
    pub fn text_server_port() -> String {
        format!("8334")
    }
}

pub mod eframe_tools {
    use crate::string_tools::newliner;
    use eframe::egui::{ScrollArea, Ui};
    pub fn scroll_and_vert(ui: &mut Ui, id: impl std::hash::Hash, f: impl FnOnce(&mut Ui)) {
        ScrollArea::vertical()
            .id_source(id)
            .show(ui, |ui| ui.horizontal_wrapped(|ui| f(ui)));
    }
    pub fn text_edit(ui: &mut Ui, writable: &mut String) {
        ui.add(eframe::egui::TextEdit::singleline(writable));
    }
    pub fn space_vert(amount: usize, ui: &mut Ui) {
        //! Add vertical space using newlines
        ui.label(format!("{}", newliner(amount)));
    }
}

#[cfg(test)]
mod tests {
    //use super::{
    //    pages::login::LoginForm,
    //    sessions::SessionInfo,
    //    url_tools::{Routes, Urls},
    //};

    //#[test]
    //fn login_to_server() {
    //    fn get_access_rights(
    //        username: &String,
    //        url: String,
    //    ) -> Result<ureq::Response, ureq::Error> {
    //        let url = &format!("{}?username={}", url, username);
    //        let result = ureq::get(url).call();

    //        result
    //    }
    //    let login_form = &LoginForm{ username: String::from("luke@gmail.com"), String::from("1234"), String::from(""), sender: None);
    //    // send LoginForm to Server
    //    let response = ureq::post(&Urls::default(Routes::Login))
    //        .send_bytes(&serde_json::to_vec(login_form).unwrap())
    //        .unwrap();

    //    // Convert Response and assign it to session_info(SessionInfo)
    //    let _sessinfo = SessionInfo::response_to_session_info(response).unwrap();

    //    let response =
    //        get_access_rights(&login_form.username, Urls::default(Routes::AccessRights)).unwrap();
    //    let mut hasher: std::collections::HashMap<String, Vec<String>> =
    //        serde_json::from_str(&response.into_string().unwrap()).unwrap();
    //    let vecker = hasher.remove("access_rights").unwrap();
    //    println!("{:?}", vecker);
    //    panic!("forced panic")
    //}
}
