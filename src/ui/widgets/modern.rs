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

/// An elevated obsidian card container with smooth borders and inner padding.
pub fn card_container<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(colors::PANEL_BG)
        .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, add_contents)
}

/// Modern pill chip button for presets or actions.
pub fn pill_chip(ui: &mut egui::Ui, text: &str, is_active: bool, accent_color: Color32) -> bool {
    let padding = Vec2::new(12.0, 5.0);
    let font_id = egui::FontId::proportional(10.5);
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        font_id.clone(),
        if is_active { colors::TEXT_PRIMARY } else { colors::TEXT_MUTED },
    );
    let size = Vec2::new(galley.size().x + padding.x * 2.0, 24.0);

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let radius = CornerRadius::same((rect.height() / 2.0) as u8);

        let bg = if is_active {
            Color32::from_rgba_premultiplied(accent_color.r(), accent_color.g(), accent_color.b(), 0x30)
        } else if response.hovered() {
            colors::PANEL_ELEVATED
        } else {
            colors::PANEL_BG_LIGHT
        };

        let stroke_color = if is_active {
            accent_color
        } else if response.hovered() {
            colors::BORDER_STRONG
        } else {
            colors::BORDER_SUBTLE
        };

        painter.rect_filled(rect, radius, bg);
        painter.rect_stroke(rect, radius, Stroke::new(1.0, stroke_color), egui::StrokeKind::Inside);

        let text_color = if is_active {
            accent_color
        } else if response.hovered() {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_MUTED
        };

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_id,
            text_color,
        );
    }

    response.clicked()
}

/// Modern segmented dock tab item.
pub fn segmented_pill_tab(ui: &mut egui::Ui, label: &str, is_active: bool) -> bool {
    let font_id = egui::FontId::proportional(11.0);
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        font_id.clone(),
        colors::TEXT_PRIMARY,
    );
    let size = Vec2::new(galley.size().x + 22.0, 26.0);

    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let radius = CornerRadius::same(6);

        if is_active {
            painter.rect_filled(rect, radius, colors::PANEL_ELEVATED);
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, colors::ACCENT_CYAN_GLOW),
                egui::StrokeKind::Inside,
            );
            // Crisp bottom accent line
            painter.line_segment(
                [
                    Pos2::new(rect.left() + 8.0, rect.bottom() - 1.5),
                    Pos2::new(rect.right() - 8.0, rect.bottom() - 1.5),
                ],
                Stroke::new(1.8, colors::ACCENT_CYAN),
            );
        } else if response.hovered() {
            painter.rect_filled(rect, radius, colors::PANEL_BG_LIGHT);
        }

        let text_color = if is_active {
            colors::TEXT_PRIMARY
        } else if response.hovered() {
            colors::TEXT_PRIMARY
        } else {
            colors::TEXT_MUTED
        };

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font_id,
            text_color,
        );
    }

    response.clicked()
}

/// Compact micro-card control for directional calibration (0.6x to 1.4x).
pub fn direction_micro_bar(
    ui: &mut egui::Ui,
    angle_deg: f32,
    mult: &mut f32,
    is_selected: bool,
) -> (bool, bool) {
    let size = Vec2::new(ui.available_width().max(140.0), 22.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let mut selected_clicked = false;
    let mut changed = false;

    if response.clicked() {
        selected_clicked = true;
    }

    if response.dragged() {
        let delta = response.drag_delta();
        let step = delta.x * 0.005;
        *mult = (*mult + step).clamp(0.60, 1.40);
        changed = true;
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        let radius = CornerRadius::same(4);

        let bg = if is_selected {
            colors::PANEL_ELEVATED
        } else if response.hovered() {
            colors::PANEL_BG_LIGHT
        } else {
            colors::CHASSIS_BG
        };

        let stroke_color = if is_selected {
            colors::ACCENT_CYAN
        } else if response.hovered() {
            colors::BORDER_STRONG
        } else {
            colors::BORDER_SUBTLE
        };

        painter.rect_filled(rect, radius, bg);
        painter.rect_stroke(rect, radius, Stroke::new(1.0, stroke_color), egui::StrokeKind::Inside);

        // Angle label on the left
        let angle_text = format!("{:>5.1}°", angle_deg);
        painter.text(
            Pos2::new(rect.left() + 6.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            angle_text,
            egui::FontId::monospace(9.5),
            if is_selected { colors::ACCENT_CYAN } else { colors::TEXT_MUTED },
        );

        // Center visual delta deviation bar
        let bar_left = rect.left() + 52.0;
        let bar_right = rect.right() - 62.0;
        let bar_center = (bar_left + bar_right) / 2.0;
        let bar_y = rect.center().y;

        // Neutral 1.0 centerline
        painter.line_segment(
            [Pos2::new(bar_center, bar_y - 4.0), Pos2::new(bar_center, bar_y + 4.0)],
            Stroke::new(1.0, colors::BORDER_STRONG),
        );

        // Deviation bar
        let deviation = *mult - 1.0; // range -0.40 .. +0.40
        let norm_dev = (deviation / 0.40).clamp(-1.0, 1.0);
        let bar_half_width = (bar_right - bar_left) / 2.0;
        let fill_x = bar_center + norm_dev * bar_half_width;

        if deviation.abs() > 0.005 {
            let (fill_rect, fill_col) = if deviation > 0.0 {
                (
                    Rect::from_min_max(Pos2::new(bar_center, bar_y - 2.0), Pos2::new(fill_x, bar_y + 2.0)),
                    colors::ACCENT_CYAN,
                )
            } else {
                (
                    Rect::from_min_max(Pos2::new(fill_x, bar_y - 2.0), Pos2::new(bar_center, bar_y + 2.0)),
                    colors::ALERT_AMBER,
                )
            };
            painter.rect_filled(fill_rect, CornerRadius::same(1), fill_col);
        }

        // Value & Delta text on the right
        let delta_pct = (*mult - 1.0) * 100.0;
        let val_text = format!("{:>4.2}x ({:+2.0}%)", *mult, delta_pct);
        let val_color = if delta_pct.abs() < 1.0 {
            colors::TEXT_MUTED
        } else if delta_pct > 0.0 {
            colors::ACCENT_CYAN
        } else {
            colors::ALERT_AMBER
        };

        painter.text(
            Pos2::new(rect.right() - 6.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            val_text,
            egui::FontId::monospace(9.0),
            val_color,
        );
    }

    (selected_clicked, changed)
}
