use eframe::egui;
use crate::app::App;
use crate::models::{Category, SidebarCategory}; use crate::engine::types::DownloadState;
use crate::theme::IdmTheme;

pub fn render_download_table(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                render_table(ui, app);
            });
    });
}

fn render_table(ui: &mut egui::Ui, app: &mut App) {
    let row_height = 40.0;
    let row_gap = 2.0;
    let col_widths: [f32; 9] = [240.0, 80.0, 140.0, 90.0, 80.0, 100.0, 100.0, 140.0, 160.0];
    let col_names = ["File Name", "Size", "Progress", "Status", "Time left", "Speed", "Downloaded", "Last Try Date", "Description"];

    // ── Table Header ──
    let header_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 28.0));

    let mut x = header_rect.min.x;
    for (i, &title) in col_names.iter().enumerate() {
        let w = if i == col_widths.len() - 1 { (header_rect.max.x - x).max(col_widths[i]) } else { col_widths[i] };
        ui.painter().text(
            egui::pos2(x + 8.0, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(11.5),
            IdmTheme::TEXT_SECONDARY,
        );
        x += w;
    }
    ui.painter().hline(header_rect.x_range(), header_rect.max.y, egui::Stroke::new(1.0_f32, IdmTheme::BORDER));
    ui.advance_cursor_after_rect(header_rect);
    ui.add_space(4.0);

    // ── Filter Downloads ──
    let indices_to_show: Vec<usize> = app.downloads.iter().enumerate()
        .filter(|(_, d)| {
            let cat_match = match &app.selected_category {
                SidebarCategory::AllDownloads => true,
                SidebarCategory::Compressed => d.category == Category::Compressed,
                SidebarCategory::Documents => d.category == Category::Documents,
                SidebarCategory::Music => d.category == Category::Music,
                SidebarCategory::Programs => d.category == Category::Programs,
                SidebarCategory::Video => d.category == Category::Video,
                SidebarCategory::Unfinished => d.status != DownloadState::Complete,
                SidebarCategory::Finished => d.status == DownloadState::Complete,
                SidebarCategory::GrabberProjects | SidebarCategory::Queues => false,
            };
            let search_match = app.search_text.is_empty() || d.file_name.to_lowercase().contains(&app.search_text.to_lowercase());
            cat_match && search_match
        })
        .map(|(i, _)| i)
        .collect();

    if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::A)) {
        app.selected_downloads = indices_to_show.iter().copied().collect();
    }

    let mut resume_idx = None;
    let mut do_resume_selected = false;
    let mut do_stop_selected = false;
    let mut do_remove_selected = false;
    let mut do_open_selected = false;
    let mut do_open_folder_selected = false;
    let mut do_redownload_selected = false;

    // ── Render Rows ──
    for (row_idx, &dl_idx) in indices_to_show.iter().enumerate() {
        let is_selected = app.selected_downloads.contains(&dl_idx);
        let row_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), row_height));
        let response = ui.interact(row_rect, ui.id().with(("dl_row", dl_idx)), egui::Sense::click());

        let bg_color = if is_selected {
            IdmTheme::SELECTED_ROW
        } else if response.hovered() {
            IdmTheme::HOVERED_ROW
        } else if row_idx % 2 == 0 {
            IdmTheme::TABLE_ROW_EVEN
        } else {
            IdmTheme::TABLE_ROW_ODD
        };
        ui.painter().rect_filled(row_rect, 6.0, bg_color);

        if response.clicked() {
            if !ui.input(|i| i.modifiers.ctrl) { app.selected_downloads.clear(); }
            app.selected_downloads.insert(dl_idx);
        } else if response.secondary_clicked() {
            if !app.selected_downloads.contains(&dl_idx) {
                app.selected_downloads.clear();
                app.selected_downloads.insert(dl_idx);
            }
        }

        if response.double_clicked() {
            let d = &app.downloads[dl_idx];
            if d.status == DownloadState::Paused || d.status == DownloadState::Queued || d.status == DownloadState::Failed {
                resume_idx = Some(dl_idx);
            } else if d.status == DownloadState::Complete {
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd").args(["/c", "start", "", &d.save_path]).spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&d.save_path).spawn();
            }
        }

        // Context menu
        response.context_menu(|ui| {
            let mut any_paused = false;
            let mut any_downloading = false;
            let mut any_complete = false;
            for &idx in &app.selected_downloads {
                if idx < app.downloads.len() {
                    let st = &app.downloads[idx].status;
                    if *st == DownloadState::Paused || *st == DownloadState::Queued || *st == DownloadState::Failed { any_paused = true; }
                    if *st == DownloadState::Downloading { any_downloading = true; }
                    if *st == DownloadState::Complete { any_complete = true; }
                }
            }
            if ui.add_enabled(any_complete, egui::Button::new("Open")).clicked() { do_open_selected = true; ui.close_menu(); }
            if ui.add_enabled(any_complete, egui::Button::new("Open folder")).clicked() { do_open_folder_selected = true; ui.close_menu(); }
            ui.separator();
            if ui.add_enabled(true, egui::Button::new("Redownload")).clicked() { do_redownload_selected = true; ui.close_menu(); }
            ui.separator();
            if ui.add_enabled(any_paused, egui::Button::new("Resume Download")).clicked() { do_resume_selected = true; ui.close_menu(); }
            if ui.add_enabled(any_downloading, egui::Button::new("Stop Download")).clicked() { do_stop_selected = true; ui.close_menu(); }
            ui.separator();
            if ui.button("Remove").clicked() { do_remove_selected = true; ui.close_menu(); }
        });

        let d = &app.downloads[dl_idx];

        // ── Draw cells ──
        let mut x = row_rect.min.x;
        for i in 0..col_widths.len() {
            let w = if i == col_widths.len() - 1 { (row_rect.max.x - x).max(col_widths[i]) } else { col_widths[i] };
            let cell_rect = egui::Rect::from_min_size(egui::pos2(x, row_rect.min.y), egui::vec2(w, row_height));

            match i {
                0 => {
                    // Status dot
                    let dot_color = match &d.status {
                        DownloadState::Downloading => IdmTheme::STATUS_ACTIVE,
                        DownloadState::Complete => IdmTheme::STATUS_COMPLETE,
                        DownloadState::Paused => IdmTheme::STATUS_PAUSED,
                        DownloadState::Failed => IdmTheme::STATUS_ERROR,
                        _ => IdmTheme::TEXT_SECONDARY,
                    };
                    ui.painter().circle_filled(egui::pos2(cell_rect.min.x + 16.0, cell_rect.center().y), 4.0, dot_color);

                    // File name
                    let text_rect = egui::Rect::from_min_max(
                        egui::pos2(cell_rect.min.x + 28.0, cell_rect.min.y),
                        cell_rect.max,
                    );
                    ui.allocate_ui_at_rect(text_rect, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(&d.file_name).color(IdmTheme::TEXT_PRIMARY).size(13.0)).truncate());
                        });
                    });
                }
                2 => {
                    // Progress bar
                    let fraction = d.progress_fraction();
                    let bar_height = 8.0;
                    let bar_rect = egui::Rect::from_center_size(
                        egui::pos2(cell_rect.center().x - 10.0, cell_rect.center().y),
                        egui::vec2(cell_rect.width() - 40.0, bar_height),
                    );
                    let bar_rounding = bar_height / 2.0;
                    ui.painter().rect_filled(bar_rect, bar_rounding, IdmTheme::PROGRESS_BG);

                    if fraction > 0.0 {
                        let fill_rect = egui::Rect::from_min_size(bar_rect.min, egui::vec2(bar_rect.width() * fraction, bar_height));
                        let fill_color = match &d.status {
                            DownloadState::Complete => IdmTheme::STATUS_COMPLETE,
                            DownloadState::Downloading => IdmTheme::PROGRESS_FILL,
                            DownloadState::Paused => IdmTheme::STATUS_PAUSED,
                            DownloadState::Failed => IdmTheme::STATUS_ERROR,
                            _ => IdmTheme::PROGRESS_FILL,
                        };
                        ui.painter().rect_filled(fill_rect, bar_rounding, fill_color);
                    }

                    ui.painter().text(
                        egui::pos2(bar_rect.max.x + 4.0, cell_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!("{:.0}%", fraction * 100.0),
                        egui::FontId::proportional(11.0),
                        IdmTheme::TEXT_SECONDARY,
                    );
                }
                3 => {
                    // Status text
                    let color = match &d.status {
                        DownloadState::Complete => IdmTheme::STATUS_COMPLETE,
                        DownloadState::Downloading => IdmTheme::STATUS_ACTIVE,
                        DownloadState::Paused => IdmTheme::STATUS_PAUSED,
                        DownloadState::Failed => IdmTheme::STATUS_ERROR,
                        _ => IdmTheme::TEXT_SECONDARY,
                    };
                    ui.painter().text(
                        egui::pos2(cell_rect.min.x + 8.0, cell_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        d.status.to_string(),
                        egui::FontId::proportional(12.5),
                        color,
                    );
                }
                _ => {
                    let text = match i {
                        1 => d.size_display(),
                        4 => if d.status == DownloadState::Downloading { d.time_left_display() } else { String::new() },
                        5 => if d.status == DownloadState::Downloading && d.transfer_rate > 0.0 { d.rate_display() } else { String::new() },
                        6 => if d.downloaded > 0 { format_bytes(d.downloaded) } else { String::new() },
                        7 => d.last_try_date.clone().unwrap_or_default(),
                        8 => if d.status == DownloadState::Failed { d.description.clone() } else { d.description.clone() },
                        _ => String::new(),
                    };
                    let color = if i == 8 && d.status == DownloadState::Failed { IdmTheme::STATUS_ERROR } else { IdmTheme::TEXT_PRIMARY };

                    ui.allocate_ui_at_rect(cell_rect.shrink2(egui::vec2(8.0, 0.0)), |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(&text).color(color).size(12.5)).truncate());
                        });
                    });
                }
            }
            x += w;
        }

        ui.advance_cursor_after_rect(row_rect);
        ui.add_space(row_gap);
    }

    // Empty state
    if indices_to_show.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("No downloads").color(IdmTheme::TEXT_SECONDARY).size(16.0));
        });
    }

    // Deselect on empty space click
    let remaining = ui.available_rect_before_wrap();
    if remaining.height() > 0.0 {
        let response = ui.interact(remaining, ui.id().with("empty_space"), egui::Sense::click());
        if response.clicked() || response.secondary_clicked() { app.selected_downloads.clear(); }
    }

    // ── Process deferred actions ──
    if let Some(idx) = resume_idx {
        let d = &app.downloads[idx];
        let id = d.id;
        let url = d.url.clone();
        let save_path = std::path::PathBuf::from(&d.save_path);
        let connections = d.connections;
                                let request_context = d.request_context.clone();
        app.downloads[idx].status = DownloadState::Downloading;
        app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
    }
    if do_resume_selected {
        for &idx in &app.selected_downloads {
            if idx < app.downloads.len() {
                let d = &app.downloads[idx];
                if d.status == DownloadState::Paused || d.status == DownloadState::Queued || d.status == DownloadState::Failed {
                    let id = d.id;
                    let url = d.url.clone();
                    let save_path = std::path::PathBuf::from(&d.save_path);
                    let connections = d.connections;
                                let request_context = d.request_context.clone();
                    app.downloads[idx].status = DownloadState::Downloading;
                    app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
                }
            }
        }
    }
    if do_stop_selected {
        for &idx in &app.selected_downloads {
            if idx < app.downloads.len() && app.downloads[idx].status == DownloadState::Downloading {
                app.download_manager.pause_download(app.downloads[idx].id);
            }
        }
    }
    if do_redownload_selected {
        for &idx in &app.selected_downloads {
            if idx < app.downloads.len() {
                let d = &app.downloads[idx];
                let id = d.id;
                let save_path = std::path::PathBuf::from(&d.save_path);
                if d.status == DownloadState::Downloading { app.download_manager.pause_download(id); }
                let _ = std::fs::remove_file(&save_path);
                let _ = std::fs::remove_file(save_path.with_extension("idm"));
                let url = d.url.clone();
                let connections = d.connections;
                                let request_context = d.request_context.clone();
                app.downloads[idx].downloaded = 0;
                app.downloads[idx].status = DownloadState::Downloading;
                app.downloads[idx].last_try_date = Some(chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string());
                app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
            }
        }
    }
    if do_remove_selected {
        let mut to_remove: Vec<usize> = app.selected_downloads.iter().copied().collect();
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for idx in to_remove {
            if idx < app.downloads.len() {
                let d = &app.downloads[idx];
                let id = d.id;
                let save_path = std::path::PathBuf::from(&d.save_path);
                let complete = d.status == DownloadState::Complete;
                app.download_manager.cancel_download(id, !complete, &save_path);
                app.downloads.remove(idx);
            }
        }
        app.selected_downloads.clear();
    }
    if do_open_selected {
        for &idx in &app.selected_downloads {
            if idx < app.downloads.len() && app.downloads[idx].status == DownloadState::Complete {
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd").args(["/c", "start", "", &app.downloads[idx].save_path]).spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&app.downloads[idx].save_path).spawn();
            }
        }
    }
    if do_open_folder_selected {
        for &idx in &app.selected_downloads {
            if idx < app.downloads.len() && app.downloads[idx].status == DownloadState::Complete {
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("explorer").arg(format!("/select,{}", &app.downloads[idx].save_path)).spawn();
                #[cfg(not(target_os = "windows"))]
                {
                    if let Some(parent) = std::path::Path::new(&app.downloads[idx].save_path).parent() {
                        let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                    }
                }
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * KB;
    const GB: f64 = 1024.0 * MB;
    let b = bytes as f64;
    if b >= GB { format!("{:.2} GB", b / GB) }
    else if b >= MB { format!("{:.2} MB", b / MB) }
    else if b >= KB { format!("{:.2} KB", b / KB) }
    else { format!("{} B", bytes) }
}
