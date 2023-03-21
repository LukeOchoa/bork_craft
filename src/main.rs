use bork_craft::borkcraft_app::BorkCraft;
use std::future::Future;
use std::pin::Pin;

fn main() {
    let option = eframe::NativeOptions::default();
    eframe::run_native(
        "BorkCraft",
        option,
        Box::new(|_cc| Box::new(BorkCraft::default())),
    );
}
