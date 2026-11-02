use eframe::egui;

use crate::{
    dashboard, hud,
    logic::{AppState, Command},
    tray::Tray,
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Interface {
    Hud,
    Dashboard,
}

impl Interface {
    pub(crate) fn viewport(self) -> egui::ViewportBuilder {
        match self {
            Self::Hud => egui::ViewportBuilder::default()
                .with_title(hud::HUD_TITLE)
                .with_inner_size([hud::HUD_WIDTH, hud::HUD_HEIGHT])
                .with_decorations(hud::HUD_DECORATIONS)
                .with_transparent(hud::HUD_TRANSPARENT)
                .with_always_on_top(),
            Self::Dashboard => egui::ViewportBuilder::default()
                .with_title(dashboard::DASHBOARD_TITLE)
                .with_inner_size([dashboard::DASHBOARD_WIDTH, dashboard::DASHBOARD_HEIGHT])
                .with_decorations(dashboard::DASHBOARD_DECORATIONS)
                .with_transparent(dashboard::DASHBOARD_TRANSPARENT),
        }
    }
}

pub(crate) struct TaskerApp {
    state: AppState,
    root: Interface,
    hud: hud::TaskerApp,
    dashboard: dashboard::DashboardApp,
    tray: Tray,
}

impl TaskerApp {
    pub(crate) fn new(state: AppState, root: Interface, tray: Tray) -> Self {
        Self {
            state,
            root,
            hud: hud::TaskerApp,
            dashboard: dashboard::DashboardApp,
            tray,
        }
    }

    fn show(&mut self, interface: Interface, ui: &mut egui::Ui) {
        let command = match interface {
            Interface::Hud => self.hud.show(ui),
            Interface::Dashboard => self.dashboard.show(ui),
        };
        if let Some(command) = command {
            self.handle(command, ui.ctx());
        }
    }

    fn handle(&mut self, command: Command, ctx: &egui::Context) {
        let target = match command {
            Command::OpenHud => Interface::Hud,
            Command::OpenDashboard => Interface::Dashboard,
            Command::Exit => {
                self.state.handle(command);
                ctx.send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                return;
            }
        };
        self.state.handle(command);
        let id = if target == self.root {
            egui::ViewportId::ROOT
        } else {
            Self::child_id()
        };
        // Reuse the window. The dashboard doesn't need a cloning machine.
        ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
        ctx.request_repaint();
    }

    fn child_id() -> egui::ViewportId {
        egui::ViewportId::from_hash_of("secondary-interface")
    }
}

impl eframe::App for TaskerApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // This also runs when a tray event wakes a hidden root window.
        while let Some(command) = self.tray.next_command() {
            self.handle(command, ctx);
            if self.state.exit_requested {
                break;
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(self.root, ui);

        let (child, open) = match self.root {
            Interface::Hud => (Interface::Dashboard, self.state.dashboard_open),
            Interface::Dashboard => (Interface::Hud, self.state.hud_open),
        };
        if open {
            let ctx = ui.ctx().clone();
            ctx.show_viewport_immediate(Self::child_id(), child.viewport(), |ui, _class| {
                if ui.ctx().input(|i| i.viewport().close_requested()) {
                    match child {
                        Interface::Hud => self.state.hud_open = false,
                        Interface::Dashboard => self.state.dashboard_open = false,
                    }
                    ctx.request_repaint(); // Let the parent retire this window next pass.
                } else {
                    self.show(child, ui);
                }
            });
        }
        // Closing the root still exits normally; hiding it comes in the next stage.
    }
}
