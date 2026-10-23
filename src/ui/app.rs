//! Main Vectorists eframe/egui Application.

use crate::core::profile::ProfileManager;
use crate::ui::theme::{apply_theme, colors};
use crate::ui::views::{CalibratorView, DashboardView, SettingsView};
use eframe::egui::{self, Frame, Stroke};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Dashboard,
    Calibrator,
    Settings,
}

pub struct VectoristsApp {
    pub active_tab: ActiveTab,
    pub profile_manager: ProfileManager,
    pub dashboard_view: DashboardView,
    pub calibrator_view: CalibratorView,
    pub settings_view: SettingsView,
}

impl Default for VectoristsApp {
    fn default() -> Self {
        let profile_manager = ProfileManager::load_or_default();
        Self {
            active_tab: ActiveTab::Dashboard,
            profile_manager,
            dashboard_view: DashboardView::default(),
            calibrator_view: CalibratorView::default(),
            settings_view: SettingsView::default(),
        }
    }
}

impl VectoristsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);
        Self::default()
    }
}

impl eframe::App for VectoristsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Root container with dark chassis fill and padding
        ui.vertical(|ui| {
            // 1. Top Navigation Bar
            Frame::new()
                .fill(colors::CHASSIS_BG)
                .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // App Logo & Branding
                        ui.label(
                            egui::RichText::new("VECTORISTS")
                                .size(14.0)
                                .strong()
                                .color(colors::TEXT_PRIMARY),
                        );

                        ui.label(
                            egui::RichText::new("v0.1.0")
                                .size(10.0)
                                .monospace()
                                .color(colors::TEXT_FAINT),
                        );

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // Tab Navigation Buttons
                        let tabs = [
                            (ActiveTab::Dashboard, "✦ MAPA POLAR"),
                            (ActiveTab::Calibrator, "▶ CALIBRADOR"),
                            (ActiveTab::Settings, "⚙ PERFILES & AJUSTES"),
                        ];

                        for (tab, label) in tabs {
                            let is_active = self.active_tab == tab;
                            let text = egui::RichText::new(label)
                                .size(11.0)
                                .color(if is_active { colors::ACCENT_CYAN } else { colors::TEXT_MUTED });

                            if ui.selectable_label(is_active, text).clicked() {
                                self.active_tab = tab;
                            }
                        }

                        // Right-aligned active profile status & toggle
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let active_profile = self.profile_manager.active_profile_mut();
                            let toggle_label = if active_profile.is_active {
                                egui::RichText::new("CORRECCIÓN: ON").color(colors::SUCCESS_GREEN).size(11.0)
                            } else {
                                egui::RichText::new("CORRECCIÓN: OFF").color(colors::ALERT_AMBER).size(11.0)
                            };

                            if ui.button(toggle_label).clicked() {
                                active_profile.is_active = !active_profile.is_active;
                            }

                            ui.label(
                                egui::RichText::new(format!("Perfil: {}", active_profile.name))
                                    .size(11.0)
                                    .color(colors::TEXT_MUTED),
                            );
                        });
                    });
                });

            ui.add_space(4.0);

            // 2. Central Content Area
            let content_height = ui.available_height() - 32.0;
            egui::ScrollArea::both()
                .max_height(content_height)
                .show(ui, |ui| {
                    let active_profile = self.profile_manager.active_profile_mut();
                    match self.active_tab {
                        ActiveTab::Dashboard => {
                            self.dashboard_view.ui(ui, active_profile);
                        }
                        ActiveTab::Calibrator => {
                            self.calibrator_view.ui(ui, active_profile);
                        }
                        ActiveTab::Settings => {
                            self.settings_view.ui(ui, &mut self.profile_manager);
                        }
                    }
                });

            ui.add_space(4.0);

            // 3. Bottom Status Bar
            Frame::new()
                .fill(colors::CHASSIS_BG)
                .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                .inner_margin(egui::Margin::symmetric(14, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let active = self.profile_manager.active_profile();
                        ui.label(
                            egui::RichText::new(format!(
                                "Sensibilidad: {} DPI | Polling: {} Hz | Fuerza: {:.0}%",
                                active.dpi,
                                active.polling_rate_hz,
                                active.strength * 100.0
                            ))
                            .monospace()
                            .size(10.0)
                            .color(colors::TEXT_FAINT),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("Latencia: < 0.02 ms | RAM: ~12 MB")
                                    .monospace()
                                    .size(10.0)
                                    .color(colors::TEXT_FAINT),
                            );
                        });
                    });
                });
        });
    }
}
