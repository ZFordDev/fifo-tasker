use eframe::egui;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "FIFO Tasker",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(TaskerApp))),
    )
}

struct TaskerApp;

impl eframe::App for TaskerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.label("Hello, FIFO Tasker!");
    }
}