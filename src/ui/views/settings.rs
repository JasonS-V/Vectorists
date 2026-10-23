//! Settings, Profile Management, and RawAccel Interop View for Vectorists.

use crate::core::profile::{Profile, ProfileManager};
use crate::ui::theme::colors;
use eframe::egui;

pub struct SettingsView {
    pub new_profile_name: String,
    pub status_message: Option<(String, bool)>, // (Message, is_error)
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            new_profile_name: "Nuevo Perfil".to_string(),
            status_message: None,
        }
    }
}

impl SettingsView {
    pub fn ui(&mut self, ui: &mut egui::Ui, manager: &mut ProfileManager) {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("GESTIÓN DE PERFILES Y EXPORTACIÓN")
                        .size(14.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Guardar Perfiles en Disco").color(colors::ACCENT_CYAN)).clicked() {
                        match manager.save_to_default_file() {
                            Ok(_) => {
                                self.status_message = Some(("Perfiles guardados correctamente en disco.".to_string(), false));
                            }
                            Err(e) => {
                                self.status_message = Some((format!("Error al guardar: {}", e), true));
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
                // Left Column: Profile List
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.set_width(280.0);
                        ui.label(egui::RichText::new("PERFILES DISPONIBLES").size(11.0).color(colors::TEXT_PRIMARY));
                        ui.separator();

                        let mut switch_to_id = None;
                        let active_id = manager.active_profile().id.clone();

                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for p in manager.profiles() {
                                let is_active = p.id == active_id;
                                let label_text = format!(
                                    "{} {}",
                                    if is_active { "●" } else { "○" },
                                    p.name
                                );

                                let text = egui::RichText::new(label_text)
                                    .color(if is_active { colors::ACCENT_CYAN } else { colors::TEXT_PRIMARY })
                                    .size(12.0);

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
                            if ui.button(egui::RichText::new("Crear").size(10.0)).clicked() {
                                if !self.new_profile_name.trim().is_empty() {
                                    let new_id = format!("custom-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                                    let mut p = Profile::default();
                                    p.id = new_id.clone();
                                    p.name = self.new_profile_name.trim().to_string();
                                    manager.add_profile(p);
                                    manager.set_active_profile(&new_id);
                                    self.status_message = Some((format!("Perfil '{}' creado.", self.new_profile_name), false));
                                    self.new_profile_name = "Nuevo Perfil".to_string();
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Presets buttons
                        ui.label(egui::RichText::new("Presets de Fábrica:").size(10.0).color(colors::TEXT_MUTED));
                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("+ Táctico CS2/Val").size(9.0)).clicked() {
                                let p = Profile::preset_tactical_flick();
                                manager.add_profile(p.clone());
                                manager.set_active_profile(&p.id);
                                self.status_message = Some(("Preset Táctico CS2/Valorant añadido.".to_string(), false));
                            }
                            if ui.button(egui::RichText::new("+ Tracking Apex").size(9.0)).clicked() {
                                let p = Profile::preset_tracking_smooth();
                                manager.add_profile(p.clone());
                                manager.set_active_profile(&p.id);
                                self.status_message = Some(("Preset Tracking Apex añadido.".to_string(), false));
                            }
                        });
                    });
                });

                ui.add_space(8.0);

                // Right Column: Active Profile Details & Export Options
                ui.vertical(|ui| {
                    let active = manager.active_profile_mut();

                    ui.group(|ui| {
                        ui.label(egui::RichText::new("PARÁMETROS DE HARDWARE").size(11.0).color(colors::TEXT_PRIMARY));
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Nombre del Perfil:").size(11.0).color(colors::TEXT_MUTED));
                            ui.text_edit_singleline(&mut active.name);
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("DPI Base del Sensor:").size(11.0).color(colors::TEXT_MUTED));
                            let dpis = [400, 800, 1600, 3200];
                            for d in dpis {
                                if ui.selectable_label(active.dpi == d, format!("{}", d)).clicked() {
                                    active.dpi = d;
                                }
                            }
                        });

                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Frecuencia (Polling Rate):").size(11.0).color(colors::TEXT_MUTED));
                            let rates = [1000, 2000, 4000, 8000];
                            for r in rates {
                                if ui.selectable_label(active.polling_rate_hz == r, format!("{} Hz", r)).clicked() {
                                    active.polling_rate_hz = r;
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Export / Interop
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("EXPORTACIÓN & COMPATIBILIDAD EXTERNA").size(11.0).color(colors::TEXT_PRIMARY));
                        ui.separator();

                        ui.label(
                            egui::RichText::new(
                                "Genera archivos listos para usar en drivers a nivel de kernel (RawAccel) o microcontroladores USB (RP2040)."
                            )
                            .size(10.0)
                            .color(colors::TEXT_MUTED),
                        );

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            if ui.button(egui::RichText::new("Exportar a RawAccel (settings.json)").strong().color(colors::ACCENT_CYAN)).clicked() {
                                match active.export_rawaccel_json() {
                                    Ok(json_str) => {
                                        let filename = format!("rawaccel_{}.json", active.id);
                                        match std::fs::write(&filename, json_str) {
                                            Ok(_) => {
                                                self.status_message = Some((format!("Configuración RawAccel exportada como '{}'.", filename), false));
                                            }
                                            Err(e) => {
                                                self.status_message = Some((format!("Error al escribir archivo: {}", e), true));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.status_message = Some((format!("Error al generar JSON: {}", e), true));
                                    }
                                }
                            }

                            if ui.button(egui::RichText::new("Exportar Tabla LUT (CSV)").color(colors::TEXT_PRIMARY)).clicked() {
                                let csv_str = active.export_polar_lut_csv();
                                let filename = format!("lut_{}.csv", active.id);
                                match std::fs::write(&filename, csv_str) {
                                    Ok(_) => {
                                        self.status_message = Some((format!("Tabla LUT (360°) exportada como '{}'.", filename), false));
                                    }
                                    Err(e) => {
                                        self.status_message = Some((format!("Error al escribir CSV: {}", e), true));
                                    }
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Credits & Engine Info
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Vectorists v0.1.0").monospace().size(10.0).color(colors::TEXT_MUTED));
                            ui.label(egui::RichText::new("•").size(10.0).color(colors::BORDER_STRONG));
                            ui.label(egui::RichText::new("Desarrollado por Aizendev").size(10.0).color(colors::TEXT_MUTED));
                            ui.label(egui::RichText::new("•").size(10.0).color(colors::BORDER_STRONG));
                            ui.label(egui::RichText::new("Motor: Rust Puro + eframe/wgpu").size(10.0).color(colors::ACCENT_CYAN));
                        });
                    });
                });
            });
        });
    }
}
