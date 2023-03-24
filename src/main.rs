use bork_craft::borkcraft_app::BorkCraft;

fn main() {
    let option = eframe::NativeOptions::default();
    eframe::run_native(
        "BorkCraft",
        option,
        Box::new(|_cc| Box::new(BorkCraft::default())),
    );
}
