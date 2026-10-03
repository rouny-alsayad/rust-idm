use eframe::egui;
use crate::models::SidebarCategory;

pub fn get_icon_image<'a>(x: f32, y: f32, w: f32, h: f32) -> egui::Image<'a> {
    let uv = egui::Rect::from_min_max(
        egui::pos2(x / 1448.0, y / 1086.0),
        egui::pos2((x + w) / 1448.0, (y + h) / 1086.0),
    );
    egui::Image::new(egui::include_image!("../icon/icons.png"))
        .uv(uv)
}

fn sidebar_item(ui: &mut egui::Ui, app_category: &mut SidebarCategory, category: SidebarCategory, img: egui::Image<'_>, label: &str) {
    let selected = *app_category == category;
    let row_height = 32.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), egui::Sense::click());

    if response.clicked() {
        *app_category = category;
    }

    if ui.is_rect_visible(rect) {
        let rounding = 8.0;

        if selected {
            ui.painter().rect_filled(rect, rounding, crate::theme::IdmTheme::ACCENT_BLUE.linear_multiply(0.25));
            let bar_rect = egui::Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height()));
            ui.painter().rect_filled(bar_rect, 2.0, crate::theme::IdmTheme::ACCENT_BLUE);
        } else if response.hovered() {
            ui.painter().rect_filled(rect, rounding, crate::theme::IdmTheme::HOVERED_ROW);
        }

        let img_size = egui::vec2(20.0, 20.0);
        let img_rect = egui::Rect::from_center_size(
            egui::pos2(rect.min.x + 18.0, rect.center().y),
            img_size,
        );
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(img_rect)
                .layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)),
        );
        child_ui.add(img.maintain_aspect_ratio(true));

        let text_color = if selected {
            crate::theme::IdmTheme::ACCENT_BLUE
        } else {
            crate::theme::IdmTheme::TEXT_PRIMARY
        };

        ui.painter().text(
            egui::pos2(rect.min.x + 36.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.5),
            text_color,
        );
    }
}

pub fn render_sidebar(ctx: &egui::Context, app: &mut crate::app::App) {
    let frame = egui::Frame::default()
        .fill(crate::theme::IdmTheme::SIDEBAR_BG)
        .inner_margin(egui::Margin::symmetric(8_i8, 12_i8));

    egui::SidePanel::left("categories")
        .default_width(180.0)
        .width_range(160.0..=240.0)
        .frame(frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Categories")
                        .size(11.0)
                        .color(crate::theme::IdmTheme::TEXT_SECONDARY),
                );
            });
            ui.add_space(8.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                sidebar_item(ui, &mut app.selected_category, SidebarCategory::AllDownloads, get_icon_image(46.0, 468.0, 86.0, 85.0), "All Downloads");
                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Compressed, get_icon_image(173.0, 465.0, 87.0, 90.0), "Compressed");
                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Documents, get_icon_image(316.0, 462.0, 75.0, 90.0), "Documents");
                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Music, get_icon_image(443.0, 466.0, 80.0, 88.0), "Music");
                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Programs, get_icon_image(574.0, 462.0, 88.0, 89.0), "Programs");
                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Video, get_icon_image(705.0, 464.0, 85.0, 93.0), "Video");

                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Status")
                            .size(11.0)
                            .color(crate::theme::IdmTheme::TEXT_SECONDARY),
                    );
                });
                ui.add_space(4.0);

                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Unfinished, get_icon_image(867.0, 469.0, 86.0, 86.0), "Unfinished");
                sidebar_item(ui, &mut app.selected_category, SidebarCategory::Finished, get_icon_image(1007.0, 475.0, 99.0, 76.0), "Finished");
            });
        });
}
