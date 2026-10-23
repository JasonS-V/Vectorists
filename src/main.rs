pub mod core;
pub mod input;
pub mod ui;

use eframe::egui;
use ui::VectoristsApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([680.0, 560.0])
            .with_title("Vectorists · Control de precisión"),
        ..Default::default()
    };

    eframe::run_native(
        "Vectorists",
        options,
        Box::new(|cc| Ok(Box::new(VectoristsApp::new(cc)))),
    )
}
