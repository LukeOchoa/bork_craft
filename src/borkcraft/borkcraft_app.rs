// My Trash Imports
use crate::{
    eframe_tools::ModalMachine,
    get_tokio_runtime,
    increment::Inc,
    option,
    pages::{
        login::{login_page, LoginForm},
        nether_portals_page::{
            display_images::{change_image_by_user_input, display_nether_portal_images},
            download_images::*,
            page::display_nether_portals_page,
            portals::NetherPortals,
        },
    },
    sessions::{current_session_time, SessionInfo, SessionTime},
    time_of_day,
    windows::{
        client_windows::{GenericWindow, Loglet},
        error_messages::ErrorMessage,
    },
    HandleOption, Realm,
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

fn realm_modal(nps: &mut NetherPortals, id: i64, ui: &mut Ui) {
    nps.realm_modal_mut().modal_machine(id, ui);

    option(|| {
        // Check for an event; if None, return
        nps.realm_modal_mut().use_event()?;

        // Get user selected netherportals
        let pos = nps.realm_modal_mut().get_selected_option();

        // set the current netherportal to the recently selected
        nps.realm_pos_set2(pos);

        Some(())
    });
}

impl BorkCraft {
    fn update_updaters(&mut self) {
        self.unique.reset();
        self.err_msg.try_update_log();
        self.session_info.try_update().ok();
        self.nether_portals.try_update_npt().ok();

        // If there is a mm event, update the image that should be displayed
        change_image_by_user_input(&mut self.nether_portals, &Realm::Overworld);
        change_image_by_user_input(&mut self.nether_portals, &Realm::Nether);
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
                realm_modal(&mut self.nether_portals, self.unique.up(), ui);
                move_back_or_forth_buttons(
                    &mut self.nether_portals,
                    &self.runtime,
                    &mut self.unique,
                    self.err_msg.sender_clone(),
                    ui,
                );
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
            "Nether Portals" => {
                display_nether_portal_images(&mut self.nether_portals, &mut self.unique, ui)
            }
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

fn move_back_or_forth_buttons(
    nps: &mut NetherPortals,
    runtime: &tokio::runtime::Runtime,
    unique: &mut Inc,
    err_msg_sender: Sender<Loglet>,
    ui: &mut Ui,
) {
    let mut clicked = false;
    ui.horizontal(|ui| {
        // Buttons to move the nether portals selections back or forth
        if ui.button("Go Back").clicked() {
            nps.ow_pos_down();
            nps.neth_pos_down();
            clicked = true;
        }

        if ui.button("Go Forth").clicked() {
            nps.ow_pos_up();
            nps.neth_pos_up();
            clicked = true;
        }
    });
    // On change, you need to check if there are images that should be downloaded
    if clicked {
        // Download the images
        if let Some((np, position)) = should_we_reload_ow_images(nps) {
            load_images(np, position, runtime, err_msg_sender.clone());
        }

        if let Some((np, position)) = should_we_reload_nether_images(nps) {
            load_images(np, position, runtime, err_msg_sender);
        }

        // Reload/Remake ModalMachines
        reload_image_mm(nps, &Realm::Overworld, unique.up_str());
        reload_image_mm(nps, &Realm::Nether, unique.up_str());
    }

    //// Display the ModalMachines
    //nps.image_modal_mut(&Realm::Nether)
    //    .modal_machine(unique.up(), ui);
    //nps.image_modal_mut(&Realm::Overworld)
    //    .modal_machine(unique.up(), ui);
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("TopBoi").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Show Keys").clicked() {
                    self.nether_portals.show_keys();
                }
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
                    // Is this good?
                    self.handle_pages(ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_image_pages(ui);
        });

        // update
        self.update_updaters();
        ctx.request_repaint();
    }
}
