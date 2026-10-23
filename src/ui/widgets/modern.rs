//! Modern custom UI widgets for Vectorists.
//!
//! Provides sleek, modern desktop controls:
//! - ToggleSwitch (iOS/Fluent style animated pill toggle)
//! - MetricCard (Hero statistics card with icon and badge)
//! - ModernSlider (Ultra-thin track with glowing thumb and value pill)
//! - SegmentedTabs (Floating pill navigation)

use crate::ui::theme::colors;
use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Vec2};

/// A modern iOS / Fluent style toggle switch.
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    let desired_size = Vec2::new(36.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let radius = rect.height() / 2.0;

        // Track background
        let track_color = if *on {
            colors::ACCENT_CYAN
        } else {
            colors::PANEL_BG_LIGHT
        };
        painter.rect_filled(rect, CornerRadius::same(radius as u8), track_color);
        painter.rect_stroke(
            rect,
            CornerRadius::same(radius as u8),
            Stroke::new(1.0, if *on { colors::ACCENT_CYAN } else { colors::BORDER_STRONG }),
            egui::StrokeKind::Inside,
        );

        // Thumb position
        let thumb_radius = radius - 2.5;
        let thumb_x = if *on {
            rect.right() - radius
        } else {
            rect.left() + radius
        };
        let thumb_pos = Pos2::new(thumb_x, rect.center().y);

        // Thumb glow & circle
        if *on {
            painter.circle_stroke(
                thumb_pos,
                thumb_radius + 2.0,
                Stroke::new(1.5, colors::ACCENT_CYAN_FAINT),
            );
        }
        painter.circle_filled(
            thumb_pos,
            thumb_radius,
            if *on { colors::CHASSIS_BG } else { colors::TEXT_PRIMARY },
        );
    }

    if !label.is_empty() {
        ui.label(
            egui::RichText::new(label)
                .size(11.0)
                .color(if *on { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED }),
        );
    }

    response
}

/// Renders a modern hero telemetry/metric card.
pub fn metric_card(
    ui: &mut egui::Ui,
    width: f32,
    title: &str,
    value: &str,
    subtext: &str,
    badge: Option<(&str, Color32)>,
) {
    let height = 64.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);

        // Card surface with subtle top edge highlight (Apple/Linear style lighting)
        painter.rect_filled(rect, CornerRadius::same(6), colors::PANEL_BG);
        painter.rect_stroke(
            rect,
            CornerRadius::same(6),
            Stroke::new(1.0, colors::BORDER_SUBTLE),
            egui::StrokeKind::Inside,
        );

        // Top 1px illumination line
        let top_highlight = [
            Pos2::new(rect.left() + 6.0, rect.top() + 1.0),
            Pos2::new(rect.right() - 6.0, rect.top() + 1.0),
        ];
        painter.line_segment(
            top_highlight,
            Stroke::new(1.0, Color32::from_rgba_premultiplied(0xFF, 0xFF, 0xFF, 0x10)),
        );

        // Title / Header row
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 10.0),
            egui::Align2::LEFT_TOP,
            title.to_uppercase(),
            egui::FontId::monospace(9.0),
            colors::TEXT_MUTED,
        );

        // Badge if present
        if let Some((b_text, b_color)) = badge {
            let badge_pos = Pos2::new(rect.right() - 12.0, rect.top() + 10.0);
            painter.text(
                badge_pos,
                egui::Align2::RIGHT_TOP,
                b_text,
                egui::FontId::monospace(9.0),
                b_color,
            );
        }

        // Hero Value & Subtext
        painter.text(
            Pos2::new(rect.left() + 12.0, rect.top() + 26.0),
            egui::Align2::LEFT_TOP,
            value,
            egui::FontId::monospace(17.0),
            colors::TEXT_PRIMARY,
        );

        painter.text(
            Pos2::new(rect.left() + 12.0, rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            subtext,
            egui::FontId::proportional(10.0),
            colors::TEXT_FAINT,
        );
    }
}

/// Modern sleek slider with filled active segment and value indicator.
pub fn modern_slider(
    ui: &mut egui::Ui,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    display_text: &str,
) -> bool {
    let width = ui.available_width().max(120.0);
    let height = 24.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
    let mut changed = false;

    let min = *range.start();
    let max = *range.end();

    if response.dragged() || response.clicked() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            let progress = ((mouse_pos.x - (rect.left() + 6.0)) / (rect.width() - 12.0)).clamp(0.0, 1.0);
            *val = min + progress * (max - min);
            changed = true;
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let track_y = rect.center().y;
        let track_height = 4.0;
        let track_rect = Rect::from_min_max(
            Pos2::new(rect.left() + 6.0, track_y - track_height / 2.0),
            Pos2::new(rect.right() - 6.0, track_y + track_height / 2.0),
        );

        // Background track
        painter.rect_filled(track_rect, CornerRadius::same(2), colors::PANEL_BG_LIGHT);

        // Active track segment (Cyan)
        let progress = ((*val - min) / (max - min)).clamp(0.0, 1.0);
        let active_width = (track_rect.width() * progress).max(0.0);
        let active_rect = Rect::from_min_max(
            track_rect.min,
            Pos2::new(track_rect.left() + active_width, track_rect.max.y),
        );
        painter.rect_filled(active_rect, CornerRadius::same(2), colors::ACCENT_CYAN);

        // Thumb knob
        let thumb_x = track_rect.left() + active_width;
        let thumb_pos = Pos2::new(thumb_x, track_y);

        // Glow ring
        painter.circle_filled(
            thumb_pos,
            7.0,
            Color32::from_rgba_premultiplied(0x00, 0xE5, 0xFF, 0x30),
        );
        painter.circle_filled(thumb_pos, 5.0, colors::TEXT_PRIMARY);
        painter.circle_stroke(thumb_pos, 5.0, Stroke::new(1.0, colors::ACCENT_CYAN));
    }

    // Hover tooltip with value
    response.on_hover_ui_at_pointer(|ui| {
        ui.label(
            egui::RichText::new(display_text)
                .monospace()
                .size(11.0)
                .color(colors::ACCENT_CYAN),
        );
    });

    changed
}
