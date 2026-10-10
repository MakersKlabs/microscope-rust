//! Microscope (Rust) — retro-styled USB microscope capture application.
//!
//! Aims for the smallest possible self-contained binary while supporting
//! Linux, Windows and macOS from a single code base.

#![warn(rust_2018_idioms)]
// No console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod camera;
mod frame;
mod i18n;
mod recorder;
mod settings;
mod theme;
mod util;

use eframe::egui;

fn load_icon() -> egui::IconData {
    match image::load_from_memory(include_bytes!("../assets/icon.png")) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            }
        }
        Err(_) => egui::IconData {
            rgba: vec![0, 0, 0, 0],
            width: 1,
            height: 1,
        },
    }
}

fn main() -> eframe::Result<()> {
    let settings = settings::Settings::load();
    i18n::set_language(&settings.language);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([820.0, 540.0])
            .with_title("MICROSCOPE")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Microscope",
        options,
        Box::new(move |cc| Ok(Box::new(app::MicroscopeApp::new(cc, settings)))),
    )
}
