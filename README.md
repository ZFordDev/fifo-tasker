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

While it's too early to know for sure, I'm thinking:

- 🦀 [Rust](https://rust-lang.org/) + [egui](https://github.com/emilk/egui) // seriously, use egui for everything; it's the new PySide!
- 🗃️ [SQLite](https://www.sqlite.org/) via [rusqlite](https://github.com/rusqlite/rusqlite) or [SQLx](https://github.com/launchbadge/sqlx), although direct API is considered.
- 📝 Markdown descriptions via either [comrak](https://github.com/kivikakk/comrak) or [egui_markdown](https://github.com/membrane-io/egui_markdown)
- 🔒 Local-first. No accounts, no cloud, no telemetry, as per usual!

## Project Status

**Research & Development (R&D).**

Currently somewhere between *"this is a great idea"* and *"why am I building another application?"*

## Licence

Licensed under the [Apache License 2.0](LICENSE).

---

### ⭐ One Last Thing

Found a small open-source project you like?

Go give it a star.

Seriously. It takes two seconds.

Doesn't have to be this project. Just show some love when you find a good repo.