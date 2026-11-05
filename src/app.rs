use eframe::egui;
use std::sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, Sender},
};

use crate::{
    dashboard, hud,
    instance::Instance,
    logic::{AppState, Command},
    task_watch::TaskWatcher,
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
    hud: Arc<Mutex<hud::TaskerApp>>,
    dashboard: Arc<Mutex<dashboard::DashboardApp>>,
    window_sender: Sender<Command>,
    window_commands: Receiver<Command>,
    tray: Tray,
    instance: Instance,
    _task_watcher: Option<TaskWatcher>, // Keep the subscription alive for the application's lifetime.
}

impl TaskerApp {
    pub(crate) fn new(
        state: AppState,
        root: Interface,
        tray: Tray,
        instance: Instance,
        task_watcher: Option<TaskWatcher>,
    ) -> Self {
        let (window_sender, window_commands) = mpsc::channel();
        Self {
            state,
            root,
            hud: Arc::new(Mutex::new(hud::TaskerApp)),
            dashboard: Arc::new(Mutex::new(dashboard::DashboardApp)),
            window_sender,
            window_commands,
            tray,
            instance,
            _task_watcher: task_watcher,
        }
    }

    fn show(&mut self, interface: Interface, ui: &mut egui::Ui) {
        let command = {
            let data = self.state.task_data.read().unwrap();
            match interface {
                Interface::Hud => self.hud.lock().unwrap().show(ui, &data),
                Interface::Dashboard => self.dashboard.lock().unwrap().show(ui, &data),
            }
        };
        if let Some(command) = command {
            self.handle(command, ui.ctx());
        }
    }

    fn handle(&mut self, command: Command, ctx: &egui::Context) {
        let (target, open) = match command {
            Command::OpenHud => (Interface::Hud, true),
            Command::OpenDashboard => (Interface::Dashboard, true),
            Command::CloseHud => (Interface::Hud, false),
            Command::CloseDashboard => (Interface::Dashboard, false),
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
        if target == self.root {
            ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Visible(open));
        }
        if open {
            // Reuse the window. The dashboard doesn't need a cloning machine.
            ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Focus);
        }
        // A hidden root needs one UI pass to create or retire its child window.
        ctx.send_viewport_cmd_to(
            egui::ViewportId::ROOT,
            egui::ViewportCommand::RequestPaintWhileHidden,
        );
        ctx.request_repaint_of(egui::ViewportId::ROOT);
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
        if !self.state.exit_requested {
            while let Ok(command) = self.window_commands.try_recv() {
                self.handle(command, ctx);
            }
            while let Some(command) = self.instance.next_command() {
                self.handle(command, ctx);
            }
        }
        if !self.state.exit_requested && ctx.input(|i| i.viewport().close_requested()) {
            // The native close button means "back to the tray", not "goodbye".
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            let command = match self.root {
                Interface::Hud => Command::CloseHud,
                Interface::Dashboard => Command::CloseDashboard,
            };
            self.handle(command, ctx);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let root_open = match self.root {
            Interface::Hud => self.state.hud_open,
            Interface::Dashboard => self.state.dashboard_open,
        };
        if root_open {
            self.show(self.root, ui);
        }

        let (child, open) = match self.root {
            Interface::Hud => (Interface::Dashboard, self.state.dashboard_open),
            Interface::Dashboard => (Interface::Hud, self.state.hud_open),
        };
        if open {
            let ctx = ui.ctx().clone();
            let sender = self.window_sender.clone();
            let hud = Arc::clone(&self.hud);
            let dashboard = Arc::clone(&self.dashboard);
            let task_data = Arc::clone(&self.state.task_data);
            // Deferred windows can repaint independently while their parent is hidden.
            ctx.show_viewport_deferred(Self::child_id(), child.viewport(), move |ui, _class| {
                let command = if ui.ctx().input(|i| i.viewport().close_requested()) {
                    Some(match child {
                        Interface::Hud => Command::CloseHud,
                        Interface::Dashboard => Command::CloseDashboard,
                    })
                } else {
                    let data = task_data.read().unwrap();
                    match child {
                        Interface::Hud => hud.lock().unwrap().show(ui, &data),
                        Interface::Dashboard => dashboard.lock().unwrap().show(ui, &data),
                    }
                };
                if let Some(command) = command {
                    let _ = sender.send(command);
                    ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                }
            });
        }
    }
}
