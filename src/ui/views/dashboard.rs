//! Dashboard View for Vectorists.
//!
//! Visualizes the polar sensitivity map, profile parameters, manual 16-direction
//! tuning sliders, and an interactive real-time vector test canvas.

use crate::core::accumulator::SubpixelAccumulator;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use crate::ui::widgets::PolarRadarWidget;
use eframe::egui::{self, Pos2, Sense, Stroke, Vec2};

pub struct DashboardView {
    pub selected_dir_index: Option<usize>,
    pub sandbox_accumulator: SubpixelAccumulator,
    pub sandbox_trail: Vec<(Pos2, Pos2)>,
    pub test_cursor: Pos2,
}

impl Default for DashboardView {
    fn default() -> Self {
        Self {
            selected_dir_index: None,
            sandbox_accumulator: SubpixelAccumulator::new(),
            sandbox_trail: Vec::new(),
            test_cursor: Pos2::new(100.0, 100.0),
        }
    }
}

impl DashboardView {
    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        ui.horizontal(|ui| {
            // Left Column: Polar Radar Widget
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.set_width(340.0);
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new("MAPA POLAR 360°")
                                .size(14.0)
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let status_text = if profile.is_active {
                                egui::RichText::new("● ACTIVO").color(colors::SUCCESS_GREEN).size(11.0)
                            } else {
                                egui::RichText::new("○ EN ESPERA").color(colors::TEXT_MUTED).size(11.0)
                            };
                            ui.label(status_text);
                        });
                    });

                    ui.separator();

                    let radar = PolarRadarWidget::new(profile)
                        .size(Vec2::new(320.0, 320.0))
                        .selected_index(self.selected_dir_index);
                    ui.add(radar);

                    ui.separator();

                    // Legend
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("— Base (1:1)").color(colors::BORDER_STRONG).size(10.0));
                        ui.label(egui::RichText::new("● Medido").color(colors::ALERT_AMBER).size(10.0));
                        ui.label(egui::RichText::new("— Corregido").color(colors::ACCENT_CYAN).size(10.0));
                    });
                });
            });

            ui.add_space(8.0);

            // Right Column: Controls, Statistics, and Direction Tuning
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("PERFIL:")
                                .color(colors::TEXT_MUTED)
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(&profile.name)
                                .color(colors::ACCENT_CYAN)
                                .strong()
                                .size(13.0),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("{} DPI | {} Hz", profile.dpi, profile.polling_rate_hz))
                                    .color(colors::TEXT_FAINT)
                                    .monospace()
                                    .size(10.0),
                            );
                        });
                    });

                    ui.add_space(4.0);

                    // Strength Slider
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Fuerza de Asistencia:").color(colors::TEXT_PRIMARY).size(11.0));
                        let pct = (profile.strength * 100.0).round() as i32;
                        ui.label(
                            egui::RichText::new(format!("{}%", pct))
                                .color(colors::ACCENT_CYAN)
                                .monospace()
                                .size(11.0),
                        );
                    });

                    let mut strength_val = profile.strength;
                    if ui.add(egui::Slider::new(&mut strength_val, 0.0..=1.5).show_value(false)).changed() {
                        profile.strength = strength_val;
                    }

                    ui.add_space(6.0);
                    ui.separator();

                    // Biomechanical metrics summary
                    let (min_idx, min_val) = profile
                        .direction_multipliers
                        .iter()
                        .enumerate()
                        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                        .unwrap();

                    let (max_idx, max_val) = profile
                        .direction_multipliers
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                        .unwrap();

                    let min_deg = min_idx as f32 * 22.5;
                    let max_deg = max_idx as f32 * 22.5;

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Déficit Máx (Undershoot):").color(colors::TEXT_MUTED).size(10.0));
                            let delta = (*max_val - 1.0) * 100.0;
                            ui.label(
                                egui::RichText::new(format!("{:>5.1}°  ({:+4.1}%)", max_deg, delta))
                                    .color(colors::ACCENT_CYAN)
                                    .monospace()
                                    .size(11.0),
                            );
                        });

                        ui.add_space(16.0);

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Exceso Máx (Overshoot):").color(colors::TEXT_MUTED).size(10.0));
                            let delta = (*min_val - 1.0) * 100.0;
                            ui.label(
                                egui::RichText::new(format!("{:>5.1}°  ({:+4.1}%)", min_deg, delta))
                                    .color(colors::ALERT_AMBER)
                                    .monospace()
                                    .size(11.0),
                            );
                        });
                    });
                });

                ui.add_space(4.0);

                // Manual 16-Direction Tuning Area
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("CALIBRACIÓN MANUAL (16 EJES)").size(11.0).color(colors::TEXT_PRIMARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("Restablecer 1:1").size(10.0)).clicked() {
                                profile.direction_multipliers = [1.0; NUM_DIRECTIONS];
                                profile.recompute_fourier();
                            }
                        });
                    });

                    ui.add_space(4.0);

                    egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        egui::Grid::new("directions_grid")
                            .num_columns(4)
                            .spacing([8.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for i in 0..NUM_DIRECTIONS {
                                    let angle_deg = i as f32 * 22.5;
                                    let is_selected = self.selected_dir_index == Some(i);

                                    let label = egui::RichText::new(format!("{:>5.1}°", angle_deg))
                                        .monospace()
                                        .size(10.0)
                                        .color(if is_selected {
                                            colors::ACCENT_CYAN
                                        } else {
                                            colors::TEXT_MUTED
                                        });

                                    if ui.selectable_label(is_selected, label).clicked() {
                                        self.selected_dir_index = if is_selected { None } else { Some(i) };
                                    }

                                    let mut mult = profile.direction_multipliers[i];
                                    let slider = egui::Slider::new(&mut mult, 0.6..=1.4)
                                        .show_value(false);

                                    if ui.add(slider).changed() {
                                        profile.direction_multipliers[i] = mult;
                                        profile.recompute_fourier();
                                    }

                                    let delta_pct = (mult - 1.0) * 100.0;
                                    ui.label(
                                        egui::RichText::new(format!("{:>4.2}x ({:+4.0}%)", mult, delta_pct))
                                            .monospace()
                                            .size(10.0)
                                            .color(if delta_pct >= 0.0 {
                                                colors::ACCENT_CYAN
                                            } else {
                                                colors::ALERT_AMBER
                                            }),
                                    );

                                    ui.end_row();
                                }
                            });
                    });
                });

                ui.add_space(4.0);

                // Interactive Vector Sandbox
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SIMULADOR DE ENTRADA").size(11.0).color(colors::TEXT_PRIMARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("Limpiar Trazo").size(10.0)).clicked() {
                                self.sandbox_trail.clear();
                                self.sandbox_accumulator.reset();
                            }
                        });
                    });

                    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 90.0), Sense::drag());
                    let painter = ui.painter_at(rect);

                    // Background
                    painter.rect_filled(rect, 4.0, colors::CHASSIS_BG);
                    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, colors::BORDER_SUBTLE), egui::StrokeKind::Inside);

                    let center = rect.center();

                    // If dragging inside sandbox
                    if response.dragged() {
                        let delta = response.drag_delta();
                        if delta.length_sq() > 0.0 {
                            let (corr_dx, corr_dy) = profile.fourier.correct_delta(delta.x, delta.y, profile.strength);
                            let (out_dx, out_dy) = self.sandbox_accumulator.apply(corr_dx, corr_dy);

                            let new_pos = Pos2::new(
                                (self.test_cursor.x + out_dx as f32).clamp(rect.left() + 4.0, rect.right() - 4.0),
                                (self.test_cursor.y + out_dy as f32).clamp(rect.top() + 4.0, rect.bottom() - 4.0),
                            );

                            self.sandbox_trail.push((self.test_cursor, new_pos));
                            if self.sandbox_trail.len() > 100 {
                                self.sandbox_trail.remove(0);
                            }
                            self.test_cursor = new_pos;
                        }
                    } else if !rect.contains(self.test_cursor) {
                        self.test_cursor = center;
                    }

                    // Draw trail
                    for (from, to) in &self.sandbox_trail {
                        painter.line_segment([*from, *to], Stroke::new(1.5, colors::ACCENT_CYAN));
                    }

                    // Draw current cursor position
                    painter.circle_filled(self.test_cursor, 4.0, colors::ALERT_AMBER);
                    painter.circle_stroke(self.test_cursor, 5.0, Stroke::new(1.0, colors::CHASSIS_BG));

                    if !response.dragged() && self.sandbox_trail.is_empty() {
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "Arrastra el ratón aquí para probar el vector en tiempo real",
                            egui::FontId::proportional(11.0),
                            colors::TEXT_FAINT,
                        );
                    }
                });
            });
        });
    }
}
