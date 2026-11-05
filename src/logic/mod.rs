// Shared intentions; window creation stays with the GUI.
pub(crate) enum Command {
    OpenHud,
    OpenDashboard,
    CloseHud,
    CloseDashboard,
    Exit,
}

#[derive(Default)]
pub(crate) struct AppState {
    pub(crate) task_data: Arc<RwLock<tasks::TaskData>>,
    pub(crate) hud_open: bool,
    pub(crate) dashboard_open: bool,
    pub(crate) exit_requested: bool,
}

impl AppState {
    pub(crate) fn handle(&mut self, command: Command) {
        // Both interfaces can be wanted at once. No sibling rivalry here.
        match command {
            Command::OpenHud => self.hud_open = true,
            Command::OpenDashboard => self.dashboard_open = true,
            Command::CloseHud => self.hud_open = false,
            Command::CloseDashboard => self.dashboard_open = false,
            Command::Exit => self.exit_requested = true,
        }
    }
}
pub(crate) mod tasks;
use std::sync::{Arc, RwLock};
