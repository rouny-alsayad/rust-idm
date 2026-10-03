use eframe::egui;
use crate::engine::types::DownloadState;

pub fn get_icon_image<'a>(x: f32, y: f32, w: f32, h: f32) -> egui::Image<'a> {
    let uv = egui::Rect::from_min_max(
        egui::pos2(x / 1448.0, y / 1086.0),
        egui::pos2((x + w) / 1448.0, (y + h) / 1086.0),
    );
    egui::Image::new(egui::include_image!("../icon/icons.png"))
        .uv(uv)
}

/// Modern pill-shaped toolbar button with optional accent color
fn pill_button(ui: &mut egui::Ui, img: egui::Image<'_>, label: &str, accent: egui::Color32) -> egui::Response {
    let text_width = ui.fonts(|f| f.layout_no_wrap(label.to_string(), egui::FontId::proportional(13.0), egui::Color32::WHITE).size().x);
    let button_width = (18.0 + 8.0 + text_width + 24.0).max(80.0);
    let button_height = 32.0;
    let button_size = egui::vec2(button_width, button_height);
    let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let rounding = button_height / 2.0;

        let bg = if response.is_pointer_button_down_on() {
            accent
        } else if response.hovered() {
            egui::Color32::from_rgba_unmultiplied(
                accent.r().saturating_add(20),
                accent.g().saturating_add(20),
                accent.b().saturating_add(20),
                40,
            )
        } else {
            crate::theme::IdmTheme::BUTTON_BG
        };

        ui.painter().rect_filled(rect, rounding, bg);

        if response.hovered() || response.is_pointer_button_down_on() {
            ui.painter().rect_stroke(
                rect, rounding,
                egui::Stroke::new(1.0_f32, accent.linear_multiply(0.6)),
                egui::StrokeKind::Inside,
            );
        }

        let text_color = if response.is_pointer_button_down_on() {
            egui::Color32::WHITE
        } else if response.hovered() {
            accent
        } else {
            crate::theme::IdmTheme::TEXT_PRIMARY
        };

        let icon_size = egui::vec2(18.0, 18.0);
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.min.x + 20.0, rect.center().y),
            icon_size,
        );
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(icon_rect)
                .layout(egui::Layout::centered_and_justified(egui::Direction::TopDown)),
        );
        child_ui.add(img.maintain_aspect_ratio(true));

        ui.painter().text(
            egui::pos2(rect.min.x + 36.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );
    }
    response
}

/// Compact icon-only toolbar button
fn icon_button(ui: &mut egui::Ui, img: egui::Image<'_>, tooltip: &str) -> egui::Response {
    let button_size = egui::vec2(36.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let rounding = 8.0;
        let bg = if response.is_pointer_button_down_on() {
            crate::theme::IdmTheme::ACCENT_BLUE
        } else if response.hovered() {
            crate::theme::IdmTheme::BUTTON_HOVER
        } else {
            egui::Color32::TRANSPARENT
        };

        if bg != egui::Color32::TRANSPARENT {
            ui.painter().rect_filled(rect, rounding, bg);
        }

        let icon_size = egui::vec2(18.0, 18.0);
        let icon_rect = egui::Rect::from_center_size(rect.center(), icon_size);
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(icon_rect)
                .layout(egui::Layout::centered_and_justified(egui::Direction::TopDown)),
        );
        child_ui.add(img.maintain_aspect_ratio(true));
    }

    response.on_hover_text(tooltip)
}

pub fn render_toolbar(ctx: &egui::Context, app: &mut crate::app::App) {
    let frame = egui::Frame::default()
        .fill(crate::theme::IdmTheme::TOOLBAR_BG)
        .inner_margin(egui::Margin::symmetric(12_i8, 8_i8));

    egui::TopBottomPanel::top("toolbar")
        .frame(frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                // ── Primary Actions (pill buttons with colors) ──
                if pill_button(ui, get_icon_image(47.0, 282.0, 96.0, 92.0), "Add URL", crate::theme::IdmTheme::ACCENT_BLUE).clicked() {
                    app.show_add_url_dialog = true;
                }

                if pill_button(ui, get_icon_image(178.0, 281.0, 77.0, 88.0), "Resume", crate::theme::IdmTheme::STATUS_COMPLETE).clicked() {
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
                                app.downloads[idx].last_try_date = Some(chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string());
                                app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
                            }
                        }
                    }
                }

                if pill_button(ui, get_icon_image(288.0, 282.0, 86.0, 90.0), "Pause", crate::theme::IdmTheme::STATUS_PAUSED).clicked() {
                    for &idx in &app.selected_downloads {
                        if idx < app.downloads.len() {
                            if app.downloads[idx].status == DownloadState::Downloading {
                                app.download_manager.pause_download(app.downloads[idx].id);
                            }
                        }
                    }
                }

                if pill_button(ui, get_icon_image(642.0, 283.0, 73.0, 88.0), "Delete", crate::theme::IdmTheme::STATUS_ERROR).clicked() {
                    let mut to_remove: Vec<usize> = app.selected_downloads.iter().copied().collect();
                    to_remove.sort_unstable_by(|a, b| b.cmp(a));
                    for idx in to_remove {
                        if idx < app.downloads.len() {
                            let d = &app.downloads[idx];
                            let id = d.id;
                            let save_path = std::path::PathBuf::from(&d.save_path);
                            let is_complete = d.status == DownloadState::Complete;
                            app.download_manager.cancel_download(id, !is_complete, &save_path);
                            app.downloads.remove(idx);
                        }
                    }
                    app.selected_downloads.clear();
                }

                ui.add_space(8.0);

                // ── Secondary Actions (icon-only) ──
                if icon_button(ui, get_icon_image(400.0, 287.0, 99.0, 75.0), "Resume All").clicked() {
                    for idx in 0..app.downloads.len() {
                        let d = &app.downloads[idx];
                        if d.status == DownloadState::Paused || d.status == DownloadState::Queued || d.status == DownloadState::Failed {
                            let id = d.id;
                            let url = d.url.clone();
                            let save_path = std::path::PathBuf::from(&d.save_path);
                            let connections = d.connections;
                                let request_context = d.request_context.clone();
                            app.downloads[idx].status = DownloadState::Downloading;
                            app.downloads[idx].last_try_date = Some(chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string());
                            app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
                        }
                    }
                }

                if icon_button(ui, get_icon_image(520.0, 283.0, 85.0, 89.0), "Stop All").clicked() {
                    app.download_manager.stop_all();
                }

                if icon_button(ui, get_icon_image(867.0, 283.0, 90.0, 91.0), "Options").clicked() {
                    app.show_options_dialog = true;
                }

                if icon_button(ui, get_icon_image(981.0, 283.0, 85.0, 86.0), "Scheduler").clicked() {
                    app.show_scheduler_dialog = true;
                }

                // ── Search bar on the right ──
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    let search_edit = egui::TextEdit::singleline(&mut app.search_text)
                        .hint_text("Search...")
                        .desired_width(180.0);
                    ui.add(search_edit);
                });
            });
        });
}
