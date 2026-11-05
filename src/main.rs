mod app;
mod dashboard;
mod hud;
mod instance;
mod logic;
mod storage;
mod task_list;
mod task_watch;
mod tray;

use app::{Interface, TaskerApp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only the owner creates windows or a tray; later launches knock on its door.
    let Some(instance) = instance::Instance::acquire()? else {
        return Ok(());
    };
    // Translate startup arguments into the same commands other entry points will use.
    let command = if std::env::args().any(|arg| arg == "--dashboard") {
        logic::Command::OpenDashboard
    } else {
        logic::Command::OpenHud
    };
    let mut state = logic::AppState::default();
    // Only the owner reads task data; later launches have already forwarded and exited.
    let loaded = storage::task_file().and_then(|path| {
        eprintln!("Task data file: {}", path.display());
        storage::load(&path)
    });
    {
        let mut data = state.task_data.write().unwrap();
        match loaded {
            Ok(Some(tasks)) => {
                data.document = tasks;
                data.revision = 1;
            }
            Ok(None) => eprintln!("No task data file yet; starting with an empty list."),
            Err(error) => data.load_error = Some(error.to_string()),
        }
        if let Some(error) = &data.load_error {
            eprintln!("{error}. Starting with an empty list; the file has been left unchanged.");
        } else {
            eprintln!("Loaded {} tasks.", data.document.tasks().len());
        }
    }
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
        Box::new(move |cc| {
            // Create the tray on the GUI thread once the event loop is running.
            let tray = tray::Tray::new(cc.egui_ctx.clone())?;
            instance.attach(cc.egui_ctx.clone());
            let watcher = storage::task_file().and_then(|path| {
                task_watch::TaskWatcher::new(
                    path,
                    std::sync::Arc::clone(&state.task_data),
                    cc.egui_ctx.clone(),
                )
            });
            let watcher = match watcher {
                Ok(watcher) => Some(watcher),
                Err(error) => {
                    eprintln!("Automatic task reload unavailable: {error}");
                    state.task_data.write().unwrap().watch_error = Some(error.to_string());
                    None
                }
            };
            instance.serve_tasks(
                std::sync::Arc::clone(&state.task_data),
                watcher.as_ref().map(task_watch::TaskWatcher::reload_handle),
            );
            Ok(Box::new(TaskerApp::new(
                state, root, tray, instance, watcher,
            )))
        }),
    )?;
    Ok(())
}
