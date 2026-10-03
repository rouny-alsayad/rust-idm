use eframe::egui;
use crate::models::{Category, DownloadItem}; use crate::engine::types::DownloadState;

pub fn render_add_url_dialog(ctx: &egui::Context, app: &mut crate::app::App) {
    let mut open = app.show_add_url_dialog;
    if !open {
        return;
    }

    egui::Window::new("Enter new address to download")
        .open(&mut open)
        .resizable(false)
        .default_size([500.0, 180.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Address:");
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut app.add_url_state.url),
                );
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.checkbox(&mut app.add_url_state.use_auth, "Use authorization");
                if app.add_url_state.use_auth {
                    ui.horizontal(|ui| {
                        ui.label("Login:");
                        ui.text_edit_singleline(&mut app.add_url_state.login);
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut app.add_url_state.password).password(true));
                    });
                }
            });

            ui.add_space(8.0);

            // Save location
            ui.horizontal(|ui| {
                ui.label("Save to:");
                let save_dir = &app.config.save.default_directory;
                ui.label(
                    egui::RichText::new(if save_dir.is_empty() { "Downloads" } else { save_dir })
                        .color(crate::theme::IdmTheme::TEXT_SECONDARY),
                );
            });

            ui.add_space(16.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    app.show_add_url_dialog = false;
                    app.add_url_state.url.clear();
                    app.add_url_state.use_auth = false;
                    app.add_url_state.login.clear();
                    app.add_url_state.password.clear();
                }
                if ui.button("Download Now").clicked() {
                    let url = app.add_url_state.url.trim().to_string();
                    if !url.is_empty() {
                        let file_name = url
                            .split('?').next().unwrap_or(&url)
                            .split('/').last()
                            .filter(|s| !s.is_empty())
                            .unwrap_or("download")
                            .to_string();

                        let ext = file_name.rsplit('.').next().unwrap_or("");
                        let category = Category::from_extension(ext);
                        let save_path = app.get_save_path(&file_name);
                        let id = app.next_download_id();
                        let request_context = if app.add_url_state.use_auth {
                            crate::engine::types::RequestContext {
                                basic_username: Some(app.add_url_state.login.clone()),
                                basic_password: Some(app.add_url_state.password.clone()),
                                ..Default::default()
                            }
                        } else {
                            crate::engine::types::RequestContext::default()
                        };

                        let new_item = DownloadItem {
                            id,
                            file_name: file_name.clone(),
                            url: url.clone(),
                            status: DownloadState::Downloading,
                            category,
                            last_try_date: Some(
                                chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string()
                            ),
                            save_path: save_path.to_string_lossy().to_string(),
                            connections: app.config.connection.max_connections,
                            request_context: request_context.clone(),
                            ..Default::default()
                        };
                        app.downloads.push(new_item);

                        app.download_manager.start_download_with_context(
                            id,
                            url,
                            save_path,
                            app.config.connection.max_connections,
                            request_context,
                        );

                        app.add_url_state.url.clear();
                        app.add_url_state.use_auth = false;
                        app.add_url_state.login.clear();
                        app.add_url_state.password.clear();
                    }
                    app.show_add_url_dialog = false;
                }
                if ui.button("Download Later").clicked() {
                    let url = app.add_url_state.url.trim().to_string();
                    if !url.is_empty() {
                        let file_name = url
                            .split('?').next().unwrap_or(&url)
                            .split('/').last()
                            .filter(|s| !s.is_empty())
                            .unwrap_or("download")
                            .to_string();
                        let ext = file_name.rsplit('.').next().unwrap_or("");
                        let category = Category::from_extension(ext);
                        let save_path = app.get_save_path(&file_name);
                        let id = app.next_download_id();
                        let request_context = if app.add_url_state.use_auth {
                            crate::engine::types::RequestContext {
                                basic_username: Some(app.add_url_state.login.clone()),
                                basic_password: Some(app.add_url_state.password.clone()),
                                ..Default::default()
                            }
                        } else {
                            crate::engine::types::RequestContext::default()
                        };

                        let new_item = DownloadItem {
                            id,
                            file_name,
                            url,
                            status: DownloadState::Queued,
                            category,
                            last_try_date: Some(
                                chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string()
                            ),
                            save_path: save_path.to_string_lossy().to_string(),
                            connections: app.config.connection.max_connections,
                            request_context: request_context.clone(),
                            ..Default::default()
                        };
                        app.downloads.push(new_item);

                        app.add_url_state.url.clear();
                        app.add_url_state.use_auth = false;
                        app.add_url_state.login.clear();
                        app.add_url_state.password.clear();
                    }
                    app.show_add_url_dialog = false;
                }
            });
        });

    app.show_add_url_dialog = open;
}
