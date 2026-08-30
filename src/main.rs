use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use thiserror::Error;
use waver_engine::spawn_output;
use waver_ui::WaverApp;

#[derive(Debug, Error)]
enum AppError {
    #[error("failed to start window: {0}")]
    Eframe(String),
    #[error("command queue missing after stream setup")]
    Commands,
}

struct AppShell {
    app: WaverApp,
    _audio: waver_engine::AudioRuntime,
}

impl eframe::App for AppShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.app.ui(ui);
    }
}

fn run() -> Result<(), AppError> {
    let mut audio = spawn_output();
    let commands = audio.take_commands().ok_or(AppError::Commands)?;
    let app = WaverApp::new(
        commands,
        Arc::clone(&audio.status),
        audio.device_name.clone(),
        audio.error.clone(),
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_title("waver"),
        ..Default::default()
    };

    eframe::run_native(
        "waver",
        options,
        Box::new(move |cc| {
            waver_ui::setup_fonts(&cc.egui_ctx);
            Ok(Box::new(AppShell { app, _audio: audio }))
        }),
    )
    .map_err(|err| AppError::Eframe(err.to_string()))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
