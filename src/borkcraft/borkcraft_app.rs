// emilk imports
use crate::image_tools::*;
use eframe::egui::{self, ScrollArea};
use egui_extras::RetainedImage;

pub struct BorkCraft {
    pub image: RetainedImage,
}

impl Default for BorkCraft {
    fn default() -> Self {
        Self {
            image: get_image().unwrap(),
        }
    }
}

const LONG_BOI: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Mauris malesuada, erat quis efficitur scelerisque, libero quam mattis eros, at malesuada est nunc quis est. Curabitur tincidunt a nisi nec bibendum. Nunc eget magna risus. Fusce elit quam, porttitor sed turpis eget, gravida egestas orci. Nulla placerat dui a orci fringilla vehicula. Curabitur rhoncus leo ut lacus condimentum, sed pellentesque nibh blandit. Aenean sit amet arcu a neque rhoncus laoreet. In ac metus sit amet mi ultricies tincidunt. Nulla imperdiet velit vestibulum aliquam volutpat. Morbi viverra id turpis at aliquet. Maecenas euismod turpis at maximus lobortis. Pellentesque semper risus in facilisis commodo. Nullam iaculis, leo ut auctor volutpat, tellus neque porta orci, vitae elementum mi diam vel nibh. Phasellus sagittis sodales orci, a viverra felis dapibus at. Cras sed nulla porttitor, euismod massa vitae, elementum ipsum.";

impl eframe::App for BorkCraft {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("topboi").show(ctx, |ui| {
            ui.label("Ryujin is too beautiful!");
        });
        egui::SidePanel::left(1).show(ctx, |ui| {
            ScrollArea::vertical().id_source("source").show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let unnecessarily_long_text = LONG_BOI.to_string();
                    for i in 0..10 {
                        let label = format!("{}): {}", i, unnecessarily_long_text);
                        ui.label(label);
                        ui.end_row();
                    }
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            display_retained_image(&self.image, ui);
        });
        //egui::SidePanel::right(2).show(ctx, |ui| {
        //    display_retained_image(&self.image, ui);
        //});
    }
}
