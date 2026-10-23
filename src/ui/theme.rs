//! Shared visual tokens for the native precision workspace.
//! Graphite surfaces, restrained blue accents and Windows system typography.
use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

pub mod colors {
    use super::Color32;

    // Surface hierarchy: navigation, workspace, cards and controls.
    pub const CHASSIS_BG: Color32 = Color32::from_rgb(34, 36, 46);
    pub const SIDEBAR_BG: Color32 = Color32::from_rgb(25, 27, 35);
    pub const PANEL_BG: Color32 = Color32::from_rgb(42, 45, 56);
    pub const PANEL_BG_LIGHT: Color32 = Color32::from_rgb(49, 53, 65);
    pub const PANEL_ELEVATED: Color32 = Color32::from_rgb(57, 62, 76);
    pub const PANEL_HOVER: Color32 = Color32::from_rgb(64, 70, 85);

    // Precision Borders
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 20); // ~8% white
    pub const BORDER_STRONG: Color32 = Color32::from_rgba_unmultiplied_const(255, 255, 255, 38); // ~15% white

    // Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(245, 244, 248); // #F5F4F8: Headers, KPIs, main text
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(181, 178, 192); // #B5B2C0: Nav items, labels, descriptions
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(161, 162, 178);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(157, 158, 173);

    // Accents retain contrast on both card and control surfaces.
    pub const ACCENT_VIOLET: Color32 = Color32::from_rgb(174, 147, 240);
    pub const ACCENT_GOLD: Color32 = Color32::from_rgb(229, 185, 133);
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(116, 169, 246);

    // Active Product Accent (Default: Graphite Blue for high-precision workspace)
    pub const ACCENT: Color32 = ACCENT_BLUE;
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(128, 173, 245);
    pub const ACCENT_FAINT: Color32 = Color32::from_rgba_unmultiplied_const(100, 153, 236, 38);
    pub const ACCENT_GLOW: Color32 = Color32::from_rgba_unmultiplied_const(100, 153, 236, 60);

    // Functional Semantic Feedback
    pub const SUCCESS: Color32 = Color32::from_rgb(101, 197, 160); // #65C5A0: Success green
    pub const SUCCESS_FAINT: Color32 = Color32::from_rgba_unmultiplied_const(101, 197, 160, 35);
    pub const WARNING: Color32 = Color32::from_rgb(231, 182, 110); // #E7B66E: Warning amber
    pub const WARNING_FAINT: Color32 = Color32::from_rgba_unmultiplied_const(231, 182, 110, 35);
    pub const DANGER: Color32 = Color32::from_rgb(237, 133, 141); // #ED858D: Danger red
    pub const DANGER_FAINT: Color32 = Color32::from_rgba_unmultiplied_const(237, 133, 141, 35);
    pub const INFO: Color32 = Color32::from_rgb(118, 182, 243); // #76B6F3: Info blue

    // Aliases for compatibility
    pub const ACCENT_CYAN: Color32 = ACCENT;
    pub const ACCENT_CYAN_HOVER: Color32 = ACCENT_HOVER;
    pub const ACCENT_CYAN_FAINT: Color32 = ACCENT_FAINT;
    pub const ACCENT_CYAN_GLOW: Color32 = ACCENT_GLOW;
    pub const ALERT_AMBER: Color32 = WARNING;
    pub const ALERT_AMBER_FAINT: Color32 = WARNING_FAINT;
    pub const SUCCESS_GREEN: Color32 = SUCCESS;
    pub const SUCCESS_GREEN_FAINT: Color32 = SUCCESS_FAINT;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ThemeAccent {
    #[default]
    GraphiteBlue,
    SoftViolet,
    MidnightGold,
}

impl ThemeAccent {
    pub fn name(self) -> &'static str {
        match self {
            Self::GraphiteBlue => "Azul",
            Self::SoftViolet => "Violeta",
            Self::MidnightGold => "Champán",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::GraphiteBlue => colors::ACCENT_BLUE,
            Self::SoftViolet => colors::ACCENT_VIOLET,
            Self::MidnightGold => colors::ACCENT_GOLD,
        }
    }
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
    apply_theme_with_accent(ctx, colors::ACCENT);
}

pub fn accent(ui: &egui::Ui) -> Color32 {
    ui.visuals().selection.stroke.color
}

pub fn accent_soft(ui: &egui::Ui) -> Color32 {
    ui.visuals().selection.bg_fill
}

pub fn apply_theme_with_accent(ctx: &egui::Context, accent: Color32) {
    let soft = Color32::from_rgba_unmultiplied_const(accent.r(), accent.g(), accent.b(), 28);
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
        widget.corner_radius = CornerRadius::same(10);
        widget.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);
    }
    visuals.widgets.inactive.bg_fill = colors::PANEL_BG_LIGHT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.hovered.bg_fill = colors::PANEL_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors::BORDER_STRONG);
    visuals.widgets.active.bg_fill = soft;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, accent);
    visuals.widgets.open = visuals.widgets.active;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors::BORDER_SUBTLE);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_MUTED);
    visuals.selection.bg_fill = soft;
    visuals.selection.stroke = Stroke::new(1.5, accent);
    ctx.set_visuals(visuals);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(28.0));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(13.5));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(12.5));
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.spacing.button_padding = egui::vec2(16.0, 9.0);
        style.spacing.interact_size.y = 36.0;
        style.spacing.window_margin = egui::Margin::same(20);
        style.spacing.scroll.bar_width = 6.0;
        style.visuals.slider_trailing_fill = true;
        style.visuals.handle_shape = egui::style::HandleShape::Circle;
    });
}
