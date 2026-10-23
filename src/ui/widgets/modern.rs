//! Shared native controls with keyboard focus and accessible names.
use crate::ui::theme::{accent, accent_soft, colors};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};

pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool, label: &str) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(38.0, 22.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::accesskit::Role::Switch, ui.is_enabled(), *on, label)
    });
    let t = ui.ctx().animate_bool(response.id, *on);
    ui.painter().rect_filled(
        rect,
        11,
        if *on {
            accent(ui)
        } else {
            colors::PANEL_ELEVATED
        },
    );
    ui.painter().circle_filled(
        Pos2::new(rect.left() + 11.0 + t * 16.0, rect.center().y),
        7.0,
        colors::TEXT_PRIMARY,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            14,
            Stroke::new(2.0, accent(ui)),
            egui::StrokeKind::Outside,
        );
    }
    response.on_hover_text(label)
}

pub fn card_container<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Frame::new()
        .fill(colors::PANEL_BG)
        .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
        .corner_radius(12)
        .inner_margin(16)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.spacing_mut().interact_size.y = 28.0;
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), contents)
                .inner
        })
}

pub fn metric_card(
    ui: &mut egui::Ui,
    width: f32,
    title: &str,
    value: &str,
    subtext: &str,
    highlighted: bool,
) {
    let foreground = if highlighted {
        colors::SIDEBAR_BG
    } else {
        colors::TEXT_PRIMARY
    };
    let secondary = if highlighted {
        foreground
    } else {
        colors::TEXT_SECONDARY
    };
    egui::Frame::new()
        .fill(if highlighted {
            accent(ui)
        } else {
            colors::PANEL_BG
        })
        .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
        .corner_radius(12)
        .inner_margin(egui::Margin::symmetric(18, 14))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width((width - 38.0).max(0.0));
                ui.set_min_height(76.0);
                ui.spacing_mut().item_spacing.y = 3.0;
                ui.label(egui::RichText::new(title).size(12.0).color(secondary));
                ui.label(egui::RichText::new(value).size(30.0).color(foreground));
                ui.label(egui::RichText::new(subtext).size(11.0).color(secondary));
            });
        });
}

pub fn pill_chip(ui: &mut egui::Ui, text: &str, active: bool, tint: Color32) -> bool {
    ui.add(
        egui::Button::selectable(
            active,
            egui::RichText::new(text).size(12.0).color(if active {
                tint
            } else {
                colors::TEXT_SECONDARY
            }),
        )
        .fill(if active {
            Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), 24)
        } else {
            colors::PANEL_BG_LIGHT
        })
        .stroke(Stroke::new(
            1.0,
            if active { tint } else { colors::BORDER_SUBTLE },
        ))
        .corner_radius(7)
        .min_size(egui::vec2(0.0, 32.0)),
    )
    .clicked()
}

pub fn quick_preset_capsule(
    ui: &mut egui::Ui,
    width: f32,
    title: &str,
    subtext: &str,
    glyph: &str,
    active: bool,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width.max(0.0), 58.0), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::accesskit::Role::RadioButton,
            ui.is_enabled(),
            active,
            title,
        )
    });
    let p = ui.painter_at(rect);
    p.rect_filled(
        rect,
        8,
        if active {
            accent_soft(ui)
        } else if response.hovered() {
            colors::PANEL_ELEVATED
        } else {
            colors::PANEL_BG_LIGHT
        },
    );
    p.rect_stroke(
        rect,
        8,
        Stroke::new(
            if response.has_focus() { 2.0 } else { 1.0 },
            if active || response.has_focus() {
                accent(ui)
            } else {
                colors::BORDER_SUBTLE
            },
        ),
        egui::StrokeKind::Inside,
    );
    let icon = Pos2::new(rect.left() + 20.0, rect.center().y);
    p.text(
        icon,
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(20.0),
        if active {
            accent(ui)
        } else {
            colors::TEXT_MUTED
        },
    );
    p.text(
        rect.left_top() + egui::vec2(40.0, 19.0),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(12.0),
        colors::TEXT_PRIMARY,
    );
    p.text(
        rect.left_top() + egui::vec2(40.0, 39.0),
        egui::Align2::LEFT_CENTER,
        subtext,
        egui::FontId::proportional(10.5),
        colors::TEXT_SECONDARY,
    );
    response
        .on_hover_text(format!("Aplicar plantilla {title}: {subtext}"))
        .clicked()
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).strong().color(colors::SIDEBAR_BG))
            .fill(accent(ui))
            .corner_radius(8)
            .stroke(Stroke::NONE)
            .min_size(egui::vec2(0.0, 36.0)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(colors::TEXT_PRIMARY))
            .fill(colors::PANEL_BG_LIGHT)
            .corner_radius(8)
            .stroke(Stroke::new(1.0, colors::BORDER_STRONG))
            .min_size(egui::vec2(0.0, 36.0)),
    )
}

pub fn section_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(16.0)
            .strong()
            .color(colors::TEXT_PRIMARY),
    );
    if !subtitle.is_empty() {
        ui.label(
            egui::RichText::new(subtitle)
                .size(12.0)
                .color(colors::TEXT_SECONDARY),
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
            ui.spacing_mut().slider_width = (ui.available_width() - 70.0).max(40.0);
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
                        accent(ui)
                    } else {
                        colors::TEXT_SECONDARY
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
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 46.0), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::accesskit::Role::Tab, ui.is_enabled(), active, label)
    });
    let p = ui.painter();
    if active || response.hovered() {
        p.rect_filled(
            rect,
            8,
            if active {
                accent_soft(ui)
            } else {
                colors::PANEL_BG
            },
        );
    }
    if response.has_focus() {
        p.rect_stroke(
            rect,
            8,
            Stroke::new(1.5, accent(ui)),
            egui::StrokeKind::Inside,
        );
    }
    let color = if active {
        accent(ui)
    } else {
        colors::TEXT_SECONDARY
    };
    let c = Pos2::new(
        if compact {
            rect.center().x
        } else {
            rect.left() + 22.0
        },
        rect.center().y,
    );
    let stroke = Stroke::new(1.5, color);
    match icon {
        NavIcon::Polar => {
            for offset in [
                egui::vec2(-5.0, -5.0),
                egui::vec2(5.0, -5.0),
                egui::vec2(-5.0, 5.0),
                egui::vec2(5.0, 5.0),
            ] {
                p.rect_stroke(
                    egui::Rect::from_center_size(c + offset, Vec2::splat(6.0)),
                    1,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
        NavIcon::Target => {
            p.circle_stroke(c, 8.0, stroke);
            p.circle_stroke(c, 3.0, stroke);
            for dir in [
                egui::vec2(1.0, 0.0),
                egui::vec2(-1.0, 0.0),
                egui::vec2(0.0, 1.0),
                egui::vec2(0.0, -1.0),
            ] {
                p.line_segment([c + dir * 6.0, c + dir * 11.0], stroke);
            }
        }
        NavIcon::Settings => {
            for (x, offset) in [(-7.0, -4.0), (0.0, 4.0), (7.0, -1.0)] {
                p.line_segment([c + egui::vec2(x, -9.0), c + egui::vec2(x, 9.0)], stroke);
                p.circle_filled(c + egui::vec2(x, offset), 2.5, color);
            }
        }
    }
    if !compact {
        p.text(
            Pos2::new(rect.left() + 46.0, c.y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.5),
            if active { colors::TEXT_PRIMARY } else { color },
        );
        if active {
            p.circle_filled(Pos2::new(rect.right() - 14.0, c.y), 2.5, accent(ui));
        }
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
