use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use thiserror::Error;
use waver_engine::{audio_catalog, default_selection, spawn_output, spawn_output_for};
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
        if let Some(request) = self.app.take_audio_request() {
            match request {
                waver_core::AudioSettingsRequest::Refresh => {
                    self.app.update_audio_catalog(audio_catalog())
                }
                waver_core::AudioSettingsRequest::Apply(selection) => {
                    let mut candidate = spawn_output_for(&selection);
                    if let Some(error) = candidate.error.take() {
                        self.app.audio_switch_failed(error);
                    } else if let Some(commands) = candidate.take_commands() {
                        self.app.replace_audio(
                            commands,
                            Arc::clone(&candidate.status),
                            candidate.device_name.clone(),
                            selection,
                        );
                        self._audio = candidate;
                    } else {
                        self.app.audio_switch_failed(AppError::Commands.to_string());
                    }
                }
            }
        }
    }
}

fn run() -> Result<(), AppError> {
    let mut audio = spawn_output();
    let commands = audio.take_commands().ok_or(AppError::Commands)?;
    let mut app = WaverApp::new(
        commands,
        Arc::clone(&audio.status),
        audio.device_name.clone(),
        audio.error.clone(),
    );

    app.configure_audio_settings(audio_catalog(), default_selection());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 960.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_title("waver"),
        ..Default::default()
    };

    eframe::run_native(
        "waver",
        options,
        Box::new(move |cc| {
            waver_ui::setup_fonts(&cc.egui_ctx);
            waver_ui::setup_theme(&cc.egui_ctx);
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
