// My Trash Imports
use crate::{
    get_tokio_runtime,
    increment::Inc,
    pages::{
        login::{login_page, LoginForm},
        nether_portals::nether_portals_page,
        portals::NetherPortals,
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
use std::{
    future::Future,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Once,
    },
};

// GLOBALS
static START: Once = Once::new();

pub struct BorkCraft {
    unique: Inc, // unique id
    runtime: tokio::runtime::Runtime,
    login_form: LoginForm,
    session_info: SessionInfo,
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

        START.call_once(|| {
            real_init(sender, key_receiver, err_msg.sender_clone());
        });
        Self {
            unique: Inc::new(),
            runtime,
            login_form,
            session_info,
            nether_portals,
            err_msg,
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

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("TopBoi").show(ctx, |ui| {
            ui.horizontal(|ui| {
                display_session_time_left(
                    &mut self.session_info,
                    self.unique.up(),
                    ui,
                    ctx.clone(),
                );

                display_err_msgs(&mut self.err_msg, self.unique.up(), ui, ctx.clone());
            });
        });

        egui::SidePanel::left(self.unique.up()).show(ctx, |ui| {
            ScrollArea::vertical()
                .id_source(self.unique.up())
                .show(ui, |ui| {
                    login_page(
                        &mut self.session_info,
                        &mut self.login_form,
                        ui,
                        &mut self.err_msg,
                    );
                    nether_portals_page(&mut self.nether_portals, &self.err_msg, &self.runtime, ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            // do work
        });

        // update
        self.unique.reset();
        self.err_msg.try_update_log();
        _ = self.session_info.try_update();

        //
        _ = self.nether_portals.try_update_npt();

        ctx.request_repaint();
    }
}
