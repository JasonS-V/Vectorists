//! Modern Visual Theme and Palette for Vectorists.
//!
//! Inspired by high-end precision audio and esports utilities
//! (Linear, Raycast, SoundID Reference, Wootility), featuring deep obsidian surfaces,
//! refined vectorial ice-cyan accents, and strict WCAG AAA typography.

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

/// Optical Carbon & Vectorial Cyan color tokens.
pub mod colors {
    use super::Color32;

    pub const CHASSIS_BG: Color32 = Color32::from_rgb(0x0C, 0x0E, 0x14);
    pub const PANEL_BG: Color32 = Color32::from_rgb(0x13, 0x16, 0x22);
    pub const PANEL_BG_LIGHT: Color32 = Color32::from_rgb(0x18, 0x1C, 0x2A);
    pub const PANEL_ELEVATED: Color32 = Color32::from_rgb(0x1F, 0x24, 0x36);

    // Exact premultiplied alpha tokens for crisp, artifact-free GPU rendering
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(20, 20, 20, 20); // ~8% white
    pub const BORDER_STRONG: Color32 = Color32::from_rgba_premultiplied(40, 40, 40, 40); // ~16% white

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8); // Contrast >7.5:1 (AAA)
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x64, 0x74, 0x8B); // Contrast >4.5:1 (AA)

    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x00, 0xD8, 0xF6);
    pub const ACCENT_CYAN_HOVER: Color32 = Color32::from_rgb(0x38, 0xE1, 0xFF);
    pub const ACCENT_CYAN_FAINT: Color32 = Color32::from_rgba_premultiplied(0, 20, 23, 24);
    pub const ACCENT_CYAN_GLOW: Color32 = Color32::from_rgba_premultiplied(0, 47, 54, 56);

    pub const ALERT_AMBER: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);
    pub const ALERT_AMBER_FAINT: Color32 = Color32::from_rgba_premultiplied(23, 15, 1, 24);

    pub const SUCCESS_GREEN: Color32 = Color32::from_rgb(0x10, 0xB9, 0x81);
    pub const SUCCESS_GREEN_FAINT: Color32 = Color32::from_rgba_premultiplied(2, 17, 12, 24);
}

/// Applies the Vectorists modern visuals and system typography to an egui context.
pub fn apply_theme(ctx: &egui::Context) {
    // 1. Install modern Windows typography (Segoe UI as primary sans-serif)
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    {
        if let Ok(font_bytes) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
            fonts.font_data.insert(
                "segoeui".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(font_bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "segoeui".to_owned());
        }

        // Install clean monospace only as fallback
        let mono_path = if std::path::Path::new("C:\\Windows\\Fonts\\segoeui.ttf").exists() {
            "C:\\Windows\\Fonts\\segoeui.ttf"
        } else {
            "C:\\Windows\\Fonts\\consola.ttf"
        };

        if let Ok(mono_bytes) = std::fs::read(mono_path) {
            fonts.font_data.insert(
                "clean_system".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(mono_bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "clean_system".to_owned());
        }
    }

    ctx.set_fonts(fonts);

    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(colors::TEXT_PRIMARY);
    visuals.panel_fill = colors::CHASSIS_BG;
    visuals.window_fill = colors::PANEL_BG;
    visuals.faint_bg_color = colors::PANEL_BG;
    visuals.extreme_bg_color = colors::CHASSIS_BG;

    // Concentric border radii: 8px outer cards, 6px inner controls
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);
    visuals.widgets.open.corner_radius = CornerRadius::same(6);

    // Inactive widget styling
    visuals.widgets.inactive.bg_fill = colors::PANEL_BG_LIGHT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);

    // Hovered widget styling (crisp accent edge)
    visuals.widgets.hovered.bg_fill = colors::PANEL_ELEVATED;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::BORDER_STRONG);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);

    // Active (pressed) widget styling
    visuals.widgets.active.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);

    // Focus & selection indicators
    visuals.selection.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.selection.stroke = Stroke::new(1.0, colors::ACCENT_CYAN);

    // Crisp, subtle borders on containers
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
