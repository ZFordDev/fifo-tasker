# 📋 FIFO Tasker

**Add it. Queue it. Do it. Tick it.**

Ever start working on one project and accidentally discover 12 things that need fixing in another?

Yeah. Me too.

FIFO Tasker is a lightweight, local-first task manager for developers, hobbyists, and anyone whose to-do list has developed a personality.

## How It Works

- 📥 **Add a task.** It goes to the back of the queue.
- 📌 **Mark it priority.** Congratulations, you've jumped the queue.
- 🏷️ **Pick a project.** Or make a new one.
- ✅ **Tick it off.** Changed your mind? You've got 30 seconds to undo.
- 📜 **View history.** Proof you actually did something!
- 💤 **Forgot a project?** Optional reminders gently bring it back.

> [!NOTE]
> These are planned features rather than a list of what's already implemented. The end result may vary.

## 🖥️ Two Ways to Procrastinate Productively

**The Dashboard**

The main manager. You won't need it often, but it gives you ultimate control!

**The HUD**

Glint-inspired desktop widget with your task queue and a quick-add button.

Everything you need to keep things flowing should be here.

## Under the Hood

**The chosen stack:**

- 🦀 [Rust](https://rust-lang.org/) + [egui](https://github.com/emilk/egui) for the application and UI.
- 🔒 Local-first. No accounts, no cloud, no telemetry, as per usual!

**Still investigating:**

- 🗃️ [SQLite](https://www.sqlite.org/) via [rusqlite](https://github.com/rusqlite/rusqlite) or [SQLx](https://github.com/launchbadge/sqlx). Direct SQLite API access is also being considered.
- 📝 Markdown descriptions via [comrak](https://github.com/kivikakk/comrak) or [egui_markdown](https://github.com/membrane-io/egui_markdown).

## Files!

The current source structure:

```text
src/
├── main.rs              # Startup and launch arguments
├── app.rs               # Shared controller and window lifecycle
├── tray.rs              # System tray menu and commands
├── instance.rs          # Single-instance ownership and launch forwarding
├── instance/
│   └── tests.rs         # Startup, communication and recovery tests
├── logic/
│   └── mod.rs           # Shared application state and commands
├── hud/                 # Floating HUD
│   ├── mod.rs           # HUD content and controls
│   ├── config.rs        # Settings and derived constants
│   └── frame.rs         # Workspace, border and dragging
└── dashboard/           # Placeholder dashboard
    ├── mod.rs           # Dashboard content and controls
    ├── config.rs        # Window and workspace settings
    └── frame.rs         # Workspace and border drawing
```

Task management and database storage are still to come.

## Project Status

**Research & Development (R&D).**

Currently somewhere between *"this is a great idea"* and *"why am I building another application?"*

The current prototype has placeholder HUD and dashboard windows, a system tray, and single-instance launch handling. Closing the windows keeps the app running; use **Exit** in the tray to quit. Windows launch and lifecycle checks have been completed, while Ubuntu validation is still pending.

From the repository directory, run `cargo run` to open the HUD. Running it again opens or focuses the existing dashboard. Use `cargo run -- --dashboard` to start directly in the dashboard.

Next is a robust CLI and command/response handling, followed by database integration and actual task management. The to-do list still has a to-do list.

## Need info?
the repos wiki is actively being worked on! you can see devlogs, and a few placeholder documents  
Find it here 👉 [Wiki](https://github.com/ZFordDev/fifo-tasker/wiki)

## Licence

Licensed under the [Apache License 2.0](LICENSE).

---

### ⭐ One Last Thing

Found a small open-source project you like?

Go give it a star.

Seriously. It takes two seconds.

Doesn't have to be this project. Just show some love when you find a good repo.
