//! 360-degree Interactive Polar Radar Widget for Vectorists.
//!
//! Renders the baseline unit circle, discrete 16-direction raw biomechanics,
//! and continuous Fourier harmonic compensation curve with hover inspection.

use crate::core::math::TWO_PI;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2, Widget};

pub struct PolarRadarWidget<'a> {
    profile: &'a Profile,
    size: Vec2,
    selected_index: Option<usize>,
}

impl<'a> PolarRadarWidget<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        Self {
            profile,
            size: Vec2::splat(320.0),
            selected_index: None,
        }
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn selected_index(mut self, idx: Option<usize>) -> Self {
        self.selected_index = idx;
        self
    }
}

impl<'a> Widget for PolarRadarWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, mut response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter_at(rect);
        let center = rect.center();
        let max_radius = (rect.width().min(rect.height()) / 2.0) - 24.0;
        let base_radius = max_radius * 0.70; // 1.0 multiplier maps to 70% of radius

        // 1. Draw Radar Background & Concentric Reference Circles
        painter.circle_filled(center, max_radius + 12.0, colors::PANEL_BG);
        painter.circle_stroke(center, max_radius + 12.0, Stroke::new(1.0, colors::BORDER_SUBTLE));

        // Range rings: 0.5x, 0.75x, 1.0x (base), 1.25x
        let rings = [(0.5, "0.5x"), (0.75, "0.75x"), (1.0, "1.0x"), (1.25, "1.25x")];
        for (scale, label) in rings {
            let r = base_radius * scale;
            let stroke = if (scale - 1.0).abs() < 1e-4 {
                Stroke::new(1.5, colors::BORDER_STRONG) // Emphasize 1.0 baseline
            } else {
                Stroke::new(1.0, Color32::from_rgba_premultiplied(0x23, 0x27, 0x33, 0x80))
            };
            painter.circle_stroke(center, r, stroke);

            // Ring text label (along 90° vertical upward axis)
            let text_pos = Pos2::new(center.x + 4.0, center.y - r - 2.0);
            painter.text(
                text_pos,
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::monospace(9.0),
                colors::TEXT_FAINT,
            );
        }

        // 2. Draw 16 Radial Rays
        let step_angle = TWO_PI / (NUM_DIRECTIONS as f32);
        for i in 0..NUM_DIRECTIONS {
            let angle = (i as f32) * step_angle;
            let dir = Vec2::new(angle.cos(), angle.sin());
            let outer_pos = center + dir * (max_radius + 6.0);
            
            let is_cardinal = i % 4 == 0;
            let stroke = if is_cardinal {
                Stroke::new(1.0, colors::BORDER_STRONG)
            } else {
                Stroke::new(0.5, Color32::from_rgba_premultiplied(0x23, 0x27, 0x33, 0x60))
            };
            painter.line_segment([center, outer_pos], stroke);

            // Cardinal labels
            if is_cardinal {
                let cardinal_text = match i {
                    0 => "0° R",
                    4 => "90° U",
                    8 => "180° L",
                    12 => "270° D",
                    _ => "",
                };
                let label_pos = center + dir * (max_radius + 18.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    cardinal_text,
                    egui::FontId::proportional(10.0),
                    colors::TEXT_MUTED,
                );
            }
        }

        // 3. Draw Continuous Fourier Fitted Curve (Cyan polygon)
        const SAMPLES: usize = 120;
        let mut continuous_points = Vec::with_capacity(SAMPLES);
        for s in 0..SAMPLES {
            let theta = (s as f32 / SAMPLES as f32) * TWO_PI;
            // Screen orientation: +Y in screen is downward, so standard math (sin) maps downward.
            // When moving mouse up, dy is negative. We align standard Cartesian angles.
            let raw_multiplier = self.profile.fourier.eval(theta);
            let effective_multiplier = 1.0 + self.profile.strength * (raw_multiplier - 1.0);
            let r = (base_radius * effective_multiplier).clamp(base_radius * 0.2, max_radius + 10.0);
            let pt = center + Vec2::new(theta.cos(), theta.sin()) * r;
            continuous_points.push(pt);
        }

        if continuous_points.len() >= 3 {
            // Semi-transparent area
            painter.add(egui::Shape::convex_polygon(
                continuous_points.clone(),
                Color32::from_rgba_premultiplied(0x00, 0xE5, 0xFF, 0x18),
                Stroke::new(2.0, colors::ACCENT_CYAN),
            ));
        }

        // 4. Draw Discrete Measured Multiplier Nodes (Amber dots)
        for i in 0..NUM_DIRECTIONS {
            let angle = (i as f32) * step_angle;
            let raw_mult = self.profile.direction_multipliers[i];
            let effective_mult = 1.0 + self.profile.strength * (raw_mult - 1.0);
            let r = (base_radius * effective_mult).clamp(base_radius * 0.2, max_radius + 10.0);
            let pt = center + Vec2::new(angle.cos(), angle.sin()) * r;

            let is_selected = self.selected_index == Some(i);
            let node_color = if is_selected {
                colors::ACCENT_CYAN
            } else {
                colors::ALERT_AMBER
            };

            let node_radius = if is_selected { 5.0 } else { 3.5 };
            painter.circle_filled(pt, node_radius, node_color);
            painter.circle_stroke(pt, node_radius + 1.0, Stroke::new(1.0, colors::PANEL_BG));
        }

        // Center origin dot
        painter.circle_filled(center, 3.0, colors::TEXT_MUTED);

        // 5. Hover and Angle Inspection
        if let Some(hover_pos) = response.hover_pos() {
            let diff = hover_pos - center;
            let dist = diff.length();
            if dist <= max_radius + 20.0 {
                let mut hover_angle = diff.y.atan2(diff.x);
                if hover_angle < 0.0 {
                    hover_angle += TWO_PI;
                }
                let hover_deg = hover_angle.to_degrees();

                // Draw inspection cursor ray
                let cursor_edge = center + diff.normalized() * max_radius;
                painter.line_segment([center, cursor_edge], Stroke::new(1.0, colors::ACCENT_CYAN));

                let val = self.profile.fourier.eval(hover_angle);
                let eff_val = 1.0 + self.profile.strength * (val - 1.0);
                let delta_pct = (eff_val - 1.0) * 100.0;

                response = response.on_hover_ui_at_pointer(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Ángulo: {:>5.1}°", hover_deg))
                            .monospace()
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "Multiplicador: {:>4.2}x ({:+5.1}%)",
                            eff_val, delta_pct
                        ))
                        .monospace()
                        .color(if delta_pct >= 0.0 {
                            colors::ACCENT_CYAN
                        } else {
                            colors::ALERT_AMBER
                        }),
                    );
                });
            }
        }

        response
    }
}
