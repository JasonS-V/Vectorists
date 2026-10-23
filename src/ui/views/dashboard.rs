//! Response workspace: live sensor measurements, curve presets and local preview.
use crate::core::accumulator::SubpixelAccumulator;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::{accent, colors};
use crate::ui::widgets::radar::direction_index;
use crate::ui::widgets::{
    PolarRadarWidget, card_container, direction_micro_bar, metric_card, quick_preset_capsule,
    section_heading,
};
use eframe::egui::{self, Sense, Stroke, Vec2};

fn preview_delta(profile: &Profile, delta: Vec2) -> (f32, f32) {
    if profile.is_active {
        profile
            .fourier
            .correct_delta(delta.x, delta.y, profile.strength)
    } else {
        (delta.x, delta.y)
    }
}

#[derive(Default)]
pub struct DashboardView {
    pub selected_dir_index: Option<usize>,
    pub sandbox_accumulator: SubpixelAccumulator,
    // Canvas-relative positions keep trails aligned after a resize or scroll.
    pub sandbox_trail: Vec<(Vec2, Vec2)>,
    pub test_cursor: Option<Vec2>,
    pub last_delta: (i32, i32),
}

impl DashboardView {
    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &mut Profile, hz: u32, cps: f32) {
        let width = ui.available_width();
        let max_adjustment = profile
            .direction_multipliers
            .iter()
            .map(|m| (m - 1.0).abs())
            .fold(0.0_f32, f32::max);
        let sensor_value = if hz > 0 {
            format!("{hz} Hz")
        } else {
            "En espera".into()
        };
        let sensor_detail = if hz > 0 {
            format!("{cps:.0} cuentas/s · entrada directa del sensor")
        } else {
            format!(
                "{} DPI · {} Hz configurados",
                profile.dpi, profile.polling_rate_hz
            )
        };
        let intensity = format!("{:.0}%", profile.strength * 100.0);
        let adjustment = format!("{:.1}%", max_adjustment * 100.0);
        let metrics = [
            (
                "Actividad del sensor",
                sensor_value.as_str(),
                sensor_detail.as_str(),
                true,
            ),
            (
                "Ajuste direccional máximo",
                adjustment.as_str(),
                "Desviación configurada respecto a 1:1",
                false,
            ),
            (
                "Intensidad de la curva",
                intensity.as_str(),
                if profile.is_active {
                    "Vista previa activa · 16 direcciones"
                } else {
                    "Vista previa en pausa · respuesta 1:1"
                },
                false,
            ),
        ];
        if width >= 660.0 {
            let card_width = ((width - 24.0) / 3.0).floor();
            ui.horizontal_top(|ui| {
                for (title, value, detail, highlighted) in metrics {
                    metric_card(ui, card_width, title, value, detail, highlighted);
                }
            });
        } else {
            for (title, value, detail, highlighted) in metrics {
                metric_card(ui, width, title, value, detail, highlighted);
            }
        }
        ui.add_space(4.0);
        if width >= 850.0 {
            let left_width = (width * 0.56).floor();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(left_width, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| self.map_card(ui, profile),
                );
                ui.vertical(|ui| {
                    ui.set_width(width - left_width - 12.0);
                    self.response_controls(ui, profile);
                });
            });
        } else {
            self.map_card(ui, profile);
            self.response_controls(ui, profile);
        }
        ui.add_space(4.0);
        self.sandbox(ui, profile);
    }

    fn map_card(&mut self, ui: &mut egui::Ui, profile: &Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width((width - 34.0).max(0.0));
            ui.horizontal(|ui| {
                section_heading(ui, "Tu respuesta en 360°", "");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("MAPA POLAR")
                            .size(10.0)
                            .color(colors::TEXT_MUTED),
                    );
                });
            });
            ui.label(
                egui::RichText::new("Cada dirección, el mismo control.")
                    .size(12.0)
                    .color(colors::TEXT_SECONDARY),
            );
            let height = (ui.ctx().viewport_rect().height() - 460.0).clamp(260.0, 320.0);
            let side = (ui.available_width() - 8.0).min(height);
            ui.vertical_centered(|ui| {
                let response = ui.add(
                    PolarRadarWidget::new(profile)
                        .size(Vec2::splat(side))
                        .selected_index(self.selected_dir_index),
                );
                if response.clicked()
                    && let Some(position) = response.interact_pointer_pos()
                {
                    self.selected_dir_index =
                        Some(direction_index(position - response.rect.center()));
                }
            });
            ui.horizontal_wrapped(|ui| {
                for (color, text) in [
                    (accent(ui), "— Respuesta efectiva"),
                    (colors::WARNING, "● Ajustes"),
                    (colors::TEXT_MUTED, "○ Neutro 1:1"),
                ] {
                    ui.label(egui::RichText::new(text).size(11.0).color(color));
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("16 direcciones")
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Haz clic en un eje para ajustarlo")
                            .size(11.0)
                            .color(colors::TEXT_MUTED),
                    );
                });
            });
        });
    }

    fn response_controls(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            let inner_width = (width - 34.0).max(0.0);
            ui.set_width(inner_width);
            section_heading(
                ui,
                "A tu manera",
                "Elige una base y encuentra tu punto de control.",
            );
            ui.add_space(2.0);
            let presets = [
                ("Neutro", "Respuesta 1:1", "○", [1.0; NUM_DIRECTIONS]),
                (
                    "Flick táctico",
                    "Movimientos rápidos",
                    "↗",
                    Profile::preset_tactical_flick().direction_multipliers,
                ),
                (
                    "Tracking suave",
                    "Seguimiento continuo",
                    "∿",
                    Profile::preset_tracking_smooth().direction_multipliers,
                ),
                (
                    "Microajuste",
                    "Ajustes de hasta ±4%",
                    "+",
                    [
                        1.02, 1.03, 1.04, 1.03, 1.02, 1.00, 0.98, 0.97, 0.98, 1.00, 1.02, 1.03,
                        1.04, 1.03, 1.02, 1.01,
                    ],
                ),
            ];
            let capsule_width = ((inner_width - 12.0) / 2.0).floor();
            for row in presets.chunks(2) {
                ui.horizontal(|ui| {
                    for (title, detail, glyph, multipliers) in row {
                        if quick_preset_capsule(
                            ui,
                            capsule_width,
                            title,
                            detail,
                            glyph,
                            profile.direction_multipliers == *multipliers,
                        ) {
                            profile.direction_multipliers = *multipliers;
                            profile.recompute_fourier();
                        }
                    }
                });
            }
            ui.add_space(4.0);
            let strength_label = ui
                .horizontal(|ui| {
                    let label = ui.label(
                        egui::RichText::new("Intensidad")
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.0}%", profile.strength * 100.0))
                                .monospace()
                                .size(13.0)
                                .color(accent(ui)),
                        );
                    });
                    label
                })
                .inner;
            ui.spacing_mut().slider_width = inner_width;
            ui.add(
                egui::Slider::new(&mut profile.strength, 0.0..=1.5)
                    .show_value(false)
                    .step_by(0.01),
            )
            .labelled_by(strength_label.id)
            .on_hover_text("Intensidad de curva: 0% neutro · 100% curva completa · 150% refuerzo");
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("0% · Neutro")
                        .size(10.5)
                        .color(colors::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("150% · Refuerzo")
                            .size(10.5)
                            .color(colors::TEXT_MUTED),
                    );
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                section_heading(ui, "Ajuste por dirección", "");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("Restablecer")
                        .on_hover_text("Restablecer los 16 ejes a 1:1")
                        .clicked()
                    {
                        profile.direction_multipliers = [1.0; NUM_DIRECTIONS];
                        profile.recompute_fourier();
                    }
                });
            });
            let mut index = self.selected_dir_index.unwrap_or(0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Dirección")
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                egui::ComboBox::from_id_salt("selected-direction")
                    .width(110.0)
                    .selected_text(format!("{:.1}°", index as f32 * 22.5))
                    .show_ui(ui, |ui| {
                        for i in 0..NUM_DIRECTIONS {
                            ui.selectable_value(&mut index, i, format!("{:.1}°", i as f32 * 22.5));
                        }
                    })
                    .response
                    .on_hover_text("Seleccionar dirección del mapa polar");
            });
            if index != self.selected_dir_index.unwrap_or(0) {
                self.selected_dir_index = Some(index);
            }
            let (_, changed) = direction_micro_bar(
                ui,
                index as f32 * 22.5,
                &mut profile.direction_multipliers[index],
                true,
            );
            if changed {
                self.selected_dir_index = Some(index);
                profile.recompute_fourier();
            }
            egui::CollapsingHeader::new("Ver las 16 direcciones")
                .id_salt("all-directions")
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("direction-list")
                        .min_scrolled_height(134.0)
                        .max_height(134.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for i in 0..NUM_DIRECTIONS {
                                let (clicked, changed) = direction_micro_bar(
                                    ui,
                                    i as f32 * 22.5,
                                    &mut profile.direction_multipliers[i],
                                    self.selected_dir_index == Some(i),
                                );
                                if clicked {
                                    self.selected_dir_index = Some(i);
                                }
                                if changed {
                                    profile.recompute_fourier();
                                }
                            }
                        });
                });
        });
    }
    fn sandbox(&mut self, ui: &mut egui::Ui, profile: &Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width((width - 34.0).max(0.0));
            ui.horizontal(|ui| {
                section_heading(ui, "Prueba tu movimiento", "");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Limpiar trazo").clicked() {
                        self.sandbox_trail.clear();
                        self.test_cursor = None;
                        self.sandbox_accumulator.reset();
                        self.last_delta = (0, 0);
                    }
                });
            });

            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 88.0), Sense::drag());
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 10, colors::SIDEBAR_BG);
            p.rect_stroke(
                rect,
                10,
                Stroke::new(1.0, colors::BORDER_SUBTLE),
                egui::StrokeKind::Inside,
            );

            // Matrix grid dots
            for x in (16..rect.width() as usize).step_by(22) {
                for y in (16..rect.height() as usize).step_by(22) {
                    p.circle_filled(
                        rect.min + egui::vec2(x as f32, y as f32),
                        0.7,
                        colors::BORDER_STRONG,
                    );
                }
            }

            if response.drag_started() {
                self.test_cursor = Some(rect.size() * 0.5);
                self.sandbox_accumulator.reset();
            }
            if response.dragged() {
                let (x, y) = preview_delta(profile, response.drag_delta());
                let (x, y) = self.sandbox_accumulator.apply(x, y);
                self.last_delta = (x, y);
                let from = self.test_cursor.unwrap_or(rect.size() * 0.5);
                let to = (from + egui::vec2(x as f32, y as f32))
                    .clamp(Vec2::splat(6.0), rect.size() - Vec2::splat(6.0));
                self.sandbox_trail.push((from, to));
                if self.sandbox_trail.len() > 240 {
                    self.sandbox_trail.remove(0);
                }
                self.test_cursor = Some(to);
            }

            for (from, to) in &self.sandbox_trail {
                p.line_segment(
                    [rect.min + *from, rect.min + *to],
                    Stroke::new(2.2, accent(ui)),
                );
            }

            // Floating peak telemetry badge in top-right of canvas (Ref A style)
            let badge_w = 118.0;
            let badge_h = 24.0;
            let badge_rect = egui::Rect::from_min_size(
                rect.right_top() + egui::vec2(-badge_w - 12.0, 10.0),
                egui::vec2(badge_w, badge_h),
            );
            p.rect_filled(badge_rect, 12, colors::PANEL_BG);
            p.rect_stroke(
                badge_rect,
                12,
                Stroke::new(1.0, colors::BORDER_STRONG),
                egui::StrokeKind::Inside,
            );
            p.text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("Δx {:+}  Δy {:+}", self.last_delta.0, self.last_delta.1),
                egui::FontId::monospace(10.0),
                if self.last_delta != (0, 0) {
                    colors::SUCCESS
                } else {
                    colors::TEXT_MUTED
                },
            );

            if let Some(cursor) = self.test_cursor {
                let cursor = (rect.min + cursor)
                    .clamp(rect.min + Vec2::splat(5.0), rect.max - Vec2::splat(5.0));
                p.circle_stroke(cursor, 7.0, Stroke::new(1.0, accent(ui)));
                p.circle_filled(cursor, 4.0, colors::WARNING);
            } else {
                p.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Haz clic y arrastra en este espacio para probar tu curva",
                    egui::FontId::proportional(12.5),
                    colors::TEXT_MUTED,
                );
            }

            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(
                    "Vista previa local · Arrastra para comparar la respuesta de tu curva.",
                )
                .size(11.0)
                .color(colors::TEXT_FAINT),
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paused_preview_preserves_raw_movement() {
        let mut profile = Profile::preset_tactical_flick();
        profile.is_active = false;
        let (x, y) = preview_delta(&profile, Vec2::new(100.0, 0.0));
        assert!((x - 100.0).abs() < 0.001);
        assert!(y.abs() < 0.001);
    }
    #[test]
    fn active_preview_uses_profile_strength() {
        let mut profile = Profile::preset_tactical_flick();
        profile.is_active = true;
        let (x, _) = preview_delta(&profile, Vec2::new(100.0, 0.0));
        assert!(x < 100.0);
        profile.strength = 0.0;
        assert!((preview_delta(&profile, Vec2::new(100.0, 0.0)).0 - 100.0).abs() < 0.001);
    }
}
