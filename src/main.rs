#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auto_detect;
mod color_engine;
mod crosshair;
mod profiles;
mod ui;

use auto_detect::ProcessDetector;
use color_engine::DisplayTunerEngine;
use crosshair::CrosshairController;
use std::sync::{Arc, Mutex};
use ui::LarusXApp;

fn main() -> eframe::Result<()> {
    println!("Initializing LarusX Display & Visibility Engine (Rust Native)...");

    // 1. Initialize core display engine
    let engine = Arc::new(DisplayTunerEngine::new());
    if !engine.is_ready() {
        eprintln!("Warning: Magnification API could not be initialized.");
    }

    // 2. Initialize hardware crosshair controller
    let crosshair_ctrl = Arc::new(CrosshairController::new());

    // 3. Setup auto process detector
    let current_foreground = Arc::new(Mutex::new(String::new()));
    let detector = ProcessDetector::new();
    let fg_clone = current_foreground.clone();
    detector.start_monitoring(move |proc_name| {
        if let Ok(mut lock) = fg_clone.lock() {
            *lock = proc_name;
        }
    });

    // 4. Load application icon
    let icon_bytes = include_bytes!("../assets/app_icon.png");
    let icon_data = eframe::icon_data::from_png_bytes(icon_bytes).ok();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([540.0, 680.0])
        .with_min_inner_size([460.0, 520.0])
        .with_title("LarusX — Competitive Display & Crosshair Engine (Syojhin & Lara)")
        .with_active(true);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    // 5. Configure eframe native window
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let engine_app = engine.clone();
    let crosshair_app = crosshair_ctrl.clone();
    let fg_app = current_foreground.clone();

    eframe::run_native(
        "LarusX",
        options,
        Box::new(move |cc| {
            Ok(Box::new(LarusXApp::new(
                cc,
                engine_app,
                crosshair_app,
                fg_app,
            )))
        }),
    )
}
