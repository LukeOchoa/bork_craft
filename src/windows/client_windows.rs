use crate::{eframe_tools::scroll_and_vert, string_tools::*, try_access};
use eframe::egui::{Context, Ui};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct GenericWindow {
    pub is_window_open: bool,
    pub try_to_open_window: bool,
    pub log: MessageLog,
    pub name: String,
}

impl GenericWindow {
    pub fn default() -> Self {
        Default::default()
    }
    pub fn new(name: &str) -> Self {
        let mut gw = Self::default();
        gw.name(name);
        gw
    }

    pub fn name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn namae(&self) -> String {
        self.name.to_string()
    }

    pub fn open_window_on_click(&mut self, ui: &mut Ui, name: &str) {
        eframe::egui::Grid::new(name).show(ui, |ui| {
            if ui.button(name).clicked() {
                self.is_window_open = !self.is_window_open
            }
            if self.try_to_open_window {
                self.is_window_open = false;
                self.try_to_open_window = false;
            }
            ui.end_row();
        });
    }

    pub fn show(&mut self, ctx: Context, f: impl Fn(&mut Ui, Context, &MessageLog)) -> bool {
        let mut is_window_shut: bool = self.is_window_open;
        eframe::egui::Window::new(&self.name)
            .resizable(true)
            .open(&mut is_window_shut)
            .show(&ctx, |ui| f(ui, ctx.clone(), &self.log));

        self.is_window_open = is_window_shut;

        is_window_shut
    }

    fn helper(generic_window: &mut Self, id: i64, ui: &mut Ui, ctx: Context) {
        generic_window.show(ctx.clone(), |ui, _, log| {
            scroll_and_vert(ui, id, |ui| {
                log.show(ui);
            })
        });
        generic_window.open_window_on_click(ui, &generic_window.namae());
    }
    pub fn display_generic_window(gn: GN, id: i64, ui: &mut Ui, ctx: Context) {
        match gn {
            GN::Tau(generic_window) => try_access(generic_window, |mut access| {
                Self::helper(&mut access, id, ui, ctx.clone());
            })
            .unwrap(),
            GN::Green(generic_window) => {
                Self::helper(generic_window, id, ui, ctx);
            }
        }
    }
}
pub enum GN<'a> {
    Tau(&'a Arc<Mutex<GenericWindow>>),
    Green(&'a mut GenericWindow),
}

#[derive(Default)]
pub struct MessageLog {
    log: Vec<Loglet>,
}

impl MessageLog {
    pub fn default() -> Self {
        Default::default()
    }
    pub fn show(&self, ui: &mut Ui) {
        for (index, loglet) in self.log.iter().enumerate() {
            let formated_loglet = format!("{}):{}{}", index, newliner(1), loglet.format_loglet());
            ui.label(formated_loglet);
            ui.label(newliner(3));
            ui.end_row()
        }
    }
    pub fn push(&mut self, loglet: Loglet) {
        self.log.push(loglet)
    }
}
pub struct Loglet {
    kind: String,
    msg: String,
    time: String,
}

impl Loglet {
    pub fn new(kind: String, msg: String, time: String) -> Loglet {
        Self { kind, msg, time }
    }

    pub fn format_loglet(&self) -> String {
        let lyne = |elem: &String| -> String { format!("|{}{}{}|", newliner(1), tabber(1), elem) };
        format!(
            "Kind:{}Message:{}Time:{}",
            lyne(&self.kind),
            lyne(&self.msg),
            lyne(&self.time)
        )
    }
}
