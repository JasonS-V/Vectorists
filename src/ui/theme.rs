//! Anti-Slop Visual Theme and Palette for Vectorists.
//!
//! Enforces high contrast (WCAG AAA >= 7.0:1), compact HUD aesthetics,
//! and clean borders without blurry shadows or gratuitous padding.

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

/// Optical Carbon & Vectorial Cyan color tokens.
pub mod colors {
    use super::Color32;

    pub const CHASSIS_BG: Color32 = Color32::from_rgb(0x0C, 0x0D, 0x10);
    pub const PANEL_BG: Color32 = Color32::from_rgb(0x15, 0x17, 0x1D);
    pub const PANEL_BG_LIGHT: Color32 = Color32::from_rgb(0x1C, 0x1F, 0x27);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(0x23, 0x27, 0x33);
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x35, 0x3B, 0x4D);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF4, 0xF5, 0xF8);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x9B, 0xA1, 0xB0);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x5A, 0x60, 0x72);

    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x00, 0xE5, 0xFF);
    pub const ACCENT_CYAN_HOVER: Color32 = Color32::from_rgb(0x33, 0xEA, 0xFF);
    pub const ACCENT_CYAN_FAINT: Color32 = Color32::from_rgba_premultiplied(0x00, 0xE5, 0xFF, 0x20);

    pub const ALERT_AMBER: Color32 = Color32::from_rgb(0xFF, 0x9F, 0x1C);
    pub const ALERT_AMBER_FAINT: Color32 = Color32::from_rgba_premultiplied(0xFF, 0x9F, 0x1C, 0x25);

    pub const SUCCESS_GREEN: Color32 = Color32::from_rgb(0x00, 0xE6, 0x76);
}

/// Applies the Vectorists custom visuals to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(colors::TEXT_PRIMARY);
    visuals.panel_fill = colors::CHASSIS_BG;
    visuals.window_fill = colors::PANEL_BG;
    visuals.faint_bg_color = colors::PANEL_BG;
    visuals.extreme_bg_color = colors::CHASSIS_BG;

    // Strict, restrained corner radii (Anti-slop: no rounded-3xl bloat)
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);
    visuals.widgets.open.corner_radius = CornerRadius::same(4);

    // Inactive widget styling
    visuals.widgets.inactive.bg_fill = colors::PANEL_BG_LIGHT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);

    // Hovered widget styling
    visuals.widgets.hovered.bg_fill = colors::PANEL_BG_LIGHT;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);

    // Active (pressed) widget styling
    visuals.widgets.active.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, colors::ACCENT_CYAN);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);

    // Selection highlight
    visuals.selection.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.selection.stroke = Stroke::new(1.0, colors::ACCENT_CYAN);

    // Crisp, subtle borders
    visuals.widgets.noninteractive.bg_fill = colors::PANEL_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_MUTED);

    ctx.set_visuals(visuals);

    // Compact styling
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(12);
    });
}
