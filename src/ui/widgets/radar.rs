//! 360-degree Interactive Tactical Polar Radar Widget for Vectorists.
//!
//! Renders concentric telemetry range rings, continuous glowing Fourier curve,
//! raw biomechanical nodes, and interactive hover HUD inspection.

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
        let max_radius = (rect.width().min(rect.height()) / 2.0) - 26.0;
        let base_radius = max_radius * 0.70; // 1.0 multiplier maps to 70% of radius

        // 1. Radar Bezel & Background
        painter.circle_filled(center, max_radius + 14.0, colors::CHASSIS_BG);
        painter.circle_stroke(center, max_radius + 14.0, Stroke::new(1.0, colors::BORDER_SUBTLE));
        painter.circle_stroke(center, max_radius + 8.0, Stroke::new(0.5, Color32::from_rgba_premultiplied(0x1E, 0x22, 0x2E, 0x80)));

        // 2. Concentric Telemetry Range Rings
        let rings = [
            (0.50, "0.5x"),
            (0.75, "0.75x"),
            (1.00, "1.00x BASE"),
            (1.25, "1.25x"),
        ];

        for (scale, label) in rings {
            let r = base_radius * scale;
            let is_base = (scale - 1.00).abs() < 1e-4;

            if is_base {
                // Dual glow stroke on baseline unit circle
                painter.circle_stroke(
                    center,
                    r,
                    Stroke::new(3.0, Color32::from_rgba_premultiplied(0x00, 0xF0, 0xFF, 0x18)),
                );
                painter.circle_stroke(center, r, Stroke::new(1.2, colors::BORDER_STRONG));
            } else {
                painter.circle_stroke(
                    center,
                    r,
                    Stroke::new(0.6, Color32::from_rgba_premultiplied(0x1E, 0x22, 0x2E, 0x80)),
                );
            }

            // Ring label placed along the upward axis
            let text_pos = Pos2::new(center.x + 5.0, center.y - r - 2.0);
            painter.text(
                text_pos,
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::monospace(8.5),
                if is_base { colors::ACCENT_CYAN } else { colors::TEXT_FAINT },
            );
        }

        // 3. 16 Radial Rays & 8 Compass Direction Markers
        let step_angle = TWO_PI / (NUM_DIRECTIONS as f32);
        for i in 0..NUM_DIRECTIONS {
            let angle = (i as f32) * step_angle;
            let dir = Vec2::new(angle.cos(), angle.sin());
            let outer_pos = center + dir * (max_radius + 8.0);

            let is_cardinal = i % 4 == 0;
            let is_intercardinal = i % 2 == 0;

            let stroke = if is_cardinal {
                Stroke::new(1.0, colors::BORDER_STRONG)
            } else if is_intercardinal {
                Stroke::new(0.6, Color32::from_rgba_premultiplied(0x1E, 0x22, 0x2E, 0x90))
            } else {
                Stroke::new(0.4, Color32::from_rgba_premultiplied(0x1E, 0x22, 0x2E, 0x40))
            };
            painter.line_segment([center, outer_pos], stroke);

            // Compass Markers
            if is_intercardinal {
                let compass_text = match i {
                    0 => "0° E",
                    2 => "45° NE",
                    4 => "90° N",
                    6 => "135° NW",
                    8 => "180° W",
                    10 => "225° SW",
                    12 => "270° S",
                    14 => "315° SE",
                    _ => "",
                };
                let label_pos = center + dir * (max_radius + 18.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    compass_text,
                    egui::FontId::proportional(9.0),
                    if is_cardinal { colors::TEXT_PRIMARY } else { colors::TEXT_FAINT },
                );
            }
        }

        // 4. Raw Biomechanical Measured Polygon (Amber dashed/connected outline)
        let mut raw_points = Vec::with_capacity(NUM_DIRECTIONS);
        for i in 0..NUM_DIRECTIONS {
            let angle = (i as f32) * step_angle;
            let raw_mult = self.profile.direction_multipliers[i];
            let r = (base_radius * raw_mult).clamp(base_radius * 0.25, max_radius + 12.0);
            raw_points.push(center + Vec2::new(angle.cos(), angle.sin()) * r);
        }

        if raw_points.len() >= 3 {
            for i in 0..NUM_DIRECTIONS {
                let next = (i + 1) % NUM_DIRECTIONS;
                painter.line_segment(
                    [raw_points[i], raw_points[next]],
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(0xFF, 0xA0, 0x1E, 0x70)),
                );
            }
        }

        // 5. Continuous Fourier Curve (Multi-layered Glowing Cyan Polygon)
        const SAMPLES: usize = 128;
        let mut continuous_points = Vec::with_capacity(SAMPLES);
        for s in 0..SAMPLES {
            let theta = (s as f32 / SAMPLES as f32) * TWO_PI;
            let raw_multiplier = self.profile.fourier.eval(theta);
            let effective_multiplier = 1.0 + self.profile.strength * (raw_multiplier - 1.0);
            let r = (base_radius * effective_multiplier).clamp(base_radius * 0.25, max_radius + 12.0);
            let pt = center + Vec2::new(theta.cos(), theta.sin()) * r;
            continuous_points.push(pt);
        }

        if continuous_points.len() >= 3 {
            // Layer A: Soft ambient fill
            painter.add(egui::Shape::convex_polygon(
                continuous_points.clone(),
                Color32::from_rgba_premultiplied(0x00, 0xF0, 0xFF, 0x16),
                Stroke::NONE,
            ));

            // Layer B: Outer neon glow stroke
            for i in 0..SAMPLES {
                let next = (i + 1) % SAMPLES;
                painter.line_segment(
                    [continuous_points[i], continuous_points[next]],
                    Stroke::new(3.5, Color32::from_rgba_premultiplied(0x00, 0xF0, 0xFF, 0x30)),
                );
            }

            // Layer C: Sharp crisp vector line
            for i in 0..SAMPLES {
                let next = (i + 1) % SAMPLES;
                painter.line_segment(
                    [continuous_points[i], continuous_points[next]],
                    Stroke::new(1.8, colors::ACCENT_CYAN),
                );
            }
        }

        // 6. Discrete Measured Multiplier Nodes (Interactive Amber Dots with halo)
        for i in 0..NUM_DIRECTIONS {
            let pt = raw_points[i];
            let is_selected = self.selected_index == Some(i);

            let node_radius = if is_selected { 5.5 } else { 3.5 };
            let node_color = if is_selected {
                colors::ACCENT_CYAN
            } else {
                colors::ALERT_AMBER
            };

            // Glow / halo for selected node
            if is_selected {
                painter.circle_stroke(pt, node_radius + 3.0, Stroke::new(1.2, colors::ACCENT_CYAN_GLOW));
            }

            painter.circle_filled(pt, node_radius, node_color);
            painter.circle_stroke(pt, node_radius + 1.0, Stroke::new(1.0, colors::PANEL_BG));
        }

        // 7. Tactical Center Reticle
        painter.circle_filled(center, 3.5, colors::TEXT_PRIMARY);
        painter.circle_stroke(center, 6.0, Stroke::new(1.0, colors::BORDER_STRONG));
        painter.line_segment(
            [Pos2::new(center.x - 8.0, center.y), Pos2::new(center.x + 8.0, center.y)],
            Stroke::new(0.8, colors::ACCENT_CYAN),
        );
        painter.line_segment(
            [Pos2::new(center.x, center.y - 8.0), Pos2::new(center.x, center.y + 8.0)],
            Stroke::new(0.8, colors::ACCENT_CYAN),
        );

        // 8. Interactive Hover Ray & Floating HUD Card
        if let Some(hover_pos) = response.hover_pos() {
            let diff = hover_pos - center;
            let dist = diff.length();
            if dist <= max_radius + 24.0 {
                let mut hover_angle = diff.y.atan2(diff.x);
                if hover_angle < 0.0 {
                    hover_angle += TWO_PI;
                }
                let hover_deg = hover_angle.to_degrees();

                // Draw laser ray towards cursor
                let cursor_edge = center + diff.normalized() * (max_radius + 10.0);
                painter.line_segment(
                    [center, cursor_edge],
                    Stroke::new(1.2, colors::ACCENT_CYAN_HOVER),
                );

                let raw_val = self.profile.fourier.eval(hover_angle);
                let eff_val = 1.0 + self.profile.strength * (raw_val - 1.0);
                let delta_pct = (eff_val - 1.0) * 100.0;

                response = response.on_hover_ui_at_pointer(|ui| {
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new(format!("BEARING: {:>5.1}°", hover_deg))
                                .monospace()
                                .strong()
                                .size(11.0)
                                .color(colors::TEXT_PRIMARY),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Effective Scale: {:>4.2}x ({:+5.1}%)", eff_val, delta_pct))
                                .monospace()
                                .size(11.0)
                                .color(if delta_pct >= 0.0 {
                                    colors::ACCENT_CYAN
                                } else {
                                    colors::ALERT_AMBER
                                }),
                        );
                        ui.label(
                            egui::RichText::new(format!("Raw Measured:    {:>4.2}x", raw_val))
                                .monospace()
                                .size(10.0)
                                .color(colors::TEXT_MUTED),
                        );
                    });
                });
            }
        }

        response
    }
}
