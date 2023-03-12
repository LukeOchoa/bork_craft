// emilk imports
use crate::{
    eframe_tools::scroll_and_vert, increment::Inc, sessions::SessionInfo, try_access,
    windows::client_windows::Loglet,
};
use eframe::egui::{self, Context, ScrollArea, Ui};
use std::sync::{Arc, Mutex, Once};

// GLOBALS
static START: Once = Once::new();

pub struct BorkCraft {
    unique: Inc, // unique id
    session_info: Arc<Mutex<SessionInfo>>,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            unique: Inc::new(),
            session_info: Arc::new(Mutex::new(SessionInfo::default())),
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
            }
        })
        .unwrap();
    });
}

fn display_session_time_left(ui: &mut Ui, ctx: Context, session_info: &Arc<Mutex<SessionInfo>>) {
    try_access(&session_info, |mut access| {
        access
            .display
            .show(ctx.clone(), "Session Time", |ui, _, log| {
                scroll_and_vert(ui, "Session Time", |ui| {
                    log.show(ui);
                });
            });
        access.display.open_window_on_click(ui, "Session Time");
    })
    .unwrap();
}

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        init(&self.session_info);
        egui::TopBottomPanel::top("TopBoi").show(ctx, |ui| {
            display_session_time_left(ui, ctx.clone(), &self.session_info);
            // do work
        });

        egui::SidePanel::left(self.unique.up()).show(ctx, |ui| {
            ScrollArea::vertical()
                .id_source(self.unique.up())
                .show(ui, |_ui| {
                    // do work
                });
        });

        egui::CentralPanel::default().show(ctx, |_ui| {
            // do work
        });

        self.unique.reset();
    }
}
