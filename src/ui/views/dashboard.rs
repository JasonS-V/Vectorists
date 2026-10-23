//! Dashboard View for Vectorists.
//!
//! Modern esports HUD featuring hero telemetry cards, tactical polar radar,
//! factory preset chips, sleek sliders, and a dot-matrix vector sandbox.

use crate::core::accumulator::SubpixelAccumulator;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use crate::ui::widgets::{metric_card, modern_slider, PolarRadarWidget};
use eframe::egui::{self, Color32, CornerRadius, Pos2, Sense, Stroke, Vec2};

pub struct DashboardView {
    pub selected_dir_index: Option<usize>,
    pub sandbox_accumulator: SubpixelAccumulator,
    pub sandbox_trail: Vec<(Pos2, Pos2)>,
    pub test_cursor: Pos2,
    pub last_delta: (i32, i32),
}

impl Default for DashboardView {
    fn default() -> Self {
        Self {
            selected_dir_index: None,
            sandbox_accumulator: SubpixelAccumulator::new(),
            sandbox_trail: Vec::new(),
            test_cursor: Pos2::new(100.0, 100.0),
            last_delta: (0, 0),
        }
    }
}

impl DashboardView {
    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &mut Profile, detected_hz: u32, detected_cps: f32) {
        ui.vertical(|ui| {
            // 1. Hero Telemetry Row (3 Sleek Metric Cards)
            ui.horizontal(|ui| {
                let card_width = ((ui.available_width() - 16.0) / 3.0).max(180.0);

                // Card 1: Live Hardware Sensor
                let (hz_val, hz_badge, hz_color) = if detected_hz > 0 {
                    (
                        format!("{} Hz", detected_hz),
                        "● STREAMING",
                        colors::SUCCESS_GREEN,
                    )
                } else {
                    (
                        format!("{} Hz", profile.polling_rate_hz),
                        "STANDBY",
                        colors::TEXT_FAINT,
                    )
                };
                metric_card(
                    ui,
                    card_width,
                    "Hardware Sensor",
                    &hz_val,
                    &format!("{:.0} CPS uncoalesced", detected_cps),
                    Some((hz_badge, hz_color)),
                );

                ui.add_space(8.0);

                // Card 2: Directional Bias Max Spread
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

                let spread_val = format!("{:+2.0}% / {:+2.0}%", (*max_val - 1.0) * 100.0, (*min_val - 1.0) * 100.0);
                let spread_sub = format!("Deficit: {:>3.0}° | Overshoot: {:>3.0}°", max_idx as f32 * 22.5, min_idx as f32 * 22.5);
                metric_card(
                    ui,
                    card_width,
                    "Anatomical Bias",
                    &spread_val,
                    &spread_sub,
                    Some(("16-AXIS", colors::ALERT_AMBER)),
                );

                ui.add_space(8.0);

                // Card 3: Compensation Engine
                let (status_text, status_color) = if profile.is_active {
                    ("ACTIVE", colors::ACCENT_CYAN)
                } else {
                    ("PAUSED", colors::ALERT_AMBER)
                };
                metric_card(
                    ui,
                    card_width,
                    "Harmonic Engine",
                    &format!("{:.0}% α", profile.strength * 100.0),
                    &format!("N=4 Fourier | {} DPI", profile.dpi),
                    Some((status_text, status_color)),
                );
            });

            ui.add_space(10.0);

            // 2. Main Two-Column Split
            ui.horizontal(|ui| {
                // Left Column: Tactical Polar Radar
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.set_width(330.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("360° POLAR RESPONSE MAP")
                                    .size(11.0)
                                    .strong()
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new("HARMONIC N=4")
                                        .monospace()
                                        .size(9.0)
                                        .color(colors::ACCENT_CYAN),
                                );
                            });
                        });

                        ui.separator();

                        let radar = PolarRadarWidget::new(profile)
                            .size(Vec2::new(310.0, 310.0))
                            .selected_index(self.selected_dir_index);
                        ui.add(radar);

                        ui.separator();

                        // Legend Chips
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("— Baseline").color(colors::BORDER_STRONG).size(9.5));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("● Raw Anatomy").color(colors::ALERT_AMBER).size(9.5));
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("— Fourier Curve").color(colors::ACCENT_CYAN).size(9.5));
                        });
                    });
                });

                ui.add_space(8.0);

                // Right Column: Presets, Fine Tuning, and Vector Simulator
                ui.vertical(|ui| {
                    // Profile & Presets Bar
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("PRESETS:").size(10.0).color(colors::TEXT_MUTED));
                            if ui.button(egui::RichText::new("1:1 Neutral").size(10.0)).clicked() {
                                profile.direction_multipliers = [1.0; NUM_DIRECTIONS];
                                profile.recompute_fourier();
                            }
                            if ui.button(egui::RichText::new("CS2/Val Tactical").size(10.0).color(colors::ACCENT_CYAN)).clicked() {
                                let p = Profile::preset_tactical_flick();
                                profile.direction_multipliers = p.direction_multipliers;
                                profile.recompute_fourier();
                            }
                            if ui.button(egui::RichText::new("Apex Tracking").size(10.0).color(colors::SUCCESS_GREEN)).clicked() {
                                let p = Profile::preset_tracking_smooth();
                                profile.direction_multipliers = p.direction_multipliers;
                                profile.recompute_fourier();
                            }
                        });

                        ui.add_space(4.0);

                        // Modern Strength Slider
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Assistance Intensity (α):").size(11.0).color(colors::TEXT_PRIMARY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let pct = (profile.strength * 100.0).round() as i32;
                                ui.label(
                                    egui::RichText::new(format!("{:>3}%", pct))
                                        .monospace()
                                        .size(11.0)
                                        .color(colors::ACCENT_CYAN),
                                );
                            });
                        });

                        let mut strength_val = profile.strength;
                        let slider_label = format!("{:.0}%", strength_val * 100.0);
                        if modern_slider(
                            ui,
                            &mut strength_val,
                            0.0..=1.5,
                            &slider_label,
                        ) {
                            profile.strength = strength_val;
                        }
                    });

                    ui.add_space(4.0);

                    // Manual 16-Axis Tuner
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("MANUAL 16-AXIS CALIBRATION").size(10.5).color(colors::TEXT_PRIMARY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("Reset").size(9.0)).clicked() {
                                    profile.direction_multipliers = [1.0; NUM_DIRECTIONS];
                                    profile.recompute_fourier();
                                }
                            });
                        });

                        ui.add_space(2.0);

                        egui::ScrollArea::vertical().max_height(115.0).show(ui, |ui| {
                            egui::Grid::new("directions_grid")
                                .num_columns(4)
                                .spacing([6.0, 3.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    for i in 0..NUM_DIRECTIONS {
                                        let angle_deg = i as f32 * 22.5;
                                        let is_selected = self.selected_dir_index == Some(i);

                                        let label = egui::RichText::new(format!("{:>5.1}°", angle_deg))
                                            .monospace()
                                            .size(9.5)
                                            .color(if is_selected {
                                                colors::ACCENT_CYAN
                                            } else {
                                                colors::TEXT_MUTED
                                            });

                                        if ui.selectable_label(is_selected, label).clicked() {
                                            self.selected_dir_index = if is_selected { None } else { Some(i) };
                                        }

                                        let mut mult = profile.direction_multipliers[i];
                                        let slider = egui::Slider::new(&mut mult, 0.6..=1.4).show_value(false);

                                        if ui.add(slider).changed() {
                                            profile.direction_multipliers[i] = mult;
                                            profile.recompute_fourier();
                                        }

                                        let delta_pct = (mult - 1.0) * 100.0;
                                        ui.label(
                                            egui::RichText::new(format!("{:>4.2}x ({:+3.0}%)", mult, delta_pct))
                                                .monospace()
                                                .size(9.0)
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

                    // Dot-Matrix Live Vector Simulator Canvas
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("LIVE VECTOR SIMULATOR").size(10.5).color(colors::TEXT_PRIMARY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("Clear").size(9.0)).clicked() {
                                    self.sandbox_trail.clear();
                                    self.sandbox_accumulator.reset();
                                    self.last_delta = (0, 0);
                                }
                                let (dx, dy) = self.last_delta;
                                ui.label(
                                    egui::RichText::new(format!("Δx: {:+3}, Δy: {:+3}", dx, dy))
                                        .monospace()
                                        .size(9.5)
                                        .color(colors::ACCENT_CYAN),
                                );
                            });
                        });

                        let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 80.0), Sense::drag());
                        let painter = ui.painter_at(rect);

                        // Background surface
                        painter.rect_filled(rect, CornerRadius::same(4), colors::CHASSIS_BG);
                        painter.rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0, colors::BORDER_SUBTLE), egui::StrokeKind::Inside);

                        // Dot Matrix Background Grid
                        let dot_spacing = 16.0;
                        let mut x = rect.left() + dot_spacing;
                        while x < rect.right() - 8.0 {
                            let mut y = rect.top() + dot_spacing;
                            while y < rect.bottom() - 8.0 {
                                painter.circle_filled(
                                    Pos2::new(x, y),
                                    1.0,
                                    Color32::from_rgba_premultiplied(0x1E, 0x22, 0x2E, 0x50),
                                );
                                y += dot_spacing;
                            }
                            x += dot_spacing;
                        }

                        let center = rect.center();

                        // Center reticle
                        painter.line_segment(
                            [Pos2::new(center.x - 6.0, center.y), Pos2::new(center.x + 6.0, center.y)],
                            Stroke::new(0.6, colors::BORDER_STRONG),
                        );
                        painter.line_segment(
                            [Pos2::new(center.x, center.y - 6.0), Pos2::new(center.x, center.y + 6.0)],
                            Stroke::new(0.6, colors::BORDER_STRONG),
                        );

                        // Drag handling
                        if response.dragged() {
                            let delta = response.drag_delta();
                            if delta.length_sq() > 0.0 {
                                let (corr_dx, corr_dy) = profile.fourier.correct_delta(delta.x, delta.y, profile.strength);
                                let (out_dx, out_dy) = self.sandbox_accumulator.apply(corr_dx, corr_dy);
                                self.last_delta = (out_dx, out_dy);

                                let new_pos = Pos2::new(
                                    (self.test_cursor.x + out_dx as f32).clamp(rect.left() + 6.0, rect.right() - 6.0),
                                    (self.test_cursor.y + out_dy as f32).clamp(rect.top() + 6.0, rect.bottom() - 6.0),
                                );

                                self.sandbox_trail.push((self.test_cursor, new_pos));
                                if self.sandbox_trail.len() > 120 {
                                    self.sandbox_trail.remove(0);
                                }
                                self.test_cursor = new_pos;
                            }
                        } else if !rect.contains(self.test_cursor) {
                            self.test_cursor = center;
                        }

                        // Render glowing particle trail
                        for (from, to) in &self.sandbox_trail {
                            painter.line_segment([*from, *to], Stroke::new(1.4, colors::ACCENT_CYAN));
                        }

                        // Current cursor dot with glowing halo
                        painter.circle_filled(
                            self.test_cursor,
                            6.0,
                            Color32::from_rgba_premultiplied(0xFF, 0xA0, 0x1E, 0x30),
                        );
                        painter.circle_filled(self.test_cursor, 3.5, colors::ALERT_AMBER);
                        painter.circle_stroke(self.test_cursor, 4.0, Stroke::new(1.0, colors::CHASSIS_BG));

                        if !response.dragged() && self.sandbox_trail.is_empty() {
                            painter.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "Drag cursor to test sub-pixel compensated vector trail",
                                egui::FontId::proportional(10.5),
                                colors::TEXT_FAINT,
                            );
                        }
                    });
                });
            });
        });
    }
}
