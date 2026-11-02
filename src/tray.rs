use std::{
    error::Error,
    sync::mpsc::{self, Receiver},
};

use eframe::egui;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::logic::Command;

pub(crate) struct Tray {
    // Dropping the icon removes it. Keep our tiny mission control on duty.
    _icon: TrayIcon,
    commands: Receiver<Command>,
}

impl Tray {
    pub(crate) fn new(ctx: egui::Context) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let menu = Menu::new();
        let hud = MenuItem::with_id("open-hud", "Open HUD", true, None);
        let dashboard = MenuItem::with_id("open-dashboard", "Open Dashboard", true, None);
        let exit = MenuItem::with_id("exit", "Exit", true, None);
        menu.append_items(&[&hud, &dashboard, &PredefinedMenuItem::separator(), &exit])?;

        // Embed the SVG's 32px export so moving the executable won't lose its hat.
        let image =
            image::load_from_memory(include_bytes!("../assets/logo/tray.png"))?.into_rgba8();
        let (width, height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), width, height)?;
        let icon = TrayIconBuilder::new()
            .with_tooltip("FIFO Tasker")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()?;

        let (sender, commands) = mpsc::channel();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let command = match event.id.as_ref() {
                "open-hud" => Command::OpenHud,
                "open-dashboard" => Command::OpenDashboard,
                "exit" => Command::Exit,
                _ => return,
            };
            if sender.send(command).is_ok() {
                // Ring the controller's doorbell, even if the other UI was active.
                ctx.request_repaint_of(egui::ViewportId::ROOT);
            }
        }));

        Ok(Self {
            _icon: icon,
            commands,
        })
    }

    pub(crate) fn next_command(&self) -> Option<Command> {
        self.commands.try_recv().ok()
    }
}
