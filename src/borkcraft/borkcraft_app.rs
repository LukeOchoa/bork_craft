// emilk imports
use crate::{
    eframe_tools::scroll_and_vert,
    increment::Inc,
    pages::login::{login_page, LoginForm},
    sessions::SessionInfo,
    try_access,
    windows::client_windows::{GenericWindow, Loglet, GN},
};
use eframe::egui::{self, Context, ScrollArea, Ui};
use std::sync::{Arc, Mutex, Once};

// GLOBALS
static START: Once = Once::new();

pub struct BorkCraft {
    unique: Inc, // unique id
    login_form: LoginForm,
    session_info: Arc<Mutex<SessionInfo>>,
    err_msg: Arc<Mutex<GenericWindow>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            unique: Inc::new(),
            login_form: LoginForm::default(),
            session_info: Arc::new(Mutex::new(SessionInfo::default())),
            err_msg: Arc::new(Mutex::new(GenericWindow::new("Error Messages"))),
        }
    }
}

fn init(session_info: &Arc<Mutex<SessionInfo>>) {
    START.call_once(|| {
        try_access(session_info, |mut access| {
            for i in 0..10 {
                let (kind, msg, time) = (
                    format!("|kind:{}|", i),
                    format!("|msg:{}|", i),
                    format!("|time:{}|", i),
                );
                access.display.log.push(Loglet::new(kind, msg, time));
                access.display.name("Session Time");
            }
        })
        .unwrap();
    });
}

fn display_session_time_left(
    session_info: &Arc<Mutex<SessionInfo>>,
    id: i64,
    ui: &mut Ui,
    ctx: Context,
) {
    try_access(&session_info, |mut access| {
        let name = &access.display.namae();
        access.display.show(ctx.clone(), |ui, _, log| {
            scroll_and_vert(ui, id, |ui| {
                log.show(ui);
            });
        });
        access.display.open_window_on_click(ui, name);
    })
    .unwrap();
}
fn display_err_msgs(err_msg: &Arc<Mutex<GenericWindow>>, id: i64, ui: &mut Ui, ctx: Context) {
    GenericWindow::display_generic_window(GN::Tau(err_msg), id, ui, ctx.clone());
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        init(&self.session_info);
        egui::TopBottomPanel::top("TopBoi").show(ctx, |ui| {
            ui.horizontal(|ui| {
                display_session_time_left(&self.session_info, self.unique.up(), ui, ctx.clone());
                display_err_msgs(&self.err_msg, self.unique.up(), ui, ctx.clone());
            });
        });

        egui::SidePanel::left(self.unique.up()).show(ctx, |ui| {
            ScrollArea::vertical()
                .id_source(self.unique.up())
                .show(ui, |ui| {
                    login_page(&mut self.session_info, &mut self.login_form, ui);
                });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            // do work
        });

        self.unique.reset();
    }
}
