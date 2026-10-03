use eframe::egui;
use crate::models::{DownloadItem, Category}; use crate::engine::types::DownloadState;
use std::cell::Cell;

pub fn render_file_info_dialog(ctx: &egui::Context, app: &mut crate::app::App) {
    if app.fetching_info {
        // Show spinner as a separate small OS window while fetching
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("fetching_info_window"),
            egui::ViewportBuilder::default()
                .with_title("Fetching...")
                .with_inner_size([300.0, 80.0])
                .with_resizable(false)
                .with_always_on_top(),
            |ctx, _class| {
                crate::theme::IdmTheme::apply(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Getting file size and name...");
                    });
                });
            },
        );
        return;
    }

    if app.pending_file_info.is_none() { return; }

    let start_download = Cell::new(false);
    let download_later = Cell::new(false);
    let close = Cell::new(false);

    // Take info out so we can pass it into the viewport closure
    let mut info = app.pending_file_info.take().unwrap();

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("file_info_window"),
        egui::ViewportBuilder::default()
            .with_title("Download File Info")
            .with_inner_size([620.0, 320.0])
            .with_resizable(false)
            .with_always_on_top(),
        |ctx, _class| {
            crate::theme::IdmTheme::apply(ctx);

            // Check if user closed the OS window via X button
            if ctx.input(|i| i.viewport().close_requested()) {
                close.set(true);
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        // URL
                        ui.horizontal(|ui| {
                            ui.label("URL");
                            let mut url_display = info.url.clone();
                            ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut url_display));
                        });
                        ui.add_space(8.0);
                        // Category
                        ui.horizontal(|ui| {
                            ui.label("Category");
                            egui::ComboBox::from_id_salt("file_info_cat")
                                .selected_text(format!("{:?}", info.category))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut info.category, Category::Compressed, "Compressed");
                                    ui.selectable_value(&mut info.category, Category::Documents, "Documents");
                                    ui.selectable_value(&mut info.category, Category::Music, "Music");
                                    ui.selectable_value(&mut info.category, Category::Programs, "Programs");
                                    ui.selectable_value(&mut info.category, Category::Video, "Video");
                                    ui.selectable_value(&mut info.category, Category::All, "All");
                                });
                        });
                        ui.add_space(8.0);
                        // Save As
                        ui.horizontal(|ui| {
                            ui.label("Save As");
                            ui.add_sized(ui.available_size() - egui::vec2(30.0, 0.0), egui::TextEdit::singleline(&mut info.save_path));
                            if ui.button("...").clicked() {
                                if let Some(path) = rfd::FileDialog::new().set_file_name(&info.file_name).save_file() {
                                    info.save_path = path.to_string_lossy().to_string();
                                }
                            }
                        });
                        // Remember path
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut info.remember_path, format!("Remember this path for \"{:?}\" category", info.category));
                        });
                        ui.add_space(8.0);
                        // Description
                        ui.horizontal(|ui| {
                            ui.label("Description");
                            ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut info.description));
                        });
                    });

                    // Right side info (icon + size)
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.add(crate::ui::sidebar::get_icon_image(316.0, 462.0, 75.0, 90.0).max_size(egui::vec2(48.0, 48.0)));
                        ui.add_space(8.0);
                        let size_str = if let Some(s) = info.size {
                            format!("{:.2} MB", s as f64 / 1_048_576.0)
                        } else {
                            "Unknown".to_string()
                        };
                        ui.label(egui::RichText::new(size_str).strong());
                    });
                });

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() { close.set(true); }
                        if ui.button("Start Download").clicked() { start_download.set(true); }
                        if ui.button("Download Later").clicked() { download_later.set(true); }
                    });
                });
            });
        },
    );

    if close.get() {
        // User cancelled — don't put info back
        return;
    }

    if start_download.get() || download_later.get() {
        let id = app.next_download_id();
        let new_item = DownloadItem {
            id,
            file_name: info.file_name,
            url: info.url.clone(),
            status: if start_download.get() { DownloadState::Downloading } else { DownloadState::Paused },
            size: info.size,
            downloaded: 0,
            transfer_rate: 0.0,
            time_left_secs: None,
            save_path: info.save_path.clone(),
            last_try_date: Some(chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string()),
            category: info.category,
            description: info.description,
            connections: app.config.connection.max_connections,
            request_context: info.request_context.clone(),
        };
        app.downloads.push(new_item);

        if start_download.get() {
            app.active_progress_window_id = Some(id);
            app.download_manager.start_download_with_context(
                id,
                info.url,
                std::path::PathBuf::from(info.save_path),
                app.config.connection.max_connections,
                info.request_context,
            );
        }
    } else {
        // No action yet — put info back so the window stays open next frame
        app.pending_file_info = Some(info);
    }
}
