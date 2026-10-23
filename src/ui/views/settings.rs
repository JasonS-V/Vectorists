//! Settings, Profile Management, and RawAccel Interop View for Vectorists.

use crate::core::profile::{Profile, ProfileManager};
use crate::ui::theme::colors;
use crate::ui::widgets::{card_container, pill_chip};
use eframe::egui;

pub struct SettingsView {
    pub new_profile_name: String,
    pub status_message: Option<(String, bool)>, // (Message, is_error)
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            new_profile_name: "New Profile".to_string(),
            status_message: None,
        }
    }
}

impl SettingsView {
    pub fn ui(&mut self, ui: &mut egui::Ui, manager: &mut ProfileManager) {
        ui.vertical(|ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("PROFILE CONFIGURATION & EXPORT")
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Save Profiles to Disk").color(colors::ACCENT_CYAN)).clicked() {
                        match manager.save_to_default_file() {
                            Ok(_) => {
                                self.status_message = Some(("Profiles saved to disk successfully.".to_string(), false));
                            }
                            Err(e) => {
                                self.status_message = Some((format!("Failed to save profiles: {}", e), true));
                            }
                        }
                    }
                });
            });

            ui.separator();

            if let Some((ref msg, is_err)) = self.status_message {
                let col = if is_err { colors::ALERT_AMBER } else { colors::SUCCESS_GREEN };
                ui.label(egui::RichText::new(msg).color(col).size(11.0));
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                // Left Column: Profile Selector & Presets
                ui.vertical(|ui| {
                    card_container(ui, |ui| {
                        ui.set_width(280.0);
                        ui.label(egui::RichText::new("AVAILABLE PROFILES").size(10.5).strong().color(colors::TEXT_PRIMARY));
                        ui.separator();

                        let mut switch_to_id = None;
                        let active_id = manager.active_profile().id.clone();

                        egui::ScrollArea::vertical().max_height(190.0).show(ui, |ui| {
                            for p in manager.profiles() {
                                let is_active = p.id == active_id;
                                let label_text = format!(
                                    "{} {}",
                                    if is_active { "●" } else { "○" },
                                    p.name
                                );

                                let text = egui::RichText::new(label_text)
                                    .color(if is_active { colors::ACCENT_CYAN } else { colors::TEXT_PRIMARY })
                                    .size(11.5);

                                if ui.selectable_label(is_active, text).clicked() {
                                    switch_to_id = Some(p.id.clone());
                                }
                            }
                        });

                        if let Some(id) = switch_to_id {
                            manager.set_active_profile(&id);
                        }

                        ui.separator();

                        // Add new profile
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.new_profile_name);
                            if pill_chip(ui, "+ Add", false, colors::ACCENT_CYAN) {
                                if !self.new_profile_name.trim().is_empty() {
                                    let new_id = format!("custom-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                                    let mut p = Profile::default();
                                    p.id = new_id.clone();
                                    p.name = self.new_profile_name.trim().to_string();
                                    manager.add_profile(p);
                                    manager.set_active_profile(&new_id);
                                    self.status_message = Some((format!("Profile '{}' created.", self.new_profile_name), false));
                                    self.new_profile_name = "New Profile".to_string();
                                }
                            }
                        });

                        ui.add_space(6.0);

                        // Presets buttons
                        ui.label(egui::RichText::new("Factory Presets:").size(10.0).color(colors::TEXT_MUTED));
                        ui.horizontal(|ui| {
                            if pill_chip(ui, "+ Tactical Flick", false, colors::ACCENT_CYAN) {
                                let p = Profile::preset_tactical_flick();
                                manager.add_profile(p.clone());
                                manager.set_active_profile(&p.id);
                                self.status_message = Some(("CS2/Valorant Tactical preset loaded.".to_string(), false));
                            }
                            if pill_chip(ui, "+ Apex Smooth", false, colors::SUCCESS_GREEN) {
                                let p = Profile::preset_tracking_smooth();
                                manager.add_profile(p.clone());
                                manager.set_active_profile(&p.id);
                                self.status_message = Some(("Apex Legends Tracking preset loaded.".to_string(), false));
                            }
                        });
                    });
                });

                ui.add_space(8.0);

                // Right Column: Active Profile Details & Export Options
                ui.vertical(|ui| {
                    let active = manager.active_profile_mut();

                    card_container(ui, |ui| {
                        ui.label(egui::RichText::new("HARDWARE SENSOR PARAMETERS").size(10.5).strong().color(colors::TEXT_PRIMARY));
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Profile Name:").size(11.0).color(colors::TEXT_MUTED));
                            ui.text_edit_singleline(&mut active.name);
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Sensor Base DPI:").size(11.0).color(colors::TEXT_MUTED));
                            let dpis = [400, 800, 1600, 3200];
                            for d in dpis {
                                if pill_chip(ui, &format!("{}", d), active.dpi == d, colors::ACCENT_CYAN) {
                                    active.dpi = d;
                                }
                            }
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Polling Frequency:").size(11.0).color(colors::TEXT_MUTED));
                            let rates = [1000, 2000, 4000, 8000];
                            for r in rates {
                                if pill_chip(ui, &format!("{} Hz", r), active.polling_rate_hz == r, colors::ACCENT_CYAN) {
                                    active.polling_rate_hz = r;
                                }
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Driver Interop & Export
                    card_container(ui, |ui| {
                        ui.label(egui::RichText::new("DRIVER INTEROP & HARDWARE EXPORT").size(10.5).strong().color(colors::TEXT_PRIMARY));
                        ui.separator();

                        ui.label(
                            egui::RichText::new(
                                "Generate ready-to-load configuration files for kernel-level mouse drivers (RawAccel) or hardware passthrough USB dongles (RP2040)."
                            )
                            .size(10.0)
                            .color(colors::TEXT_MUTED),
                        );

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            if pill_chip(ui, "Export to RawAccel (.json)", false, colors::ACCENT_CYAN) {
                                match active.export_rawaccel_json() {
                                    Ok(json_str) => {
                                        let filename = format!("rawaccel_{}.json", active.id);
                                        match std::fs::write(&filename, json_str) {
                                            Ok(_) => {
                                                self.status_message = Some((format!("RawAccel configuration exported as '{}'.", filename), false));
                                            }
                                            Err(e) => {
                                                self.status_message = Some((format!("Failed to write file: {}", e), true));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.status_message = Some((format!("JSON serialization failed: {}", e), true));
                                    }
                                }
                            }

                            if pill_chip(ui, "Export 360° LUT Table (.csv)", false, colors::TEXT_PRIMARY) {
                                let csv_str = active.export_polar_lut_csv();
                                let filename = format!("lut_{}.csv", active.id);
                                match std::fs::write(&filename, csv_str) {
                                    Ok(_) => {
                                        self.status_message = Some((format!("360-degree LUT table exported as '{}'.", filename), false));
                                    }
                                    Err(e) => {
                                        self.status_message = Some((format!("Failed to write CSV: {}", e), true));
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Engine Metadata & Credits Card
                    card_container(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Vectorists v0.1.0").monospace().size(10.0).color(colors::TEXT_MUTED));
                            ui.label(egui::RichText::new("•").size(10.0).color(colors::BORDER_STRONG));
                            ui.label(egui::RichText::new("Developed by Aizendev").size(10.0).color(colors::TEXT_MUTED));
                            ui.label(egui::RichText::new("•").size(10.0).color(colors::BORDER_STRONG));
                            ui.label(egui::RichText::new("Engine: Pure Rust + eframe/wgpu").size(10.0).color(colors::ACCENT_CYAN));
                        });
                    });
                });
            });
        });
    }
}
