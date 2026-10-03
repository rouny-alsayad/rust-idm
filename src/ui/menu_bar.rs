use eframe::egui;
use crate::engine::types::DownloadState;

pub fn render_menu_bar(ctx: &egui::Context, app: &mut crate::app::App) {
    egui::TopBottomPanel::top("menu_bar")
        .frame(egui::Frame::default().fill(crate::theme::IdmTheme::MENU_BAR_BG))
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.visuals_mut().button_frame = false;

                ui.menu_button("Tasks", |ui| {
                    if ui.button("Add new download").clicked() {
                        app.show_add_url_dialog = true;
                        ui.close_menu();
                    }
                    let _ = ui.button("Add batch download");
                    let _ = ui.button("Add batch download from clipboard");
                    let _ = ui.button("Run site grabber");
                    ui.separator();
                    let _ = ui.button("Show drop target");
                    ui.separator();
                    ui.menu_button("Export", |ui| {
                        let _ = ui.button("Export download list");
                    });
                    ui.menu_button("Import", |ui| {
                        let _ = ui.button("Import download list");
                    });
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        app.download_manager.stop_all();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("File", |ui| {
                    let has_selection = !app.selected_downloads.is_empty();

                    if ui.add_enabled(has_selection, egui::Button::new("Resume")).clicked() {
                        for &idx in &app.selected_downloads {
                            if idx < app.downloads.len() {
                                let d = &app.downloads[idx];
                                if d.status == DownloadState::Paused || d.status == DownloadState::Queued {
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
                        ui.close_menu();
                    }

                    if ui.add_enabled(has_selection, egui::Button::new("Stop Download")).clicked() {
                        for &idx in &app.selected_downloads {
                            if idx < app.downloads.len() {
                                let d = &app.downloads[idx];
                                if d.status == DownloadState::Downloading {
                                    app.download_manager.pause_download(d.id);
                                }
                            }
                        }
                        ui.close_menu();
                    }

                    if ui.add_enabled(has_selection, egui::Button::new("Remove")).clicked() {
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
                        ui.close_menu();
                    }

                    ui.add_enabled(has_selection, egui::Button::new("Redownload"));
                    if ui.add_enabled(has_selection, egui::Button::new("Redownload")).clicked() {
                        let mut to_redownload: Vec<usize> = app.selected_downloads.iter().copied().collect();
                        for idx in to_redownload {
                            if idx < app.downloads.len() {
                                let d = &app.downloads[idx];
                                let id = d.id;
                                let save_path = std::path::PathBuf::from(&d.save_path);

                                if d.status == DownloadState::Downloading {
                                    app.download_manager.pause_download(id);
                                }

                                let _ = std::fs::remove_file(&save_path);
                                let _ = std::fs::remove_file(save_path.with_extension("idm"));

                                let url = d.url.clone();
                                let connections = d.connections;
                                let request_context = d.request_context.clone();
                                app.downloads[idx].downloaded = 0;
                                app.downloads[idx].status = DownloadState::Downloading;
                                app.downloads[idx].last_try_date = Some(
                                    chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string()
                                );

                                app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
                            }
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("Downloads", |ui| {
                    if ui.button("Resume All").clicked() {
                        for idx in 0..app.downloads.len() {
                            let d = &app.downloads[idx];
                            if d.status == DownloadState::Paused || d.status == DownloadState::Queued {
                                let id = d.id;
                                let url = d.url.clone();
                                let save_path = std::path::PathBuf::from(&d.save_path);
                                let connections = d.connections;
                                let request_context = d.request_context.clone();
                                app.downloads[idx].status = DownloadState::Downloading;
                                app.downloads[idx].last_try_date = Some(
                                    chrono::Local::now().format("%b %d %H:%M:%S %Y").to_string()
                                );
                                app.download_manager.resume_download_with_context(id, url, save_path, connections, request_context);
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Pause All").clicked() {
                        app.download_manager.stop_all();
                        ui.close_menu();
                    }
                    if ui.button("Stop All").clicked() {
                        app.download_manager.stop_all();
                        ui.close_menu();
                    }
                    ui.separator();

                    if ui.button("Delete All Completed").clicked() {
                        app.downloads.retain(|d| d.status != DownloadState::Complete);
                        app.selected_downloads.clear();
                        ui.close_menu();
                    }

                    ui.separator();
                    if ui.button("Find (Ctrl+F)").clicked() {
                        app.show_search = true;
                        ui.close_menu();
                    }
                    let _ = ui.button("Find Next (F3)");
                    ui.separator();
                    if ui.button("Scheduler").clicked() {
                        app.show_scheduler_dialog = true;
                        ui.close_menu();
                    }
                    ui.menu_button("Start queue", |ui| {
                        if ui.button("Start Main download queue").clicked() {
                            let queued: Vec<(u64, String, String, u8, crate::engine::types::RequestContext)> = app.downloads.iter()
                                .filter(|d| d.status == DownloadState::Queued)
                                .map(|d| (d.id, d.url.clone(), d.save_path.clone(), d.connections, d.request_context.clone()))
                                .collect();
                            for (id, url, path, connections, request_context) in queued {
                                if let Some(item) = app.downloads.iter_mut().find(|d| d.id == id) {
                                    item.status = DownloadState::Downloading;
                                }
                                app.download_manager.start_download_with_context(id, url, std::path::PathBuf::from(path), connections, request_context);
                            }
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Stop queue", |ui| {
                        if ui.button("Stop Main download queue").clicked() {
                            app.download_manager.stop_all();
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Speed Limiter", |ui| {
                        let _ = ui.button("Turn on");
                        let _ = ui.button("Turn off");
                        ui.separator();
                        let _ = ui.button("Settings...");
                    });
                    ui.separator();
                    if ui.button("Options").clicked() {
                        app.show_options_dialog = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    let mut b1 = true;
                    ui.checkbox(&mut b1, "Toolbar");
                    let mut b2 = true;
                    ui.checkbox(&mut b2, "Categories");
                    ui.separator();
                    ui.menu_button("Columns", |ui| {
                        let _ = ui.button("File Name");
                        let _ = ui.button("Size");
                    });
                });

                ui.menu_button("Help", |ui| {
                    let _ = ui.button("About Internet Download Manager");
                });
            });
        });
}
