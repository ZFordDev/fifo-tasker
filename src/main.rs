mod app;
mod dashboard;
mod hud;
mod logic;

use app::{Interface, TaskerApp};

fn main() -> eframe::Result<()> {
    // Translate startup arguments into the same commands other entry points will use.
    let command = if std::env::args().any(|arg| arg == "--dashboard") {
        logic::Command::OpenDashboard
    } else {
        logic::Command::OpenHud
    };
    let mut state = logic::AppState::default();
    state.handle(command);

    let root = if state.dashboard_open {
        Interface::Dashboard
    } else {
        Interface::Hud
    };
    let options = eframe::NativeOptions {
        viewport: root.viewport(),
        ..Default::default()
    };

    // One event loop hosts both interfaces; no competing drivers at the wheel.
    eframe::run_native(
        "FIFO Tasker",
        options,
        Box::new(move |_cc| Ok(Box::new(TaskerApp::new(state, root)))),
    )
}
