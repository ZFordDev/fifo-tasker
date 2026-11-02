// Shared intentions; window creation stays with the GUI.
pub(crate) enum Command {
    OpenHud,
    OpenDashboard,
    Exit,
}

#[derive(Default)]
pub(crate) struct AppState {
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
            Command::Exit => self.exit_requested = true,
        }
    }
}
