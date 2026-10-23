//! Guided radial flick calibration with explicit progress and reviewable results.
use crate::core::math::TWO_PI;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use crate::ui::widgets::{card_container, primary_button, section_heading};
use eframe::egui::{self, Pos2, Sense, Stroke, Vec2};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalibratorState {
    Idle,
    WaitingForCenter,
    TargetActive {
        direction_index: usize,
        angle_rad: f32,
        spawn_time: Instant,
    },
    Completed,
}

pub struct CalibratorView {
    pub state: CalibratorState,
    pub measured_multipliers: [f32; NUM_DIRECTIONS],
    pub measured_settle_ms: [f32; NUM_DIRECTIONS],
    pub direction_sequence: Vec<usize>,
    pub sequence_cursor: usize,
    pub last_feedback: Option<String>,
}

impl Default for CalibratorView {
    fn default() -> Self {
        Self {
            state: CalibratorState::Idle,
            measured_multipliers: [1.0; NUM_DIRECTIONS],
            measured_settle_ms: [0.0; NUM_DIRECTIONS],
            direction_sequence: (0..NUM_DIRECTIONS).collect(),
            sequence_cursor: 0,
            last_feedback: None,
        }
    }
}

impl CalibratorView {
    pub fn start_calibration(&mut self) {
        // Alternate opposing axes to balance fatigue and directional predictability.
        self.direction_sequence = vec![0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 3, 11, 5, 13, 7, 15];
        self.sequence_cursor = 0;
        self.measured_multipliers = [1.0; NUM_DIRECTIONS];
        self.measured_settle_ms = [0.0; NUM_DIRECTIONS];
        self.state = CalibratorState::WaitingForCenter;
        self.last_feedback = None;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        let width = ui.available_width();
        if self.state == CalibratorState::Idle && ui.ctx().viewport_rect().height() >= 700.0 {
            card_container(ui, |ui| {
                ui.set_width(width - 34.0);
                section_heading(
                    ui,
                    "Una curva que empieza contigo",
                    "Mantén tu agarre habitual y mueve el ratón con naturalidad.",
                );
                ui.add_space(4.0);
                ui.columns(3, |columns| {
                    for (column, (number, title, hint)) in columns.iter_mut().zip([
                        ("01", "Centra", "Haz clic en el punto central."),
                        ("02", "Apunta", "Muévete al objetivo y haz clic."),
                        ("03", "Repite", "Completa las 16 direcciones."),
                    ]) {
                        column.label(
                            egui::RichText::new(number)
                                .monospace()
                                .color(colors::ACCENT_CYAN),
                        );
                        column.strong(title);
                        column.label(
                            egui::RichText::new(hint)
                                .size(12.0)
                                .color(colors::TEXT_MUTED),
                        );
                    }
                });
            });
            ui.add_space(4.0);
        }
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(match self.state {
                        CalibratorState::Idle => "Listo cuando tú lo estés".to_owned(),
                        CalibratorState::Completed => "Calibración completada".to_owned(),
                        _ => format!("Dirección {} de {NUM_DIRECTIONS}", self.sequence_cursor + 1),
                    })
                    .size(16.0)
                    .strong(),
                );
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| match self.state {
                        CalibratorState::Idle => {
                            if primary_button(ui, "Iniciar calibración  →").clicked() {
                                self.start_calibration();
                            }
                        }
                        CalibratorState::Completed => {
                            if primary_button(ui, "Aplicar al perfil").clicked() {
                                profile.direction_multipliers = self.measured_multipliers;
                                profile.recompute_fourier();
                                self.last_feedback = Some(
                                    "Curva aplicada. Guarda los cambios para conservarla.".into(),
                                );
                            }
                            if ui.button("Repetir").clicked() {
                                self.start_calibration();
                            }
                        }
                        _ => {
                            if ui.button("Cancelar prueba").clicked() {
                                self.state = CalibratorState::Idle;
                                self.sequence_cursor = 0;
                                self.last_feedback = Some(
                                    "Prueba cancelada. Tu perfil conserva su curva anterior."
                                        .into(),
                                );
                            }
                        }
                    },
                );
            });
            let progress = self.sequence_cursor as f32 / NUM_DIRECTIONS as f32;
            ui.add(
                egui::ProgressBar::new(progress)
                    .fill(colors::ACCENT_CYAN)
                    .desired_height(4.0),
            );
            let instruction = match self.state {
                CalibratorState::Idle => {
                    "16 objetivos · un clic por dirección · sin modificar tu perfil hasta aplicar los resultados."
                }
                CalibratorState::WaitingForCenter => {
                    "Haz clic en el centro para mostrar el siguiente objetivo."
                }
                CalibratorState::TargetActive { .. } => {
                    "Apunta al objetivo ámbar y haz clic. Prioriza tu movimiento natural."
                }
                CalibratorState::Completed => {
                    "Revisa los resultados y aplica la curva al perfil cuando estés listo."
                }
            };
            ui.label(
                egui::RichText::new(instruction)
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            self.arena(ui);
            if let Some(message) = &self.last_feedback {
                ui.label(
                    egui::RichText::new(message)
                        .size(13.0)
                        .color(colors::ACCENT_CYAN),
                );
            }
        });
        if self.state == CalibratorState::Completed {
            self.results(ui);
        }
    }

    fn arena(&mut self, ui: &mut egui::Ui) {
        let height = (ui.clip_rect().bottom() - ui.cursor().top() - 40.0).clamp(180.0, 420.0);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::click());
        let p = ui.painter_at(rect);
        let center = rect.center();
        let distance = (rect.width().min(rect.height()) * 0.5 - 48.0).min(140.0);
        p.rect_filled(rect, 8, colors::SIDEBAR_BG);
        for multiplier in [0.5, 1.0, 1.25] {
            p.circle_stroke(
                center,
                distance * multiplier,
                Stroke::new(1.0, colors::BORDER_SUBTLE),
            );
        }
        for i in 0..NUM_DIRECTIONS {
            let direction = Vec2::angled(i as f32 / NUM_DIRECTIONS as f32 * TWO_PI);
            p.circle_filled(center + direction * distance, 2.0, colors::BORDER_STRONG);
        }
        p.line_segment(
            [
                Pos2::new(center.x, rect.top() + 24.0),
                Pos2::new(center.x, rect.bottom() - 24.0),
            ],
            Stroke::new(0.5, colors::BORDER_SUBTLE),
        );
        p.line_segment(
            [
                Pos2::new(rect.left() + 24.0, center.y),
                Pos2::new(rect.right() - 24.0, center.y),
            ],
            Stroke::new(0.5, colors::BORDER_SUBTLE),
        );
        match self.state {
            CalibratorState::Idle => {
                p.circle_filled(center, 32.0, colors::PANEL_BG_LIGHT);
                p.circle_stroke(center, 20.0, Stroke::new(1.5, colors::TEXT_FAINT));
                p.circle_stroke(center, 8.0, Stroke::new(1.5, colors::ACCENT_CYAN));
                p.text(
                    center + egui::vec2(0.0, 64.0),
                    egui::Align2::CENTER_CENTER,
                    "Tu próximo nivel de precisión",
                    egui::FontId::proportional(14.0),
                    colors::TEXT_MUTED,
                );
            }
            CalibratorState::WaitingForCenter => {
                p.circle_filled(center, 18.0, colors::ACCENT_CYAN);
                p.circle_stroke(center, 25.0, Stroke::new(1.0, colors::ACCENT_CYAN));
                p.circle_filled(center, 3.0, colors::CHASSIS_BG);
                p.text(
                    center + egui::vec2(0.0, 46.0),
                    egui::Align2::CENTER_CENTER,
                    "Haz clic aquí",
                    egui::FontId::proportional(13.0),
                    colors::ACCENT_CYAN,
                );
                if response.clicked()
                    && response
                        .interact_pointer_pos()
                        .is_some_and(|position| position.distance(center) <= 26.0)
                {
                    let index = self.direction_sequence[self.sequence_cursor];
                    self.state = CalibratorState::TargetActive {
                        direction_index: index,
                        angle_rad: index as f32 / NUM_DIRECTIONS as f32 * TWO_PI,
                        spawn_time: Instant::now(),
                    };
                }
            }
            CalibratorState::TargetActive {
                direction_index,
                angle_rad,
                spawn_time,
            } => {
                let target = center + Vec2::angled(angle_rad) * distance;
                p.circle_stroke(center, 5.0, Stroke::new(1.0, colors::TEXT_FAINT));
                p.line_segment([center, target], Stroke::new(1.0, colors::BORDER_STRONG));
                p.circle_filled(target, 16.0, colors::ALERT_AMBER);
                p.circle_stroke(target, 23.0, Stroke::new(1.0, colors::ALERT_AMBER));
                p.circle_filled(target, 4.0, colors::CHASSIS_BG);
                if response.clicked()
                    && let Some(position) = response.interact_pointer_pos()
                {
                    let clicked_distance = position.distance(center);
                    if clicked_distance > 20.0 {
                        self.measured_multipliers[direction_index] =
                            (distance / clicked_distance).clamp(0.70, 1.35);
                        self.measured_settle_ms[direction_index] =
                            spawn_time.elapsed().as_secs_f32() * 1000.0;
                        let error = (clicked_distance / distance - 1.0) * 100.0;
                        self.last_feedback = Some(format!(
                            "Último intento: {error:+.1}% de distancia · {:.0} ms de respuesta",
                            self.measured_settle_ms[direction_index]
                        ));
                        self.sequence_cursor += 1;
                        self.state = if self.sequence_cursor == NUM_DIRECTIONS {
                            CalibratorState::Completed
                        } else {
                            CalibratorState::WaitingForCenter
                        };
                    }
                }
            }
            CalibratorState::Completed => {
                p.circle_filled(
                    center - egui::vec2(0.0, 30.0),
                    30.0,
                    colors::SUCCESS_GREEN_FAINT,
                );
                let c = center - egui::vec2(0.0, 30.0);
                p.line_segment(
                    [c + egui::vec2(-12.0, 0.0), c + egui::vec2(-3.0, 9.0)],
                    Stroke::new(2.5, colors::SUCCESS_GREEN),
                );
                p.line_segment(
                    [c + egui::vec2(-3.0, 9.0), c + egui::vec2(13.0, -9.0)],
                    Stroke::new(2.5, colors::SUCCESS_GREEN),
                );
                p.text(
                    center + egui::vec2(0.0, 24.0),
                    egui::Align2::CENTER_CENTER,
                    "Tu curva está lista",
                    egui::FontId::proportional(23.0),
                    colors::TEXT_PRIMARY,
                );
                p.text(
                    center + egui::vec2(0.0, 58.0),
                    egui::Align2::CENTER_CENTER,
                    format!(
                        "16 / 16 direcciones · {:.0} ms de respuesta media",
                        self.measured_settle_ms.iter().sum::<f32>() / NUM_DIRECTIONS as f32
                    ),
                    egui::FontId::proportional(12.0),
                    colors::TEXT_MUTED,
                );
            }
        }
    }

    fn results(&self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Resultados por dirección",
                "Multiplicador propuesto y tiempo hasta el clic. Aún puedes repetir la prueba.",
            );
            ui.columns(4, |columns| {
                for (i, multiplier) in self.measured_multipliers.iter().enumerate() {
                    columns[i % 4].label(
                        egui::RichText::new(format!("{:.1}°   {multiplier:.3}×", i as f32 * 22.5))
                            .monospace()
                            .size(11.0),
                    );
                    columns[i % 4].label(
                        egui::RichText::new(format!("{:.0} ms", self.measured_settle_ms[i]))
                            .size(11.0)
                            .color(colors::TEXT_FAINT),
                    );
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restarting_calibration_resets_results_and_visits_every_axis() {
        let mut view = CalibratorView {
            measured_multipliers: [1.2; NUM_DIRECTIONS],
            sequence_cursor: 9,
            ..CalibratorView::default()
        };
        view.start_calibration();
        let mut directions = view.direction_sequence.clone();
        directions.sort_unstable();
        assert_eq!(directions, (0..NUM_DIRECTIONS).collect::<Vec<_>>());
        assert_eq!(view.state, CalibratorState::WaitingForCenter);
        assert_eq!(view.sequence_cursor, 0);
        assert_eq!(view.measured_multipliers, [1.0; NUM_DIRECTIONS]);
    }
}
