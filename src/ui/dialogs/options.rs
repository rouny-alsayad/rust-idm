use eframe::egui;
use crate::app::App;
use crate::models::*;

pub fn render_options_dialog(ctx: &egui::Context, app: &mut App) {
    let mut open = app.show_options_dialog;
    
    egui::Window::new("Internet Download Manager Configuration")
        .open(&mut open)
        .resizable(true)
        .default_size([560.0, 500.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // Bottom buttons
            egui::TopBottomPanel::bottom("options_bottom_buttons")
                .resizable(false)
                .show_inside(ui, |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Help").clicked() {
                                // Help action
                            }
                            if ui.button("OK").clicked() {
                                app.show_options_dialog = false;
                            }
                        });
                    });
                    ui.add_space(4.0);
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                // Tabs
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let tabs = [
                            "General",
                            "File types",
                            "Save to",
                            "Downloads",
                            "Connection",
                            "Proxy / Socks",
                            "Sites Logins",
                            "Dial Up / VPN",
                            "Sounds",
                        ];

                        for (i, &name) in tabs.iter().enumerate() {
                            if ui.selectable_label(app.options_tab == i, name).clicked() {
                                app.options_tab = i;
                            }
                        }
                    });
                });
                
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);
                    match app.options_tab {
                        0 => render_tab_general(ui, app),
                        1 => render_tab_file_types(ui, app),
                        2 => render_tab_save_to(ui, app),
                        3 => render_tab_downloads(ui, app),
                        4 => render_tab_connection(ui, app),
                        5 => render_tab_proxy(ui, app),
                        6 => render_tab_sites_logins(ui, app),
                        7 => render_tab_dial_up(ui, app),
                        8 => render_tab_sounds(ui, app),
                        _ => {}
                    }
                });
            });
        });
        
    app.show_options_dialog = open;
}

fn render_tab_general(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Browser/System Integration");
    ui.add_space(8.0);
    
    ui.checkbox(&mut app.config.general.launch_on_startup, "Launch Internet Download Manager on startup");
    ui.checkbox(&mut app.config.general.clipboard_monitoring, "Automatically start downloading of URLs placed to clipboard");
    
    ui.add_space(12.0);
    ui.heading("Capture downloads from the following browsers:");
    ui.add_space(4.0);
    
    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
        for browser in &mut app.config.general.browsers {
            ui.checkbox(&mut browser.enabled, &browser.name);
        }
    });
    
    ui.add_space(8.0);
    if ui.button("Add browser...").clicked() {
        // Add browser logic
    }
}

fn render_tab_file_types(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Downloaded file types");
    ui.add_space(8.0);
    
    ui.label("Automatically start downloading the following file types:");
    ui.add_space(4.0);
    ui.add(egui::TextEdit::multiline(&mut app.config.file_types.auto_download_types).desired_width(f32::INFINITY));
    if ui.button("Default").clicked() {
        // Reset defaults
    }
    
    ui.add_space(12.0);
    ui.label("Don't start downloading automatically from the following sites:");
    ui.add_space(4.0);
    ui.add(egui::TextEdit::multiline(&mut app.config.file_types.exception_sites).desired_width(f32::INFINITY));
    if ui.button("Default").clicked() {
        // Reset defaults
    }
    
    ui.add_space(12.0);
    ui.label("Don't start downloading automatically from the following addresses:");
    ui.add_space(4.0);
    if ui.button("Edit list...").clicked() {
        // Edit list logic
    }
}

fn render_tab_save_to(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Categories, file types, folders");
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        ui.label("Category:");
        let categories = ["General", "Compressed", "Documents", "Music", "Programs", "Video"];
        egui::ComboBox::from_id_source("save_category_combo")
            .selected_text(&app.config.save.category)
            .show_ui(ui, |ui| {
                for cat in categories {
                    ui.selectable_value(&mut app.config.save.category, cat.to_string(), cat);
                }
            });
            
        if ui.button("New").clicked() {}
        if ui.button("Edit...").clicked() {}
    });
    
    ui.add_space(12.0);
    ui.label(format!("Default download directory for {} category", app.config.save.category));
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut app.config.save.default_directory).desired_width(350.0));
        if ui.button("Browse...").clicked() {}
    });
    
    ui.add_space(8.0);
    ui.checkbox(&mut app.config.save.change_folder_on_last_selected, "Change folder for category on last selected");
    ui.checkbox(&mut app.config.save.set_server_date, "Set file creation date as provided by the server");
    
    ui.add_space(12.0);
    ui.label("Temporary directory");
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut app.config.save.temp_directory).desired_width(350.0));
        if ui.button("Browse...").clicked() {}
    });
}

fn render_tab_downloads(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Default download settings");
    ui.add_space(8.0);
    
    if ui.button("Customize 'Download progress' dialog").clicked() {
        // Edit logic
    }
    
    ui.add_space(8.0);
    ui.checkbox(&mut app.config.download_behavior.show_start_dialog, "Show start download dialog");
    ui.checkbox(&mut app.config.download_behavior.show_complete_dialog, "Show download complete dialog");
    ui.checkbox(&mut app.config.download_behavior.start_immediately, "Start downloading immediately");
    ui.checkbox(&mut app.config.download_behavior.show_queue_on_later, "Show queue selection panel on 'Download Later'");
    ui.checkbox(&mut app.config.download_behavior.show_queue_on_batch_close, "Show queue selection panel on closing batch download dialogs");
    ui.checkbox(&mut app.config.download_behavior.ignore_time_changes, "Ignore file modification time changes when resuming");
    
    ui.add_space(12.0);
    ui.label("If a duplicate download link is added:");
    let dup_options = [
        ("ask", "Show a dialog and ask what to do."),
        ("numbered", "Add the duplicate with a numbered file name"),
        ("overwrite", "Add the duplicate and overwrite the existing file"),
        ("resume", "If existing file is complete, show complete dialog; otherwise resume."),
    ];
    let selected_text = dup_options.iter()
        .find(|(k, _)| *k == app.config.download_behavior.duplicate_action.as_str())
        .map(|(_, v)| *v)
        .unwrap_or(&"Show a dialog and ask what to do.");
    
    egui::ComboBox::from_id_source("duplicate_combo")
        .selected_text(selected_text.to_string())
        .show_ui(ui, |ui| {
            for (key, label) in dup_options {
                ui.selectable_value(&mut app.config.download_behavior.duplicate_action, key.to_string(), label);
            }
        });
        
    ui.add_space(12.0);
    ui.label("User-Agent for manually added downloads:");
    ui.add(egui::TextEdit::singleline(&mut app.config.download_behavior.user_agent).desired_width(350.0));
}

fn render_tab_connection(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Connections and Limits");
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        ui.label("Default max. conn. number:");
        let conn_opts: [u8; 7] = [1, 2, 4, 8, 16, 24, 32];
        egui::ComboBox::from_id_source("max_conn_combo")
            .selected_text(app.config.connection.max_connections.to_string())
            .show_ui(ui, |ui| {
                for opt in conn_opts {
                    ui.selectable_value(&mut app.config.connection.max_connections, opt, opt.to_string());
                }
            });
    });
    
    ui.add_space(12.0);
    ui.group(|ui| {
        ui.label("Exceptions:");
        egui::Grid::new("conn_exceptions_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.label("Server");
                ui.label("Number");
                ui.end_row();
                
                for exc in &app.config.connection.exceptions {
                    ui.label(&exc.server);
                    ui.label(exc.number.to_string());
                    ui.end_row();
                }
            });
        
        ui.horizontal(|ui| {
            if ui.button("New").clicked() {}
            if ui.button("Delete").clicked() {}
            if ui.button("Edit").clicked() {}
        });
    });
    
    ui.add_space(12.0);
    ui.checkbox(&mut app.config.connection.download_limit_enabled, "Download limits");
    if app.config.connection.download_limit_enabled {
        ui.horizontal(|ui| {
            ui.label("Download no more than");
            ui.add(egui::DragValue::new(&mut app.config.connection.download_limit_mb).speed(1));
            ui.label("MBytes every");
            ui.add(egui::DragValue::new(&mut app.config.connection.download_limit_hours).speed(1));
            ui.label("hours");
        });
    }
    
    ui.add_space(8.0);
    ui.checkbox(&mut app.config.connection.show_limit_warning, "Show warning before stopping downloads");
}

fn render_tab_proxy(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Proxy / socks configuration");
    ui.add_space(8.0);
    
    if ui.button("Get System").clicked() {}
    
    ui.add_space(8.0);
    ui.checkbox(&mut app.config.proxy.use_browser_proxy, "Use proxy/socks from browser's request...");
    
    ui.add_space(12.0);
    ui.radio_value(&mut app.config.proxy.mode, "none".to_string(), "No proxy/socks");
    ui.radio_value(&mut app.config.proxy.mode, "system".to_string(), "Use system settings");
    ui.horizontal(|ui| {
        ui.radio_value(&mut app.config.proxy.mode, "auto".to_string(), "Use automatic configuration script");
        ui.add(egui::TextEdit::singleline(&mut app.config.proxy.auto_config_url).desired_width(200.0));
    });
    
    ui.radio_value(&mut app.config.proxy.mode, "manual".to_string(), "Manual proxy/socks configuration");
    if app.config.proxy.mode == "manual" {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Proxy server address:");
                ui.add(egui::TextEdit::singleline(&mut app.config.proxy.proxy_address).desired_width(150.0));
                ui.label("Port:");
                ui.add(egui::TextEdit::singleline(&mut app.config.proxy.proxy_port).desired_width(50.0));
            });
            ui.horizontal(|ui| {
                ui.label("UserName:");
                ui.add(egui::TextEdit::singleline(&mut app.config.proxy.proxy_user).desired_width(100.0));
                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut app.config.proxy.proxy_pass).desired_width(100.0));
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.config.proxy.use_http, "http");
                ui.checkbox(&mut app.config.proxy.use_https, "https");
                ui.checkbox(&mut app.config.proxy.use_ftp, "ftp");
            });
            if ui.button("Advanced / Socks...").clicked() {}
        });
    }
    
    ui.add_space(8.0);
    ui.checkbox(&mut app.config.proxy.use_pasv, "Use FTP in PASV mode");
}

fn render_tab_sites_logins(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("User names and passwords for servers/sites");
    ui.add_space(8.0);
    
    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        egui::Grid::new("logins_grid")
            .striped(true)
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.label("Site/path");
                ui.label("User");
                ui.label("Password");
                ui.end_row();
                
                for login in &app.config.site_logins {
                    ui.label(format!("{}{}", login.protocol, login.server_path));
                    ui.label(&login.user);
                    ui.label("********");
                    ui.end_row();
                }
            });
    });
    
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("New").clicked() {}
        if ui.button("Edit").clicked() {}
        if ui.button("Remove").clicked() {}
    });
}

fn render_tab_dial_up(ui: &mut egui::Ui, app: &mut App) {
    ui.heading("Dial up / VPN settings");
    ui.add_space(8.0);
    
    ui.checkbox(&mut app.config.dial_up.enabled, "Use Windows Dial Up / VPN Networking");
    
    ui.add_space(12.0);
    ui.group(|ui| {
        ui.label("Connection options:");
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Connection:");
            egui::ComboBox::from_id_source("vpn_conn_combo")
                .selected_text("")
                .show_ui(ui, |_ui| {});
        });
        
        ui.horizontal(|ui| {
            ui.label("User name:");
            let mut u = "".to_string();
            ui.add(egui::TextEdit::singleline(&mut u).desired_width(120.0));
            ui.label("Password:");
            let mut p = "".to_string();
            ui.add(egui::TextEdit::singleline(&mut p).desired_width(120.0));
        });
        
        ui.horizontal(|ui| {
            let mut save_pw = true;
            ui.checkbox(&mut save_pw, "Save password");
            if ui.button("Apply").clicked() {}
        });
    });
    
    ui.add_space(12.0);
    ui.group(|ui| {
        ui.label("Redial options:");
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Redial attempts (zero if endlessly):");
            ui.add(egui::DragValue::new(&mut app.config.dial_up.redial_attempts));
        });
        
        ui.horizontal(|ui| {
            ui.label("Time between redial attempts:");
            ui.add(egui::DragValue::new(&mut app.config.dial_up.redial_interval));
            ui.label("seconds");
        });
    });
}

fn render_tab_sounds(ui: &mut egui::Ui, _app: &mut App) {
    ui.heading("Sound settings");
    ui.add_space(8.0);
    
    ui.label("Select sounds for Internet Download Manager events");
    ui.add_space(4.0);
    
    let events = [
        "Download complete",
        "Download failed",
        "Queue processing started",
        "Queue processing stopped/finished"
    ];
    
    egui::Grid::new("sounds_grid")
        .striped(true)
        .min_col_width(200.0)
        .show(ui, |ui| {
            ui.label("Event");
            ui.label("Sound file");
            ui.end_row();
            
            for ev in events {
                ui.horizontal(|ui| {
                    let mut b = false;
                    ui.checkbox(&mut b, "");
                    ui.label(ev);
                });
                ui.label(""); // Path would go here
                ui.end_row();
            }
        });
        
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Browse...").clicked() {}
        if ui.button("Play").clicked() {}
    });
}
