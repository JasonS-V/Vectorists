//! Response workspace: live telemetry, polar inspection and preview controls.
use crate::core::accumulator::SubpixelAccumulator;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use crate::ui::widgets::radar::direction_index;
use crate::ui::widgets::{
    PolarRadarWidget, card_container, direction_micro_bar, metric_card, pill_chip, section_heading,
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
        let metric_width = (width - 24.0) / 3.0;
        ui.horizontal_top(|ui| {
            metric_card(
                ui,
                metric_width,
                "Frecuencia del sensor",
                &if hz > 0 {
                    format!("{hz} Hz")
                } else {
                    "En espera".into()
                },
                &if hz > 0 {
                    format!("{cps:.0} cuentas por segundo")
                } else {
                    "Mueve el ratón para medir".into()
                },
            );
            let max = profile
                .direction_multipliers
                .iter()
                .map(|m| (m - 1.0).abs())
                .fold(0.0_f32, f32::max);
            metric_card(
                ui,
                metric_width,
                "Ajuste direccional máx.",
                &format!("{:.1}%", max * 100.0),
                "Sobre la respuesta neutra 1:1",
            );
            metric_card(
                ui,
                metric_width,
                "Intensidad de corrección",
                &format!("{:.0}%", profile.strength * 100.0),
                if profile.is_active {
                    "Vista previa de la curva activa"
                } else {
                    "Vista previa en pausa · 1:1"
                },
            );
        });
        ui.add_space(8.0);
        if width >= 780.0 {
            ui.horizontal_top(|ui| {
                let left = (width * 0.53).floor();
                ui.allocate_ui_with_layout(
                    egui::vec2(left, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.map_card(ui, profile);
                    },
                );
                ui.vertical(|ui| {
                    ui.set_width((width - left - 12.0).max(0.0));
                    self.tuning_card(ui, profile);
                    self.directions_card(ui, profile);
                });
            });
        } else {
            self.map_card(ui, profile);
            self.tuning_card(ui, profile);
            self.directions_card(ui, profile);
        }
        ui.add_space(8.0);
        self.sandbox(ui, profile);
    }

    fn map_card(&mut self, ui: &mut egui::Ui, profile: &Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Tu respuesta en 360°",
                "Explora la curva y selecciona una dirección.",
            );
            let plot_height = (ui.ctx().viewport_rect().height() - 480.0).clamp(300.0, 400.0);
            let side = ui.available_width().min(plot_height);
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
                    (colors::ACCENT_CYAN, "— Curva efectiva"),
                    (colors::ALERT_AMBER, "● Ajustes por eje"),
                    (colors::TEXT_FAINT, "○ Referencia 1:1"),
                ] {
                    ui.label(egui::RichText::new(text).size(11.0).color(color));
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if let Some(i) = self.selected_dir_index {
                    ui.label(
                        egui::RichText::new(format!("EJE {:.1}°", i as f32 * 22.5))
                            .size(11.0)
                            .color(colors::TEXT_MUTED),
                    );
                    ui.label(
                        egui::RichText::new(format!("{:.3}×", profile.direction_multipliers[i]))
                            .monospace()
                            .color(colors::ACCENT_CYAN),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("16 ejes de ajuste")
                            .size(12.0)
                            .color(colors::TEXT_MUTED),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Interpolación suave · N = 4")
                            .size(11.0)
                            .color(colors::TEXT_FAINT),
                    );
                });
            });
        });
    }

    fn tuning_card(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Define tu respuesta",
                "Empieza con una base y hazla tuya.",
            );
            ui.horizontal_wrapped(|ui| {
                for (label, multipliers) in [
                    ("Neutro 1:1", [1.0; NUM_DIRECTIONS]),
                    (
                        "Flick táctico",
                        Profile::preset_tactical_flick().direction_multipliers,
                    ),
                    (
                        "Tracking suave",
                        Profile::preset_tracking_smooth().direction_multipliers,
                    ),
                ] {
                    let active = profile.direction_multipliers == multipliers;
                    if pill_chip(ui, label, active, colors::ACCENT_CYAN) {
                        profile.direction_multipliers = multipliers;
                        profile.recompute_fourier();
                    }
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Intensidad");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.0}%", profile.strength * 100.0))
                            .monospace()
                            .color(colors::ACCENT_CYAN),
                    );
                });
            });
            ui.spacing_mut().slider_width = ui.available_width();
            let label = ui.label(
                egui::RichText::new("75% recomendado para empezar")
                    .size(11.0)
                    .color(colors::TEXT_FAINT),
            );
            ui.add(
                egui::Slider::new(&mut profile.strength, 0.0..=1.5)
                    .show_value(false)
                    .step_by(0.01),
            )
            .labelled_by(label.id)
            .on_hover_text("Intensidad: 0% = neutro · 100% = curva completa · 150% = refuerzo");
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("0% · Neutro")
                        .size(10.0)
                        .color(colors::TEXT_FAINT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("150% · Refuerzo")
                            .size(10.0)
                            .color(colors::TEXT_FAINT),
                    );
                });
            });
        });
    }

    fn directions_card(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
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
            ui.label(
                egui::RichText::new("Selecciona un eje y ajusta su multiplicador.")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            egui::ScrollArea::vertical()
                .id_salt("direction-list")
                .min_scrolled_height(128.0)
                .max_height(128.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..NUM_DIRECTIONS {
                        let selected = self.selected_dir_index == Some(i);
                        let (clicked, changed) = direction_micro_bar(
                            ui,
                            i as f32 * 22.5,
                            &mut profile.direction_multipliers[i],
                            selected,
                        );
                        if clicked {
                            self.selected_dir_index = if selected { None } else { Some(i) };
                        }
                        if changed {
                            profile.recompute_fourier();
                        }
                    }
                });
        });
    }

    fn sandbox(&mut self, ui: &mut egui::Ui, profile: &Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            ui.horizontal(|ui| {
                section_heading(ui, "Prueba el movimiento", "");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Limpiar trazo").clicked() {
                        self.sandbox_trail.clear();
                        self.test_cursor = None;
                        self.sandbox_accumulator.reset();
                        self.last_delta = (0, 0);
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "Δx {:+}  /  Δy {:+}",
                            self.last_delta.0, self.last_delta.1
                        ))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_FAINT),
                    );
                });
            });
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 112.0), Sense::drag());
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 6, colors::SIDEBAR_BG);
            for x in (16..rect.width() as usize).step_by(20) {
                for y in (16..rect.height() as usize).step_by(20) {
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
                    Stroke::new(1.7, colors::ACCENT_CYAN),
                );
            }
            if let Some(cursor) = self.test_cursor {
                let cursor = (rect.min + cursor)
                    .clamp(rect.min + Vec2::splat(5.0), rect.max - Vec2::splat(5.0));
                p.circle_filled(cursor, 4.0, colors::ALERT_AMBER);
            } else {
                p.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Haz clic y arrastra para probar tu curva",
                    egui::FontId::proportional(13.0),
                    colors::TEXT_MUTED,
                );
            }
            ui.label(egui::RichText::new("Vista previa local. Exporta tu perfil para utilizarlo con un controlador externo.").size(11.0).color(colors::TEXT_FAINT));
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
