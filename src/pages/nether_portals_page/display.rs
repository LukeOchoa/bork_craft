use std::collections::BTreeMap;

use crate::eframe_tools::ModalMachine;

use eframe::egui::Ui;
use egui_extras::Column;
use egui_extras::TableBuilder;

fn quick_table(ui: &mut Ui, columns: usize, reset: bool) -> TableBuilder {
    //! Just for settings up a simple table

    // Width of the columns
    let mut max_width = 300.0;
    let mut min_width = 40.0;
    let mut default_width = 300.0;
    let mut default_min_width = 150.0;

    // If the table should be reset, reinitalize its initial widths
    if reset {
        max_width = 70.0;
        min_width = 70.0;
        default_width = 150.0;
        default_min_width = 150.0;
    }

    // Basic settings for each extra column
    let column_settings = Column::initial(150.0)
        .range(default_min_width..=default_width)
        .resizable(true)
        .clip(false);

    // Create table based on settings
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(
            Column::initial(70.0)
                .range(min_width..=max_width)
                .resizable(true)
                .clip(false),
        );

    // If you want more than 1 table, iter through and add more!
    if columns > 0 {
        for _ in 0..columns {
            table = table.column(column_settings)
        }
    }

    // Return the fastest table alive
    table
}

pub fn portal_text_displayer(
    displayable_pt: &BTreeMap<String, String>,
    //table: TableBuilder,
    reset: bool,
    id: String,
    ui: &mut Ui,
) {
    // Allow for user to reset there scuffed up tables
    // Make a table
    ui.push_id(id, |ui| {
        let table = quick_table(ui, 1, reset);

        // Use that table bby!
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Type");
                });
                header.col(|ui| {
                    ui.strong("Details");
                });
            })
            .body(|mut body| {
                // Iter through the BTree & Display each key/element of Nether Portal Text info
                displayable_pt.iter().for_each(|(key, value)| {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("key: {}", key));
                        });
                        row.col(|ui| {
                            ui.label(format!("value: {}", value));
                        });
                    });
                });
            })
    });
}
