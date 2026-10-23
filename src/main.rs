pub mod core;
pub mod ui;

use eframe::egui;
use ui::VectoristsApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([880.0, 580.0])
            .with_min_inner_size([780.0, 500.0])
            .with_title("Vectorists - Biomechanical Mouse Curve Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Vectorists",
        options,
        Box::new(|cc| Ok(Box::new(VectoristsApp::new(cc)))),
    )
}
