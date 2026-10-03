use eframe::egui;
use crate::app::App;
use crate::theme::IdmTheme;

pub fn render_move_rename_dialog(ctx: &egui::Context, app: &mut App) {
    let mut is_open = app.show_move_rename_dialog;

    if app.move_rename_target.is_none() || app.move_rename_target.unwrap() >= app.downloads.len() {
        app.show_move_rename_dialog = false;
        app.move_rename_target = None;
        return;
    }

    let title = if app.move_rename_mode == 2 { "Rename File" } else { "Move File" };

    egui::Window::new(title)
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            let target_idx = app.move_rename_target.unwrap();

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let label = if app.move_rename_mode == 2 { "New name:" } else { "New directory:" };
                ui.label(egui::RichText::new(label).color(IdmTheme::TEXT_PRIMARY));
            });

            ui.horizontal(|ui| {
                let text_edit = egui::TextEdit::singleline(&mut app.move_rename_text).desired_width(320.0);
                ui.add(text_edit);

                if app.move_rename_mode == 1 {
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Directory")
                            .set_directory(&app.move_rename_text)
                            .pick_folder()
                        {
                            app.move_rename_text = path.to_string_lossy().to_string();
                        }
                    }
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Cancel").clicked() {
                    app.show_move_rename_dialog = false;
                    app.move_rename_target = None;
                }
                if ui.button("  OK  ").clicked() {
                    let old_path = app.downloads[target_idx].save_path.clone();
                    let old_p = std::path::Path::new(&old_path);
                    
                    let mut new_p = std::path::PathBuf::from(&old_path);
                    
                    if app.move_rename_mode == 2 {
                        // Rename mode: keep dir, change file name
                        new_p.set_file_name(&app.move_rename_text);
                    } else {
                        // Move mode: keep file name, change dir
                        let old_name = old_p.file_name().unwrap_or_default();
                        new_p = std::path::PathBuf::from(&app.move_rename_text).join(old_name);
                    }

                    if old_p != new_p {
                        if let Some(parent) = new_p.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }

                        if old_p.exists() {
                            let _ = std::fs::rename(old_p, &new_p);
                        }

                        let old_idm = old_p.with_extension("idm");
                        let new_idm = new_p.with_extension("idm");
                        if old_idm.exists() {
                            let _ = std::fs::rename(old_idm, new_idm);
                        }

                        let file_name = new_p.file_name().unwrap_or_default().to_string_lossy().to_string();
                        app.downloads[target_idx].save_path = new_p.to_string_lossy().to_string();
                        app.downloads[target_idx].file_name = file_name;
                    }

                    app.show_move_rename_dialog = false;
                    app.move_rename_target = None;
                }
            });
        });

    if !is_open {
        app.show_move_rename_dialog = false;
        app.move_rename_target = None;
    }
}
