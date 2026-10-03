use eframe::egui;

/// Modern dark theme — inspired by Arc, Figma, Discord
pub struct IdmTheme;

impl IdmTheme {
    // ── Backgrounds ──────────────────────────────────────────
    pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(18, 18, 26);
    pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(30, 30, 46);
    pub const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(22, 22, 42);
    pub const TOOLBAR_BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 34);
    pub const CARD_BG: egui::Color32 = egui::Color32::from_rgb(26, 26, 42);

    // ── Table ────────────────────────────────────────────────
    pub const TABLE_ROW_EVEN: egui::Color32 = egui::Color32::from_rgb(22, 22, 36);
    pub const TABLE_ROW_ODD: egui::Color32 = egui::Color32::from_rgb(26, 26, 42);
    pub const TABLE_HEADER_BG: egui::Color32 = egui::Color32::from_rgb(18, 18, 30);
    pub const SELECTED_ROW: egui::Color32 = egui::Color32::from_rgb(42, 58, 94);
    pub const HOVERED_ROW: egui::Color32 = egui::Color32::from_rgb(34, 34, 58);

    // ── Text ─────────────────────────────────────────────────
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(232, 232, 240);
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(110, 110, 138);

    // ── Accent ───────────────────────────────────────────────
    pub const ACCENT_BLUE: egui::Color32 = egui::Color32::from_rgb(67, 97, 238);
    pub const ACCENT_PURPLE: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);

    // ── Borders ──────────────────────────────────────────────
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(42, 42, 62);

    // ── Buttons ──────────────────────────────────────────────
    pub const BUTTON_BG: egui::Color32 = egui::Color32::from_rgb(42, 42, 64);
    pub const BUTTON_HOVER: egui::Color32 = egui::Color32::from_rgb(58, 58, 85);

    // ── Status ───────────────────────────────────────────────
    pub const STATUS_COMPLETE: egui::Color32 = egui::Color32::from_rgb(6, 214, 160);
    pub const STATUS_ERROR: egui::Color32 = egui::Color32::from_rgb(239, 71, 111);
    pub const STATUS_ACTIVE: egui::Color32 = egui::Color32::from_rgb(67, 97, 238);
    pub const STATUS_PAUSED: egui::Color32 = egui::Color32::from_rgb(255, 209, 102);

    // ── Progress ─────────────────────────────────────────────
    pub const PROGRESS_BG: egui::Color32 = egui::Color32::from_rgb(36, 36, 56);
    pub const PROGRESS_FILL: egui::Color32 = egui::Color32::from_rgb(67, 97, 238);

    // ── Menu Bar ─────────────────────────────────────────────
    pub const MENU_BAR_BG: egui::Color32 = egui::Color32::from_rgb(16, 16, 24);

    /// Apply the modern dark theme
    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.override_text_color = Some(Self::TEXT_PRIMARY);

        visuals.window_fill = Self::SURFACE;
        visuals.panel_fill = Self::BG_DARK;
        visuals.faint_bg_color = Self::TABLE_ROW_ODD;
        visuals.extreme_bg_color = egui::Color32::from_rgb(14, 14, 22);

        // Widgets
        visuals.widgets.noninteractive.bg_fill = Self::BG_DARK;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, Self::BORDER);

        visuals.widgets.inactive.bg_fill = Self::BUTTON_BG;
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Self::BORDER);

        visuals.widgets.hovered.bg_fill = Self::BUTTON_HOVER;
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Self::ACCENT_BLUE);

        visuals.widgets.active.bg_fill = Self::ACCENT_BLUE;
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, Self::ACCENT_BLUE);

        visuals.selection.bg_fill = Self::SELECTED_ROW;
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, Self::ACCENT_BLUE);

        // Window styling
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.window_stroke = egui::Stroke::new(1.0_f32, Self::BORDER);

        // Spacing — generous
        style.spacing.item_spacing = egui::vec2(10.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(16);

        ctx.set_style(style);
    }
}
