extern crate eframe;
use eframe::{egui, epi};
mod app;

use app::App;

use crossbeam_channel;

fn main() {
    let app = App::default();
    eframe::run_native(Box::new(app), epi::NativeOptions {
        always_on_top: false,
        decorated: true,
        drag_and_drop_support: false,
        icon_data: None,
        initial_window_size: None,
        resizable: true,
        transparent: false,
    });
}
