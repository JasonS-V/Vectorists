//! Shared controls use egui's focus, keyboard and accessibility semantics.
//! Formatted according to the Premium Minimal Dark Dashboard design guide.
use crate::ui::theme::colors;
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};

pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(42.0, 24.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::accesskit::Role::Switch, ui.is_enabled(), *on, label)
    });
    let t = ui.ctx().animate_bool(response.id, *on);
    ui.painter().rect_filled(
        rect.shrink2(egui::vec2(0.0, 1.0)),
        12,
        if *on {
            colors::ACCENT
        } else {
            colors::PANEL_ELEVATED
        },
    );
    if !*on {
        ui.painter().rect_stroke(
            rect.shrink2(egui::vec2(0.0, 1.0)),
            12,
            Stroke::new(1.0, colors::BORDER_STRONG),
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().circle_filled(
        Pos2::new(rect.left() + 12.0 + t * 18.0, rect.center().y),
        8.0,
        if *on {
            colors::CHASSIS_BG
        } else {
            colors::TEXT_PRIMARY
        },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            14,
            Stroke::new(2.0, colors::ACCENT),
            egui::StrokeKind::Outside,
        );
    }
    response.on_hover_text(label)
}

pub fn metric_card(ui: &mut egui::Ui, width: f32, title: &str, value: &str, subtext: &str) {
    card_container(ui, |ui| {
        ui.set_width((width - 34.0).max(0.0));
        ui.spacing_mut().item_spacing.y = 4.0;
        ui.set_min_height(if width < 220.0 { 96.0 } else { 76.0 });
        ui.label(
            egui::RichText::new(title)
                .size(12.0)
                .color(colors::TEXT_SECONDARY),
        );
        ui.label(
            egui::RichText::new(value)
                .size(26.0)
                .strong()
                .color(colors::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(subtext)
                .size(11.5)
                .color(colors::TEXT_MUTED),
        );
    });
}

pub fn card_container<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(colors::PANEL_BG)
        .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
        .corner_radius(14)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.spacing_mut().interact_size.y = 24.0;
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), contents)
                .inner
        })
}

pub fn pill_chip(ui: &mut egui::Ui, text: &str, active: bool, accent: Color32) -> bool {
    ui.add(
        egui::Button::new(egui::RichText::new(text).size(12.0).color(if active {
            accent
        } else {
            colors::TEXT_SECONDARY
        }))
        .fill(if active {
            colors::ACCENT_FAINT
        } else {
            colors::PANEL_BG_LIGHT
        })
        .stroke(Stroke::new(
            1.0,
            if active {
                accent
            } else {
                colors::BORDER_SUBTLE
            },
        ))
        .corner_radius(16)
        .min_size(egui::vec2(0.0, 32.0)),
    )
    .clicked()
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .strong()
                .color(Color32::from_rgb(23, 21, 30)),
        )
        .fill(colors::ACCENT)
        .corner_radius(10)
        .stroke(Stroke::NONE)
        .min_size(egui::vec2(0.0, 38.0)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(colors::TEXT_PRIMARY))
            .fill(colors::PANEL_ELEVATED)
            .corner_radius(10)
            .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
            .min_size(egui::vec2(0.0, 38.0)),
    )
}

pub fn section_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(egui::RichText::new(title).size(16.5).strong().color(colors::TEXT_PRIMARY));
    if !subtitle.is_empty() {
        ui.label(
            egui::RichText::new(subtitle)
                .size(12.5)
                .color(colors::TEXT_MUTED),
        );
    }
}

pub fn direction_micro_bar(
    ui: &mut egui::Ui,
    angle: f32,
    value: &mut f32,
    selected: bool,
) -> (bool, bool) {
    let mut clicked = false;
    let mut changed = false;
    ui.push_id(angle.to_bits(), |ui| {
        ui.horizontal(|ui| {
            clicked = ui
                .add_sized(
                    [56.0, 28.0],
                    egui::Button::selectable(
                        selected,
                        egui::RichText::new(format!("{angle:.1}°"))
                            .monospace()
                            .size(11.0),
                    ),
                )
                .clicked();
            ui.spacing_mut().slider_width = (ui.available_width() - 68.0).max(40.0);
            changed = ui
                .add(
                    egui::Slider::new(value, 0.5..=1.5)
                        .show_value(false)
                        .step_by(0.005),
                )
                .on_hover_text(format!(
                    "Multiplicador a {angle:.1}° · flechas para ajustar"
                ))
                .changed();
            ui.label(
                egui::RichText::new(format!("{value:.3}×"))
                    .monospace()
                    .size(11.0)
                    .color(if selected {
                        colors::ACCENT
                    } else {
                        colors::TEXT_MUTED
                    }),
            );
        });
    });
    (clicked, changed)
}

#[derive(Clone, Copy)]
pub enum NavIcon {
    Polar,
    Target,
    Settings,
}

pub fn nav_item(
    ui: &mut egui::Ui,
    label: &str,
    icon: NavIcon,
    active: bool,
    compact: bool,
) -> bool {
    let height = 44.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::accesskit::Role::Tab, ui.is_enabled(), active, label)
    });
    let painter = ui.painter();
    let is_hovered = response.hovered();

    // Background fill based on state
    let bg_color = if active {
        colors::PANEL_BG
    } else if is_hovered {
        colors::PANEL_BG_LIGHT
    } else {
        Color32::TRANSPARENT
    };

    if bg_color != Color32::TRANSPARENT {
        painter.rect_filled(rect, 10, bg_color);
    }
    if active {
        painter.rect_stroke(rect, 10, Stroke::new(1.0, colors::BORDER_SUBTLE), egui::StrokeKind::Inside);
    }

    let color = if active {
        colors::ACCENT
    } else if is_hovered {
        colors::TEXT_PRIMARY
    } else {
        colors::TEXT_SECONDARY
    };

    let c = Pos2::new(
        rect.left()
            + if compact {
                rect.width() * 0.5
            } else {
                24.0
            },
        rect.center().y,
    );
    let stroke = Stroke::new(1.6, color);
    match icon {
        NavIcon::Polar | NavIcon::Target => {
            painter.circle_stroke(c, 8.0, stroke);
            painter.circle_stroke(c, 3.0, stroke);
            if matches!(icon, NavIcon::Polar) {
                painter.line_segment([c, c + egui::vec2(8.5, -8.5)], stroke);
            } else {
                painter.line_segment(
                    [c - egui::vec2(10.0, 0.0), c + egui::vec2(10.0, 0.0)],
                    stroke,
                );
                painter.line_segment(
                    [c - egui::vec2(0.0, 10.0), c + egui::vec2(0.0, 10.0)],
                    stroke,
                );
            }
        }
        NavIcon::Settings => {
            for (x, offset) in [(-7.0, -4.0), (0.0, 4.0), (7.0, -1.0)] {
                painter.line_segment([c + egui::vec2(x, -9.0), c + egui::vec2(x, 9.0)], stroke);
                painter.circle_filled(c + egui::vec2(x, offset), 2.5, color);
            }
        }
    }
    if !compact {
        painter.text(
            Pos2::new(rect.left() + 48.0, c.y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.0),
            if active { colors::TEXT_PRIMARY } else { color },
        );
    }
    if active {
        // Left active pill indicator
        painter.rect_filled(
            egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(0.0, 10.0),
                egui::vec2(3.0, rect.height() - 20.0),
            ),
            2,
            colors::ACCENT,
        );
    }
    response.on_hover_text(label).clicked()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_toggle_can_be_activated_with_keyboard() {
        let ctx = egui::Context::default();
        let mut enabled = false;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            toggle_switch(ui, &mut enabled, "Vista previa").request_focus();
        });
        output.textures_delta.clear();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Space,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ui| {
                toggle_switch(ui, &mut enabled, "Vista previa");
            },
        );
        output.textures_delta.clear();
        assert!(enabled);
    }
}
