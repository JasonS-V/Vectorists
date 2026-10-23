//! Main Vectorists eframe/egui Application.

use crate::core::profile::ProfileManager;
use crate::input::RawInputListener;
use crate::ui::theme::{apply_theme, colors};
use crate::ui::views::{CalibratorView, DashboardView, SettingsView};
use crate::ui::widgets::{segmented_pill_tab, toggle_switch};
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
    pub raw_listener: Option<RawInputListener>,
}

impl Default for VectoristsApp {
    fn default() -> Self {
        let profile_manager = ProfileManager::load_or_default();
        let raw_listener = Some(RawInputListener::start());
        Self {
            active_tab: ActiveTab::Dashboard,
            profile_manager,
            dashboard_view: DashboardView::default(),
            calibrator_view: CalibratorView::default(),
            settings_view: SettingsView::default(),
            raw_listener,
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
        let telemetry = self.raw_listener.as_ref().map(|l| l.telemetry()).unwrap_or_default();

        ui.vertical(|ui| {
            // 1. Top Modern Floating Dock / Header
            Frame::new()
                .fill(colors::CHASSIS_BG)
                .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Branding with glowing indicator
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("VECTORISTS")
                                    .size(14.0)
                                    .strong()
                                    .color(colors::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new("v0.1.0")
                                    .size(9.5)
                                    .monospace()
                                    .color(colors::ACCENT_CYAN),
                            );
                        });

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(16.0);

                        // Modern Segmented Pill Dock
                        let tabs = [
                            (ActiveTab::Dashboard, "✦  POLAR MAP"),
                            (ActiveTab::Calibrator, "🎯  CALIBRATOR"),
                            (ActiveTab::Settings, "⚙  SETTINGS"),
                        ];

                        for (tab, label) in tabs {
                            let is_active = self.active_tab == tab;
                            if segmented_pill_tab(ui, label, is_active) {
                                self.active_tab = tab;
                            }
                        }

                        // Right-aligned Modern Toggle Switch & Active Profile Name
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let active_profile = self.profile_manager.active_profile_mut();
                            let is_on = active_profile.is_active;
                            let switch_label = if is_on { "CORRECTION" } else { "BYPASS" };

                            toggle_switch(
                                ui,
                                &mut active_profile.is_active,
                                switch_label,
                            );

                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(12.0);

                            ui.label(
                                egui::RichText::new(&active_profile.name)
                                    .size(11.0)
                                    .color(colors::TEXT_MUTED),
                            );
                            ui.label(
                                egui::RichText::new("PROFILE:")
                                    .size(9.5)
                                    .monospace()
                                    .color(colors::TEXT_FAINT),
                            );
                        });
                    });
                });

            ui.add_space(6.0);

            // 2. Central Content Area
            let content_height = ui.available_height() - 36.0;
            egui::ScrollArea::both()
                .max_height(content_height)
                .show(ui, |ui| {
                    let active_profile = self.profile_manager.active_profile_mut();
                    match self.active_tab {
                        ActiveTab::Dashboard => {
                            self.dashboard_view.ui(
                                ui,
                                active_profile,
                                telemetry.polling_rate_hz,
                                telemetry.counts_per_sec,
                            );
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

            // 3. Modern Telemetry Status Bar
            Frame::new()
                .fill(colors::CHASSIS_BG)
                .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                .inner_margin(egui::Margin::symmetric(14, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let active = self.profile_manager.active_profile();

                        // Sensor status pill
                        ui.label(
                            egui::RichText::new(format!("SENS: {} DPI | {} Hz", active.dpi, active.polling_rate_hz))
                                .monospace()
                                .size(9.5)
                                .color(colors::TEXT_FAINT),
                        );

                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("•").size(9.5).color(colors::BORDER_STRONG));
                        ui.add_space(8.0);

                        let hw_text = if telemetry.polling_rate_hz > 0 {
                            format!("LIVE: {} Hz ({:.0} CPS)", telemetry.polling_rate_hz, telemetry.counts_per_sec)
                        } else {
                            "LIVE: IDLE (Move mouse to stream)".to_string()
                        };

                        ui.label(
                            egui::RichText::new(hw_text)
                                .monospace()
                                .size(9.5)
                                .color(colors::ACCENT_CYAN),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("LATENCY: < 0.02ms | RAM: ~12 MB | PURE RUST")
                                    .monospace()
                                    .size(9.5)
                                    .color(colors::TEXT_FAINT),
                            );
                        });
                    });
                });

            ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
        });
    }
}
