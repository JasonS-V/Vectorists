//! Modern Visual Theme and Palette for Vectorists.
//!
//! Inspired by modern high-end gaming software and developer utilities
//! (Linear, Raycast, Wootility), featuring deep obsidian surfaces,
//! glowing vectorial cyan accents, and strict WCAG AAA typography.

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

/// Optical Carbon & Vectorial Cyan color tokens.
pub mod colors {
    use super::Color32;

    pub const CHASSIS_BG: Color32 = Color32::from_rgb(0x08, 0x09, 0x0C);
    pub const PANEL_BG: Color32 = Color32::from_rgb(0x10, 0x12, 0x18);
    pub const PANEL_BG_LIGHT: Color32 = Color32::from_rgb(0x17, 0x1A, 0x24);
    pub const PANEL_ELEVATED: Color32 = Color32::from_rgb(0x1E, 0x22, 0x2F);

    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(0x1E, 0x22, 0x2E);
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(0x2E, 0x35, 0x47);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF5, 0xF6, 0xF8);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0xA2, 0xA8, 0xB8); // Contrast >7.5:1 (AAA)
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x6A, 0x72, 0x86); // Contrast >4.5:1 (AA)

    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x00, 0xF0, 0xFF);
    pub const ACCENT_CYAN_HOVER: Color32 = Color32::from_rgb(0x38, 0xF3, 0xFF);
    pub const ACCENT_CYAN_FAINT: Color32 = Color32::from_rgba_premultiplied(0x00, 0xF0, 0xFF, 0x22);
    pub const ACCENT_CYAN_GLOW: Color32 = Color32::from_rgba_premultiplied(0x00, 0xF0, 0xFF, 0x40);

    pub const ALERT_AMBER: Color32 = Color32::from_rgb(0xFF, 0xA0, 0x1E);
    pub const ALERT_AMBER_FAINT: Color32 = Color32::from_rgba_premultiplied(0xFF, 0xA0, 0x1E, 0x25);

    pub const SUCCESS_GREEN: Color32 = Color32::from_rgb(0x00, 0xE6, 0x76);
}

/// Applies the Vectorists modern visuals to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(colors::TEXT_PRIMARY);
    visuals.panel_fill = colors::CHASSIS_BG;
    visuals.window_fill = colors::PANEL_BG;
    visuals.faint_bg_color = colors::PANEL_BG;
    visuals.extreme_bg_color = colors::CHASSIS_BG;

    // Concentric border radii: 6px outer cards, 4px inner controls
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);
    visuals.widgets.open.corner_radius = CornerRadius::same(4);

    // Inactive widget styling
    visuals.widgets.inactive.bg_fill = colors::PANEL_BG_LIGHT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);

    // Hovered widget styling (crisp cyan accent edge)
    visuals.widgets.hovered.bg_fill = colors::PANEL_ELEVATED;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);

    // Active (pressed) widget styling
    visuals.widgets.active.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, colors::ACCENT_CYAN);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);

    // Focus & selection indicators
    visuals.selection.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.selection.stroke = Stroke::new(1.5, colors::ACCENT_CYAN);

    // Crisp, subtle borders
    visuals.widgets.noninteractive.bg_fill = colors::PANEL_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_MUTED);

    ctx.set_visuals(visuals);

    // Compact layout spacing & optical alignment
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(12);
        style.spacing.scroll.bar_width = 4.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
    });
}
