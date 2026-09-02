#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod converter;

use app::{MyApp, setup_fonts};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 650.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Image format conversion",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(MyApp::default()))
        }),
    )
}

fn load_icon() -> eframe::egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/app_icon.png"))
        .expect("Failed to load icon")
        .into_rgba8();

    let width = image.width();
    let height = image.height();

    eframe::egui::IconData {
        rgba: image.into_raw(),
        height,
        width,
    }
}
