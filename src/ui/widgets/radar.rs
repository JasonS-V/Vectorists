//! Polar response plot in sensor coordinates: positive Y points down.
use crate::core::math::TWO_PI;
use crate::core::profile::{NUM_DIRECTIONS, Profile};
use crate::ui::theme::colors;
use eframe::egui::{self, Color32, Sense, Stroke, Vec2, Widget};

pub struct PolarRadarWidget<'a> {
    profile: &'a Profile,
    size: Vec2,
    selected_index: Option<usize>,
}

pub fn direction_index(delta: Vec2) -> usize {
    ((delta.y.atan2(delta.x).rem_euclid(TWO_PI) / TWO_PI * NUM_DIRECTIONS as f32).round() as usize)
        % NUM_DIRECTIONS
}

impl<'a> PolarRadarWidget<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        Self {
            profile,
            size: Vec2::splat(360.0),
            selected_index: None,
        }
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }
}

impl Widget for PolarRadarWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());
        response.widget_info(|| egui::WidgetInfo::labeled(egui::accesskit::Role::Image, true,
            "Mapa de respuesta polar. Los controles de dirección permiten ajustar cada ángulo con teclado."));
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.5 - 42.0;
        let strength = if self.profile.is_active {
            self.profile.strength
        } else {
            0.0
        };
        let points: Vec<_> = (0..192)
            .map(|i| {
                let angle = i as f32 / 192.0 * TWO_PI;
                (
                    angle,
                    self.profile.fourier.evaluate_with_strength(angle, strength),
                )
            })
            .collect();
        let extent = points
            .iter()
            .map(|(_, m)| *m)
            .chain(self.profile.direction_multipliers.iter().copied())
            .fold(1.5_f32, f32::max)
            .max(1.5)
            + 0.06;
        let unit = radius / extent;
        painter.circle_filled(center, radius + 12.0, colors::SIDEBAR_BG);
        for i in 0..64 {
            let a = i as f32 / 64.0 * TWO_PI;
            let dir = Vec2::angled(a);
            painter.line_segment(
                [
                    center + dir * (radius + 7.0),
                    center + dir * (radius + if i % 4 == 0 { 12.0 } else { 10.0 }),
                ],
                Stroke::new(1.0, colors::BORDER_STRONG),
            );
        }
        for multiplier in [0.5_f32, 1.0, 1.5] {
            painter.circle_stroke(
                center,
                unit * multiplier,
                Stroke::new(
                    1.0,
                    if multiplier == 1.0 {
                        colors::BORDER_STRONG
                    } else {
                        colors::BORDER_SUBTLE
                    },
                ),
            );
            painter.text(
                center + egui::vec2(8.0, -unit * multiplier - 3.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{multiplier:.1}×"),
                egui::FontId::monospace(10.0),
                colors::TEXT_FAINT,
            );
        }
        let labels = ["0°", "45°", "90°", "135°", "180°", "225°", "270°", "315°"];
        for i in 0..NUM_DIRECTIONS {
            let a = i as f32 / NUM_DIRECTIONS as f32 * TWO_PI;
            let dir = Vec2::angled(a);
            painter.line_segment(
                [center + dir * 8.0, center + dir * radius],
                Stroke::new(0.6, colors::BORDER_SUBTLE),
            );
            if i % 2 == 0 {
                painter.text(
                    center + dir * (radius + 28.0),
                    egui::Align2::CENTER_CENTER,
                    labels[i / 2],
                    egui::FontId::monospace(11.0),
                    if i % 4 == 0 {
                        colors::TEXT_MUTED
                    } else {
                        colors::TEXT_FAINT
                    },
                );
            }
        }
        let curve: Vec<_> = points
            .iter()
            .map(|(a, m)| center + Vec2::angled(*a) * unit * *m)
            .collect();
        // A triangle fan supports concave radial curves as well as the neutral circle.
        let mut mesh = egui::Mesh::default();
        let fill = Color32::from_rgba_unmultiplied(89, 223, 211, 15);
        mesh.colored_vertex(center, fill);
        for point in &curve {
            mesh.colored_vertex(*point, fill);
        }
        for i in 0..curve.len() {
            mesh.add_triangle(0, i as u32 + 1, ((i + 1) % curve.len()) as u32 + 1);
        }
        painter.add(egui::Shape::mesh(mesh));
        painter.add(egui::Shape::closed_line(
            curve,
            Stroke::new(2.0, colors::ACCENT_CYAN),
        ));
        let raw: Vec<_> = self
            .profile
            .direction_multipliers
            .iter()
            .enumerate()
            .map(|(i, m)| {
                center + Vec2::angled(i as f32 / NUM_DIRECTIONS as f32 * TWO_PI) * unit * *m
            })
            .collect();
        painter.add(egui::Shape::closed_line(
            raw.clone(),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(239, 184, 104, 90)),
        ));
        for (i, point) in raw.iter().enumerate() {
            let selected = self.selected_index == Some(i);
            if selected {
                painter.line_segment([center, *point], Stroke::new(1.0, colors::ACCENT_CYAN));
                painter.circle_stroke(*point, 8.0, Stroke::new(1.0, colors::ACCENT_CYAN));
            }
            painter.circle_filled(
                *point,
                if selected { 4.0 } else { 2.5 },
                colors::ALERT_AMBER,
            );
        }
        painter.circle_filled(center, 3.0, colors::TEXT_PRIMARY);
        if let Some(position) = response.hover_pos() {
            let delta = position - center;
            if delta.length() > 10.0 && delta.length() < radius + 16.0 {
                let angle = delta.y.atan2(delta.x).rem_euclid(TWO_PI);
                painter.line_segment(
                    [center, center + delta.normalized() * radius],
                    Stroke::new(1.0, colors::ACCENT_CYAN),
                );
                return response.on_hover_ui_at_pointer(|ui| {
                    ui.strong(format!("Dirección {:.1}°", angle.to_degrees()));
                    ui.label(format!(
                        "Respuesta efectiva: {:.3}×",
                        self.profile.fourier.evaluate_with_strength(angle, strength)
                    ));
                    ui.small("Haz clic para seleccionar esta dirección.");
                });
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plot_selection_matches_windows_sensor_axes() {
        assert_eq!(direction_index(egui::vec2(1.0, 0.0)), 0);
        assert_eq!(direction_index(egui::vec2(0.0, 1.0)), 4);
        assert_eq!(direction_index(egui::vec2(-1.0, 0.0)), 8);
        assert_eq!(direction_index(egui::vec2(0.0, -1.0)), 12);
        assert_eq!(direction_index(egui::vec2(1.0, -0.001)), 0);
    }
}
