use eframe::egui;
use std::cell::Cell;

pub fn render_complete_windows(ctx: &egui::Context, app: &mut crate::app::App) {
    let mut closed_ids = Vec::new();
    let mut open_paths = Vec::new();

    for &id in &app.completed_downloads_to_show {
        let item_idx = app.downloads.iter().position(|d| d.id == id);
        if item_idx.is_none() {
            closed_ids.push(id);
            continue;
        }
        let idx = item_idx.unwrap();
        let item = &app.downloads[idx];

        let title = "Download complete".to_string();
        let url = item.url.clone();
        let save_path = item.save_path.clone();
        let size = item.size.unwrap_or(0);
        let size_mb = size as f64 / 1_048_576.0;

        let close = Cell::new(false);
        let open_file = Cell::new(false);
        let open_folder = Cell::new(false);

        let viewport_id = egui::ViewportId::from_hash_of(&format!("complete_window_{}", id));

        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title(&title)
                .with_inner_size([500.0, 220.0])
                .with_resizable(false)
                .with_always_on_top(),
            |ctx, _class| {
                crate::theme::IdmTheme::apply(ctx);

                if ctx.input(|i| i.viewport().close_requested()) {
                    close.set(true);
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Icon (approximate folder/file icon)
                        ui.add(crate::ui::sidebar::get_icon_image(316.0, 462.0, 75.0, 90.0).max_size(egui::vec2(48.0, 48.0)));
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Download complete").strong());
                            ui.label(format!("Downloaded {:.2} MB ({} Bytes)", size_mb, size));
                        });
                    });

                    ui.add_space(8.0);

                    ui.label("Address");
                    let mut url_display = url.clone();
                    ui.add_sized(egui::vec2(ui.available_width(), 20.0), egui::TextEdit::singleline(&mut url_display).interactive(true));

                    ui.add_space(8.0);

                    ui.label("The file saved as");
                    let mut path_display = save_path.clone();
                    ui.add_sized(egui::vec2(ui.available_width(), 20.0), egui::TextEdit::singleline(&mut path_display).interactive(true));

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.button("Open").clicked() {
                            open_file.set(true);
                            close.set(true);
                        }
                        if ui.button("Open with...").clicked() {
                            // Dummy for now
                        }
                        if ui.button("Open folder").clicked() {
                            open_folder.set(true);
                            close.set(true);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close.set(true);
                            }
                        });
                    });

                    ui.add_space(8.0);
                    let mut dont_show = false;
                    ui.checkbox(&mut dont_show, "Don't show this dialog again");
                });
            },
        );

        if close.get() {
            closed_ids.push(id);
        }

        if open_file.get() {
            open_paths.push((save_path.clone(), true));
        } else if open_folder.get() {
            open_paths.push((save_path.clone(), false));
        }
    }

    app.completed_downloads_to_show.retain(|id| !closed_ids.contains(id));

    for (path_str, is_file) in open_paths {
        if is_file {
            let _ = opener::open(&path_str);
        } else {
            if let Some(parent) = std::path::Path::new(&path_str).parent() {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer").arg(format!("/select,{}", path_str)).spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = opener::open(parent);
                }
            }
        }
    }
}
