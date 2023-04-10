// My Trash Imports
use crate::{
    eframe_tools::ModalMachine,
    get_tokio_runtime,
    increment::Inc,
    pages::{
        login::{login_page, LoginForm},
        nether_portals_page::{page::display_nether_portals_page, portals::NetherPortals},
        //nether_portals_page::nether_portals_page,
        //nether_portals_page_options::portals::NetherPortals,
        //NetherPortals,
    },
    sessions::{current_session_time, SessionInfo, SessionTime},
    time_of_day,
    windows::{
        client_windows::{GenericWindow, Loglet},
        error_messages::ErrorMessage,
    },
};

// Emilk Imports
use eframe::egui::{self, Context, ScrollArea, Ui};

// Tokio Imports
// use tokio::runtime::Runtime;

// Godly Standard Library Imports
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Once,
};

// GLOBALS
static START: Once = Once::new();

fn set_base_page() -> ModalMachine {
    let options = vec!["Login".to_string(), "Nether Portals".to_string()];
    ModalMachine::new(
        options[0].clone(),
        options,
        "Welcome to BorkCraft".to_string(),
    )
}
pub struct BorkCraft {
    unique: Inc, // unique id
    runtime: tokio::runtime::Runtime,
    login_form: LoginForm,
    session_info: SessionInfo,
    base_page: ModalMachine,
    nether_portals: NetherPortals,
    err_msg: ErrorMessage,
}

impl Default for BorkCraft {
    fn default() -> Self {
        // Session Information
        let (sender, receiver) = channel();
        let session_info = SessionInfo::new(Some(receiver));

        // Login Form
        let (key_sender, key_receiver) = channel();
        let login_form = LoginForm {
            sender: Some(key_sender),
            ..LoginForm::default()
        };

        // Error Message
        let err_msg = ErrorMessage::new();

        // Tokio
        let runtime = get_tokio_runtime();

        // NetherPortals
        let nether_portals = NetherPortals::default();

        // ModalMachines
        let base_page = set_base_page();

        START.call_once(|| {
            real_init(sender, key_receiver, err_msg.sender_clone());
        });
        Self {
            unique: Inc::new(),
            runtime,
            login_form,
            session_info,
            nether_portals,
            base_page,
            err_msg,
        }
    }
}
// Temp use statement, Delete later...
use crate::pages::nether_portals_page::download_images::display_nether_portal_images;
impl BorkCraft {
    fn update_updaters(&mut self) {
        self.unique.reset();
        self.err_msg.try_update_log();
        _ = self.session_info.try_update();
        _ = self.nether_portals.try_update_npt();
    }

    fn handle_pages(&mut self, ui: &mut Ui) {
        match &self.base_page.get_selected_option() as &str {
            "Login" => {
                login_page(
                    &mut self.session_info,
                    &mut self.login_form,
                    ui,
                    &mut self.err_msg,
                );
            }
            "Nether Portals" => {
                display_nether_portals_page(
                    &mut self.nether_portals,
                    &mut self.unique,
                    &mut self.err_msg,
                    &self.runtime,
                    ui,
                );
            }
            _ => {
                ui.label("In development. Sorry...");
            }
        }
    }
    fn handle_image_pages(&mut self, ui: &mut Ui) {
        match &self.base_page.get_selected_option() as &str {
            "Login" => {}
            "Nether Portals" => display_nether_portal_images(
                &mut self.nether_portals,
                &self.runtime,
                self.err_msg.sender_clone(),
                ui,
            ),
            _ => {
                ui.label("In development. Sorry...");
            }
        }
    }
}
fn real_init(
    session_info_sender: Sender<(SessionTime, Loglet)>,
    key_receiver: Receiver<String>,
    err_sender: Sender<Loglet>,
) {
    // Consider give this thread a ctx? so that i can wake up the ui thread on an error or on a session update
    std::thread::spawn(move || {
        let mut key = String::default();
        loop {
            // Update Key if the user logged in again
            if let Ok(new_key) = key_receiver.try_recv() {
                key = new_key;
            }

            // Fetch session time. Ok() => update key; Err() => Sender error
            match current_session_time(&session_info_sender, key.clone()) {
                Ok(maybe_new_key) => key = maybe_new_key,
                Err(err) => err_sender
                    .send(Loglet::new("Error", &err.to_string(), &time_of_day()))
                    .unwrap(),
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    });
}

fn display_session_time_left(session_info: &mut SessionInfo, id: i64, ui: &mut Ui, ctx: Context) {
    // Set name and show: Button, GenericWindow Glory
    session_info.display.namae("Session Time");
    GenericWindow::display_generic_window(&mut session_info.display, id, ui, ctx);
}

fn display_err_msgs(err_msg: &mut ErrorMessage, id: i64, ui: &mut Ui, ctx: Context) {
    err_msg.display.namae("Error Messages");
    GenericWindow::display_generic_window(&mut err_msg.display, id, ui, ctx);
}

fn handle_base_page(base_page: &mut ModalMachine, id: i64, ui: &mut Ui) {
    base_page.modal_machine(id, ui);
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("TopBoi").show(ctx, |ui| {
            ui.horizontal(|ui| {
                handle_base_page(&mut self.base_page, self.unique.up(), ui);
                display_session_time_left(
                    &mut self.session_info,
                    self.unique.up(),
                    ui,
                    ctx.clone(),
                );

                display_err_msgs(&mut self.err_msg, self.unique.up(), ui, ctx.clone());
            });
        });

        egui::SidePanel::left(self.unique.up_str()).show(ctx, |ui| {
            ScrollArea::vertical()
                .id_source(self.unique.up())
                .show(ui, |ui| {
                    self.handle_pages(ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_image_pages(ui);
            // do work
        });

        // update
        self.update_updaters();
        ctx.request_repaint();
    }
}
