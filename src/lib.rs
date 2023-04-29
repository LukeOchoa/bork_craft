mod windows;

pub mod borkcraft;
pub mod images;
pub mod pages;

pub use borkcraft::*;
use chrono::{Timelike, Utc};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use windows::error_messages::ErrorMessage;

use crate::windows::client_windows::Loglet;

// Custom Types (For convenience)
type MagicError = Box<dyn std::error::Error>;
// type MagicSendError = Box<dyn std::error::Error + Send>;

// lifetime sillyness
//fn subfn<'a, 'b, T>(t: &'a Result<T, MagicError>, f: impl FnOnce(&'b MagicError))
//where
//'a: 'b,
//{
//if let Err(err) = t {
//f(err);
//}
//}

// Enums
pub enum Realm {
    Overworld,
    Nether,
}

impl Realm {
    pub fn matcher<T>(&self, overworld: T, nether: T) -> T {
        match self {
            Self::Overworld => overworld,
            Self::Nether => nether,
        }
    }
}

// Traits
pub trait HandleOption<T, X> {
    fn on_none(self, f: impl FnOnce() -> X) -> Option<T>;
}
impl<T, X> HandleOption<T, X> for Option<T> {
    fn on_none(self, f: impl FnOnce() -> X) -> Option<T> {
        if let None = self {
            f();
            return None;
        }
        self
    }
}
pub fn option(mut f: impl FnMut() -> Option<()>) -> Option<()> {
    f()
}
pub trait HandleError<T> {
    fn consume_error(self, err_msg: &mut ErrorMessage);

    fn otherwise(self, f: impl FnOnce(&MagicError)) -> Self;

    fn send_error(self, err_msg_sender: Sender<Loglet>) -> Self;
}

impl<T> HandleError<T> for Result<T, MagicError> {
    fn consume_error(self, err_msg: &mut ErrorMessage) {
        if let Err(err) = self {
            err_msg.push_err(&err.to_string())
        }
    }
    fn send_error(self, err_msg_sender: Sender<Loglet>) -> Self {
        if let Err(err) = &self {
            err_msg_sender
                .send(Loglet::err_s(&err.to_string()))
                .unwrap()
        }
        return self;
    }
    fn otherwise(self, f: impl FnOnce(&MagicError)) -> Self {
        if let Err(err) = self.as_ref() {
            f(err);
        }
        return self;
    }
}

pub trait StatusCheck {
    fn status_check(self) -> Result<ureq::Response, ErrorX>;
}

use crate::err_tools::ErrorX;
fn reason(resp: ureq::Response) -> String {
    println!("passed");
    let msg: HashMap<String, String> = serde_json::from_reader(resp.into_reader()).unwrap();
    format!("Reason: -> |{}|\n", msg["error"])
}
impl StatusCheck for ureq::Response {
    fn status_check(self) -> Result<ureq::Response, ErrorX> {
        let status = self.status();
        let pattern = || format!("status code: -> |{}|", status);
        match status {
            202 => Ok(self),
            403 => Err(ErrorX::new(&format!(
                "Request Denied... {}\n{}",
                pattern(),
                reason(self)
            ))),
            _ => Err(ErrorX::new(&format!(
                "Request was not aproved: {}\nReason: -> |{}|",
                pattern(),
                reason(self)
            ))),
        }
    }
}
//pub trait New {
//    fn new<T>() -> T;
//}

// Random Functions
fn get_tokio_runtime() -> tokio::runtime::Runtime {
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

pub mod thread_tools {
    use std::{
        future::Future,
        sync::mpsc::{channel, Receiver, Sender},
    };

    pub struct Downloader<T: Default> {
        inner: T,
        receiver: Receiver<T>,
    }

    impl<T: Default> Downloader<T> {
        pub fn new() -> (Downloader<T>, Sender<T>) {
            let (sender, receiver) = channel();
            let downloader = Self {
                inner: T::default(),
                receiver,
            };
            return (downloader, sender);
        }
    }
    pub struct Uploader<T: Default> {
        inner: T,
        sender: Sender<T>,
    }

    impl<T: Default> Uploader<T> {
        pub fn new() -> (Uploader<T>, Receiver<T>) {
            let (sender, receiver) = channel();
            let uploader = Self {
                inner: T::default(),
                sender,
            };
            return (uploader, receiver);
        }
    }

    pub struct Communicator<T: Default> {
        // For two-way communication between threads
        inner: T,
        uploader: Uploader<T>,
        uploader_receiver: Receiver<T>,
        downloader: Downloader<T>,
        downloader_sender: Sender<T>,
    }
    impl<T: Default> Communicator<T> {
        pub fn new() -> Communicator<T> {
            let (downloader, sender) = Downloader::new();
            let (uploader, receiver) = Uploader::new();
            let communicator = Self {
                inner: T::default(),
                uploader,
                uploader_receiver: receiver,
                downloader,
                downloader_sender: sender,
            };

            communicator
        }
        // Senders
        pub fn downloader_sender_clone(&self) -> Sender<T> {
            self.downloader_sender.clone()
        }

        // Receivers
        pub fn downloader_receiver(&self) -> &Receiver<T> {
            &self.downloader.receiver
        }
    }

    use poll_promise::Promise;
    pub struct SPromise<T, F>
    where
        T: Send + 'static + Default,
        F: Future,
    {
        value: T,
        some_promise: Option<Promise<T>>,
        future: Option<F>,
        sender: Option<poll_promise::Sender<T>>,
    }

    impl<T, F> SPromise<T, F>
    where
        T: Send + 'static + Default,
        F: Future,
    {
        pub fn new() -> Self {
            let (sender, promise) = Promise::new();
            Self {
                value: T::default(),
                some_promise: Some(promise),
                future: None,
                sender: Some(sender),
            }
        }
        pub fn create_promise(value: T) -> Self {
            let (sender, promise) = Promise::new();
            Self {
                value,
                some_promise: Some(promise),
                future: None,
                sender: Some(sender),
            }
        }
        pub fn make_promise() -> (Self, poll_promise::Sender<T>) {
            let (sender, promise) = Promise::new();
            (
                Self {
                    value: T::default(),
                    some_promise: Some(promise),
                    future: None,
                    sender: None,
                },
                sender,
            )
        }
        pub fn make_no_promise(value: T) -> Self {
            //! It creates a (type Self) which initializes everything to None except Self.value
            //!
            //! In essence, you make no promises...!
            Self {
                value,
                some_promise: None,
                future: None,
                sender: None,
            }
        }
        pub fn spromise_ref(&self) -> Option<&poll_promise::Promise<T>> {
            //! Return ref to promise inside SPromise
            self.some_promise.as_ref()
        }
        pub fn sender_ref(&self) -> &Option<poll_promise::Sender<T>> {
            &self.sender
        }
        pub fn take_sender(&mut self) -> Option<poll_promise::Sender<T>> {
            // This stupid freaking lib will just throw a panic if the promise is dropped...
            // Never use this again!
            // Just check if the promise exists first LOL THEN check/take the sender
            //self.some_promise.as_ref()?;
            if let Some(_) = self.some_promise {
                println!("There is a promise here bois!")
            }
            if let Some(sender) = self.sender.take() {
                return Some(sender);
            }
            None
        }
        pub fn add_value(&mut self, value: T) {
            self.value = value
        }

        pub fn set_value(&mut self, value: T) {
            self.value = value
        }
        pub fn mut_value(&mut self) -> &mut T {
            &mut self.value
        }
        pub fn ref_value(&self) -> &T {
            &self.value
        }

        // Test Values
        pub fn quick_value(&self) -> &T {
            &self.value
        }
        pub fn quick_inner(&self) -> &Option<Promise<T>> {
            &self.some_promise
        }
        pub fn test(value: T) -> Self {
            //! This is a test function that could be mainlined
            //!
            //! It creates a (type Self) which initializes everything to None except Self.value
            Self {
                value,
                some_promise: None,
                future: None,
                sender: None,
            }
        }
    }
    // Store the sender, and receiver somewhere in the struct?
    // pass the sender @ struct creation to the (F: Future)
}
// Modules
mod increment {
    pub struct Inc {
        counter: i64,
    }
    impl Inc {
        pub fn up(&mut self) -> i64 {
            self.counter = self.counter + 1;
            self.counter
        }
        pub fn up_str(&mut self) -> String {
            self.counter = self.counter + 1;
            self.counter.to_string()
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
        pub fn new(msg: &str) -> ErrorX {
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
    use super::err_tools::ErrorX;
    use serde::Serialize;

    pub fn to_vec8(cereal: &impl Serialize) -> Vec<u8> {
        serde_json::to_vec(cereal).unwrap()
    }

    pub fn status_check(response: &ureq::Response) -> Result<(), ErrorX> {
        let status = response.status();
        let pattern = || format!("status code: -> |{}|", status);
        match status {
            202 => Ok(()),
            403 => Err(ErrorX::new(&format!("Request Denied... {}", pattern()))),
            _ => Err(ErrorX::new(&format!(
                "Request was not aproved: {}",
                pattern()
            ))),
        }
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
        EstimatedAmountNetherPortals,
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
                Routes::EstimatedAmountNetherPortals => "/netherportalsestimatedamount",
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
    pub struct ModalMachine {
        selected_option: String,
        options: Vec<String>,
        name: String,
        event: Option<()>,
    }

    impl ModalMachine {
        pub fn default() -> Self {
            Self {
                selected_option: "".to_string(),
                options: Vec::new(),
                name: "".to_string(),
                event: None,
            }
        }

        pub fn new(selected_option: String, options: Vec<String>, name: String) -> Self {
            Self {
                selected_option,
                options,
                name,
                event: None,
            }
        }

        pub fn get_selected_option(&self) -> String {
            self.selected_option.clone()
        }

        pub fn modal_machine(&mut self, id: i64, ui: &mut Ui) {
            ui.push_id(id, |ui| {
                eframe::egui::ComboBox::from_label(&self.name)
                    .selected_text(&self.selected_option)
                    .show_ui(ui, |ui| {
                        self.options.iter().for_each(|option| {
                            if ui
                                .selectable_value(&mut self.selected_option, option.clone(), option)
                                .clicked()
                            {
                                self.event = Some(());
                            };
                        });
                    });
            });
        }

        pub fn use_event(&mut self) -> Option<()> {
            self.event.take()
        }

        //pub fn use_event_ref(&mut self, func: impl FnOnce(&Self)) {
        //    func(&self);
        //}
    }

    pub fn display_retained_image(
        retained_image: &egui_extras::RetainedImage,
        ui: &mut eframe::egui::Ui,
    ) {
        let mut size = retained_image.size_vec2();
        size *= (ui.available_width() / size.x).min(1.0);
        retained_image.show_size(ui, size);
    }
}

// Tests
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
