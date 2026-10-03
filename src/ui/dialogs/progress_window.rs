use eframe::egui;
use std::cell::Cell;

pub fn render_progress_window(ctx: &egui::Context, app: &mut crate::app::App) {
    let window_id = match app.active_progress_window_id {
        Some(id) => id,
        None => return,
    };

    let item_idx = app.downloads.iter().position(|d| d.id == window_id);
    if item_idx.is_none() {
        app.active_progress_window_id = None;
        return;
    }
    let idx = item_idx.unwrap();

    let item = &app.downloads[idx];
    let title = format!("{}% {}", if let Some(s) = item.size { (item.downloaded as f64 / s as f64 * 100.0) as u32 } else { 0 }, item.file_name);

    // Gather data we need for the viewport closure
    let url = item.url.clone();
    let status_str = item.status.to_string();
    let size = item.size;
    let downloaded = item.downloaded;
    let transfer_rate = item.transfer_rate;
    let time_left = item.time_left_secs;
    let connections = item.connections.max(1) as usize;

    let close = Cell::new(false);
    let pause = Cell::new(false);

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("progress_window"),
        egui::ViewportBuilder::default()
            .with_title(&title)
            .with_inner_size([580.0, 500.0])
            .with_resizable(false)
            .with_always_on_top(),
        |ctx, _class| {
            crate::theme::IdmTheme::apply(ctx);

            if ctx.input(|i| i.viewport().close_requested()) {
                close.set(true);
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let _ = ui.selectable_label(true, "Download status");
                    let _ = ui.selectable_label(false, "Speed Limiter");
                    let _ = ui.selectable_label(false, "Options on completion");
                });

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&url).small());
                    });
                    ui.add_space(8.0);

                    let fields = [
                        ("Status", status_str.clone()),
                        ("File size", if let Some(s) = size { format!("{:.3} MB", s as f64 / 1_048_576.0) } else { "Unknown".to_string() }),
                        ("Downloaded", format!("{:.3} MB ( {}% )", downloaded as f64 / 1_048_576.0, if let Some(s) = size { (downloaded as f64 / s as f64 * 100.0) as u32 } else { 0 })),
                        ("Transfer rate", format!("{:.3} MB/sec", transfer_rate / 1_048_576.0)),
                        ("Time left", time_left.map(|s| format!("{:.0} sec", s)).unwrap_or_else(|| "Unknown".to_string())),
                        ("Resume capability", "Yes".to_string()),
                    ];

                    egui::Grid::new("prog_grid").num_columns(2).show(ui, |ui| {
                        for (k, v) in fields {
                            ui.label(k);
                            ui.label(v);
                            ui.end_row();
                        }
                    });
                });

                ui.add_space(8.0);

                // Progress bar
                let progress = if let Some(s) = size { downloaded as f32 / s as f32 } else { 0.0 };
                let rect = ui.allocate_space(egui::vec2(ui.available_width(), 16.0)).1;
                ui.painter().rect_filled(rect, 2.0, egui::Color32::from_gray(200));
                let filled = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * progress, rect.height()));
                ui.painter().rect_filled(filled, 2.0, egui::Color32::from_rgb(0, 180, 0));

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("<< Hide details").clicked() {}
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() { close.set(true); }
                        if ui.button("Pause").clicked() { pause.set(true); }
                    });
                });

                ui.add_space(8.0);
                ui.vertical_centered(|ui| { ui.label("Start positions and download progress by connections"); });

                // Connection visual bar
                let bar_rect = ui.allocate_space(egui::vec2(ui.available_width(), 16.0)).1;
                ui.painter().rect_filled(bar_rect, 0.0, egui::Color32::WHITE);
                for i in 0..connections {
                    let chunk_w = bar_rect.width() / connections as f32;
                    let chunk_x = bar_rect.min.x + i as f32 * chunk_w;
                    let chunk_rect = egui::Rect::from_min_size(egui::pos2(chunk_x, bar_rect.min.y), egui::vec2(chunk_w * progress, bar_rect.height()));
                    ui.painter().rect_filled(chunk_rect, 0.0, crate::theme::IdmTheme::ACCENT_BLUE);
                    ui.painter().vline(chunk_x, bar_rect.y_range(), egui::Stroke::new(1.0_f32, egui::Color32::RED));
                }

                ui.add_space(8.0);

                // Connections Table
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("N."); ui.label("Downloaded"); ui.label("Info");
                    });
                    ui.separator();
                    for i in 1..=connections {
                        ui.horizontal(|ui| {
                            ui.label(i.to_string());
                            ui.label(format!("{:.3} MB", (downloaded as f64 / connections as f64) / 1_048_576.0));
                            ui.label("Receiving data...");
                        });
                    }
                });
            });
        },
    );

    if pause.get() {
        app.download_manager.pause_download(window_id);
        app.active_progress_window_id = None;
    }
    if close.get() {
        app.active_progress_window_id = None;
    }
}
