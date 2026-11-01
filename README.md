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
> This is more of a shopping list over a feature list; the end result may vary.

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
├── main.rs              # Application entry point
└── hud/                 # Floating HUD
    ├── mod.rs           # TaskerApp and HUD content
    ├── config.rs        # Settings and derived constants
    └── frame.rs         # Workspace, border and dragging
```

The dashboard and shared task logic will get their own modules when development reaches them.

## Project Status

**Research & Development (R&D).**

Currently somewhere between *"this is a great idea"* and *"why am I building another application?"*

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