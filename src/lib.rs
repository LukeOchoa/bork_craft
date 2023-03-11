pub mod borkcraft;

pub use borkcraft::*;

pub mod image_tools {

    use egui_extras::RetainedImage;
    use std::{fs::File, io::Read, path::Path};
    type MagicError = Box<dyn std::error::Error>;

    pub fn get_image() -> Result<RetainedImage, MagicError> {
        let path = std::path::Path::new("ryujin_main.png");
        let retained_image = turn_path_into_image(path)?;

        Ok(retained_image)
    }

    pub fn display_retained_image(retained_image: &RetainedImage, ui: &mut eframe::egui::Ui) {
        let mut size = retained_image.size_vec2();
        size *= (ui.available_width() / size.x).min(1.0);
        retained_image.show_size(ui, size);
    }

    pub fn turn_path_into_image(path: &Path) -> Result<RetainedImage, MagicError> {
        let file = File::open(path)?;
        let mut buffer = Vec::new();
        std::io::BufReader::new(file).read_to_end(&mut buffer)?;
        let image = egui_extras::image::RetainedImage::from_image_bytes("your mom", &buffer)?;

        Ok(image)
    }
}
