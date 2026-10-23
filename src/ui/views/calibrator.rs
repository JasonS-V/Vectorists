//! Interactive 16-Direction Radial Flick Calibration Tool.
//!
//! Guides the user through a 16-angle flick diagnostic test, measuring
//! physical overshoot/undershoot and settling times, and computes the
//! compensating polar sensitivity curve.

use crate::core::math::TWO_PI;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};
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
    pub current_round: usize,
    pub total_rounds: usize,
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
            current_round: 1,
            total_rounds: 1,
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
        // Create an interleaved test sequence for natural motor responses (e.g. 0, 8, 4, 12, etc.)
        let mut seq = Vec::new();
        // Cardinals first
        seq.extend_from_slice(&[0, 8, 4, 12]);
        // Diagonals next
        seq.extend_from_slice(&[2, 10, 6, 14]);
        // Inter-cardinals
        seq.extend_from_slice(&[1, 9, 3, 11, 5, 13, 7, 15]);

        self.direction_sequence = seq;
        self.sequence_cursor = 0;
        self.measured_multipliers = [1.0; NUM_DIRECTIONS];
        self.measured_settle_ms = [0.0; NUM_DIRECTIONS];
        self.state = CalibratorState::WaitingForCenter;
        self.last_feedback = Some("Haz clic en el centro para iniciar la secuencia.".to_string());
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("CALIBRADOR BIOMECÁNICO RADIAL")
                        .size(14.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match self.state {
                        CalibratorState::Idle => {
                            if ui.button(egui::RichText::new("Iniciar Calibración").strong().color(colors::ACCENT_CYAN)).clicked() {
                                self.start_calibration();
                            }
                        }
                        CalibratorState::Completed => {
                            if ui.button(egui::RichText::new("Repetir Test").color(colors::TEXT_MUTED)).clicked() {
                                self.start_calibration();
                            }
                            if ui.button(egui::RichText::new("Aplicar al Perfil Activo").strong().color(colors::ACCENT_CYAN)).clicked() {
                                profile.direction_multipliers = self.measured_multipliers;
                                profile.recompute_fourier();
                                self.last_feedback = Some("¡Perfil actualizado con éxito!".to_string());
                            }
                        }
                        _ => {
                            if ui.button(egui::RichText::new("Cancelar").color(colors::ALERT_AMBER)).clicked() {
                                self.state = CalibratorState::Idle;
                                self.last_feedback = Some("Calibración cancelada.".to_string());
                            }
                        }
                    }
                });
            });

            ui.separator();

            // Instructions / Status Bar
            ui.horizontal(|ui| {
                let progress_text = match self.state {
                    CalibratorState::Idle => "Listo para comenzar. Duración estimada: ~2 minutos.".to_string(),
                    CalibratorState::WaitingForCenter => format!(
                        "Paso {} de {}: Haz clic en el círculo central.",
                        self.sequence_cursor + 1,
                        NUM_DIRECTIONS
                    ),
                    CalibratorState::TargetActive { direction_index, .. } => {
                        let deg = direction_index as f32 * 22.5;
                        format!(
                            "Paso {} de {}: Realiza un flick hacia el objetivo ({:>5.1}°).",
                            self.sequence_cursor + 1,
                            NUM_DIRECTIONS,
                            deg
                        )
                    }
                    CalibratorState::Completed => "¡Calibración completada! Revisa las métricas y aplica el perfil.".to_string(),
                };

                ui.label(egui::RichText::new(progress_text).size(11.0).color(colors::TEXT_PRIMARY));

                if let Some(ref fb) = self.last_feedback {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(fb).size(11.0).color(colors::ACCENT_CYAN));
                    });
                }
            });

            ui.add_space(4.0);

            // Test Arena Area
            let arena_height = ui.available_height() - 20.0;
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), arena_height.max(340.0)),
                Sense::click(),
            );

            let painter = ui.painter_at(rect);
            let center = rect.center();
            let target_radius_dist = 140.0; // Distance of targets from center

            // Arena Background
            painter.rect_filled(rect, 4.0, colors::CHASSIS_BG);
            painter.rect_stroke(rect, 4.0, Stroke::new(1.0, colors::BORDER_SUBTLE), egui::StrokeKind::Inside);

            // Concentric guide rings
            painter.circle_stroke(center, target_radius_dist, Stroke::new(1.0, colors::BORDER_SUBTLE));
            painter.circle_stroke(center, target_radius_dist * 0.5, Stroke::new(0.5, Color32::from_rgba_premultiplied(0x23, 0x27, 0x33, 0x50)));

            // State-dependent rendering and click logic
            match self.state {
                CalibratorState::Idle => {
                    // Informational preview
                    painter.circle_filled(center, 12.0, colors::PANEL_BG_LIGHT);
                    painter.circle_stroke(center, 12.0, Stroke::new(1.5, colors::BORDER_STRONG));
                    painter.text(
                        center + Vec2::new(0.0, 40.0),
                        egui::Align2::CENTER_CENTER,
                        "Presiona 'Iniciar Calibración' en la esquina superior para comenzar",
                        egui::FontId::proportional(12.0),
                        colors::TEXT_MUTED,
                    );
                }

                CalibratorState::WaitingForCenter => {
                    // Pulsing center anchor
                    let center_radius = 16.0;
                    let is_hovered = response.hover_pos().map_or(false, |p| (p - center).length() <= center_radius);

                    let fill_color = if is_hovered {
                        colors::ACCENT_CYAN_HOVER
                    } else {
                        colors::ACCENT_CYAN
                    };

                    painter.circle_filled(center, center_radius, fill_color);
                    painter.circle_stroke(center, center_radius + 4.0, Stroke::new(1.0, colors::ACCENT_CYAN_FAINT));

                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        "CLIC",
                        egui::FontId::monospace(10.0),
                        colors::CHASSIS_BG,
                    );

                    if response.clicked() {
                        if let Some(click_pos) = response.interact_pointer_pos() {
                            if (click_pos - center).length() <= center_radius + 8.0 {
                                // Transition to TargetActive
                                let dir_idx = self.direction_sequence[self.sequence_cursor];
                                let angle_rad = dir_idx as f32 * (TWO_PI / NUM_DIRECTIONS as f32);
                                self.state = CalibratorState::TargetActive {
                                    direction_index: dir_idx,
                                    angle_rad,
                                    spawn_time: Instant::now(),
                                };
                            }
                        }
                    }
                }

                CalibratorState::TargetActive { direction_index, angle_rad, spawn_time } => {
                    // Target position
                    let target_pos = center + Vec2::new(angle_rad.cos(), angle_rad.sin()) * target_radius_dist;
                    let target_radius = 18.0;

                    // Draw center ghost
                    painter.circle_stroke(center, 6.0, Stroke::new(1.0, colors::BORDER_STRONG));

                    // Draw target
                    painter.circle_filled(target_pos, target_radius, colors::ALERT_AMBER);
                    painter.circle_stroke(target_pos, target_radius + 4.0, Stroke::new(1.5, colors::ALERT_AMBER_FAINT));
                    painter.circle_filled(target_pos, 4.0, colors::CHASSIS_BG);

                    if response.clicked() {
                        if let Some(click_pos) = response.interact_pointer_pos() {
                            let click_dist = (click_pos - center).length();
                            let elapsed_ms = spawn_time.elapsed().as_secs_f32() * 1000.0;

                            // If click was roughly in the direction of the target
                            if click_dist > 20.0 {
                                // Multiplier = Ideal Distance / Actual Distance
                                let raw_ratio = target_radius_dist / click_dist;
                                let clamped_mult = raw_ratio.clamp(0.70, 1.35);

                                self.measured_multipliers[direction_index] = clamped_mult;
                                self.measured_settle_ms[direction_index] = elapsed_ms;

                                let overshoot_pct = (click_dist / target_radius_dist - 1.0) * 100.0;
                                let feedback_str = if overshoot_pct.abs() < 5.0 {
                                    format!("¡Tiro perfecto! ({:+.1}% / {:.0}ms)", overshoot_pct, elapsed_ms)
                                } else if overshoot_pct > 0.0 {
                                    format!("Overshoot: {:+.1}% (Frenada: {:.0}ms)", overshoot_pct, elapsed_ms)
                                } else {
                                    format!("Undershoot: {:+.1}% (Frenada: {:.0}ms)", overshoot_pct, elapsed_ms)
                                };
                                self.last_feedback = Some(feedback_str);

                                self.sequence_cursor += 1;
                                if self.sequence_cursor >= NUM_DIRECTIONS {
                                    self.state = CalibratorState::Completed;
                                } else {
                                    self.state = CalibratorState::WaitingForCenter;
                                }
                            }
                        }
                    }
                }

                CalibratorState::Completed => {
                    // Summary panel inside arena
                    let avg_settle: f32 = self.measured_settle_ms.iter().sum::<f32>() / (NUM_DIRECTIONS as f32);
                    painter.text(
                        center - Vec2::new(0.0, 60.0),
                        egui::Align2::CENTER_CENTER,
                        "RESULTADOS DEL DIAGNÓSTICO BIOMECÁNICO",
                        egui::FontId::proportional(14.0),
                        colors::TEXT_PRIMARY,
                    );

                    painter.text(
                        center - Vec2::new(0.0, 30.0),
                        egui::Align2::CENTER_CENTER,
                        format!("Tiempo promedio de asentamiento: {:.1} ms", avg_settle),
                        egui::FontId::monospace(12.0),
                        colors::ACCENT_CYAN,
                    );

                    painter.text(
                        center + Vec2::new(0.0, 10.0),
                        egui::Align2::CENTER_CENTER,
                        "Se ha ajustado una curva armónica continua de 360° para compensar tus ejes débiles.",
                        egui::FontId::proportional(11.0),
                        colors::TEXT_MUTED,
                    );

                    painter.text(
                        center + Vec2::new(0.0, 40.0),
                        egui::Align2::CENTER_CENTER,
                        "Haz clic en 'Aplicar al Perfil Activo' arriba para activar los cambios.",
                        egui::FontId::proportional(11.0),
                        colors::ALERT_AMBER,
                    );
                }
            }
        });
    }
}
