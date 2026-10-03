use eframe::egui;

pub fn render_scheduler_dialog(ctx: &egui::Context, app: &mut crate::app::App) {
    let mut open = app.show_scheduler_dialog;
    if !open {
        return;
    }

    egui::Window::new("Scheduler")
        .open(&mut open)
        .default_size([400.0, 300.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Scheduler functionality coming soon...");
            if ui.button("Close").clicked() {
                app.show_scheduler_dialog = false;
            }
        });

    app.show_scheduler_dialog = open;
}
