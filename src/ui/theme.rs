//! Shared visual tokens for the native precision workspace.
use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

pub mod colors {
    use super::Color32;
    pub const CHASSIS_BG: Color32 = Color32::from_rgb(13, 16, 20);
    pub const SIDEBAR_BG: Color32 = Color32::from_rgb(17, 21, 26);
    pub const PANEL_BG: Color32 = Color32::from_rgb(22, 27, 33);
    pub const PANEL_BG_LIGHT: Color32 = Color32::from_rgb(29, 35, 43);
    pub const PANEL_ELEVATED: Color32 = Color32::from_rgb(37, 45, 54);
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(42, 50, 59);
    pub const BORDER_STRONG: Color32 = Color32::from_rgb(65, 77, 89);
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(237, 242, 247);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(164, 178, 191);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(139, 155, 170);
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(89, 223, 211);
    pub const ACCENT_CYAN_HOVER: Color32 = Color32::from_rgb(137, 239, 229);
    pub const ACCENT_CYAN_FAINT: Color32 = Color32::from_rgb(25, 55, 54);
    pub const ACCENT_CYAN_GLOW: Color32 = Color32::from_rgba_premultiplied(14, 56, 53, 64);
    pub const ALERT_AMBER: Color32 = Color32::from_rgb(239, 184, 104);
    pub const ALERT_AMBER_FAINT: Color32 = Color32::from_rgb(54, 44, 31);
    pub const SUCCESS_GREEN: Color32 = Color32::from_rgb(126, 216, 165);
    pub const SUCCESS_GREEN_FAINT: Color32 = Color32::from_rgb(28, 49, 40);
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Optional system fonts; egui's bundled fonts keep the executable portable.
    for (name, path, family) in [
        (
            "system-ui",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            FontFamily::Proportional,
        ),
        (
            "system-mono",
            "C:\\Windows\\Fonts\\consola.ttf",
            FontFamily::Monospace,
        ),
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.into(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, name.into());
        }
    }
    ctx.set_fonts(fonts);
    ctx.set_theme(egui::Theme::Dark);
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(colors::TEXT_PRIMARY);
    visuals.panel_fill = colors::CHASSIS_BG;
    visuals.window_fill = colors::PANEL_BG;
    visuals.faint_bg_color = colors::PANEL_BG_LIGHT;
    visuals.extreme_bg_color = colors::SIDEBAR_BG;
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(6);
        widget.fg_stroke = Stroke::new(1.5, colors::TEXT_PRIMARY);
    }
    visuals.widgets.inactive.bg_fill = colors::PANEL_BG_LIGHT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.hovered.bg_fill = colors::PANEL_ELEVATED;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::ACCENT_CYAN);
    visuals.widgets.active.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, colors::ACCENT_CYAN);
    visuals.widgets.open = visuals.widgets.active;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_MUTED);
    visuals.selection.bg_fill = colors::ACCENT_CYAN_FAINT;
    visuals.selection.stroke = Stroke::new(1.5, colors::ACCENT_CYAN);
    ctx.set_visuals(visuals);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(26.0));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(13.0));
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.interact_size.y = 32.0;
        style.spacing.window_margin = egui::Margin::same(16);
        style.spacing.scroll.bar_width = 6.0;
        style.visuals.slider_trailing_fill = true;
        style.visuals.handle_shape = egui::style::HandleShape::Circle;
    });
}
