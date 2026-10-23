//! Profile library, device parameters and transparent local exports.
use crate::core::profile::{Profile, ProfileManager};
use crate::ui::theme::{ThemeAccent, accent, colors};
use crate::ui::widgets::{
    card_container, pill_chip, primary_button, secondary_button, section_heading,
};
use eframe::egui;

#[derive(Default)]
pub struct SettingsView {
    pub new_profile_name: String,
    pub status_message: Option<(String, bool)>,
}

impl SettingsView {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        manager: &mut ProfileManager,
        theme_accent: &mut ThemeAccent,
    ) {
        if let Some((message, error)) = &self.status_message {
            egui::Frame::new()
                .fill(if *error {
                    colors::DANGER_FAINT
                } else {
                    colors::SUCCESS_FAINT
                })
                .stroke(egui::Stroke::new(
                    1.0,
                    if *error {
                        colors::DANGER
                    } else {
                        colors::SUCCESS
                    },
                ))
                .corner_radius(10)
                .inner_margin(14)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new(message).size(13.0).color(if *error {
                        colors::DANGER
                    } else {
                        colors::SUCCESS
                    }));
                });
            ui.add_space(4.0);
        }
        let width = ui.available_width();
        if width >= 780.0 {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(290.0, 0.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.profile_library(ui, manager);
                    },
                );
                ui.vertical(|ui| {
                    ui.set_width(width - 302.0);
                    self.theme_selector(ui, theme_accent);
                    self.device_settings(ui, manager.active_profile_mut());
                    self.exports(ui, manager.active_profile());
                });
            });
        } else {
            self.profile_library(ui, manager);
            self.theme_selector(ui, theme_accent);
            self.device_settings(ui, manager.active_profile_mut());
            self.exports(ui, manager.active_profile());
        }
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("Vectorists v0.1.0")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.label(
                egui::RichText::new("Windows · Perfiles guardados en este equipo")
                    .size(12.0)
                    .color(colors::TEXT_FAINT),
            );
        });
    }

    fn theme_selector(&mut self, ui: &mut egui::Ui, theme_accent: &mut ThemeAccent) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Apariencia",
                "Un espacio de trabajo que se siente tuyo.",
            );
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for (accent, label, preview_color) in [
                    (ThemeAccent::GraphiteBlue, "Azul", colors::ACCENT_BLUE),
                    (ThemeAccent::SoftViolet, "Violeta", colors::ACCENT_VIOLET),
                    (ThemeAccent::MidnightGold, "Champán", colors::ACCENT_GOLD),
                ] {
                    let active = *theme_accent == accent;
                    if pill_chip(ui, label, active, preview_color) {
                        *theme_accent = accent;
                    }
                }
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(
                    "El color se aplica a los controles y a la curva de respuesta.",
                )
                .size(11.5)
                .color(colors::TEXT_MUTED),
            );
        });
    }

    fn profile_library(&mut self, ui: &mut egui::Ui, manager: &mut ProfileManager) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Tus perfiles",
                "Una configuración para cada forma de jugar.",
            );
            ui.add_space(4.0);
            let mut selected = manager.active_index;
            egui::ScrollArea::vertical()
                .id_salt("profile-library")
                .min_scrolled_height(224.0)
                .max_height(224.0)
                .show(ui, |ui| {
                    for (index, profile) in manager.profiles().iter().enumerate() {
                        ui.push_id(index, |ui| {
                            let active = selected == index;
                            if ui
                                .add_sized(
                                    [ui.available_width(), 42.0],
                                    egui::Button::selectable(
                                        active,
                                        egui::RichText::new(&profile.name).color(if active {
                                            accent(ui)
                                        } else {
                                            colors::TEXT_PRIMARY
                                        }),
                                    )
                                    .wrap(),
                                )
                                .clicked()
                            {
                                selected = index;
                            }
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} DPI  /  {} Hz",
                                    profile.dpi, profile.polling_rate_hz
                                ))
                                .size(11.0)
                                .color(colors::TEXT_FAINT),
                            );
                            ui.add_space(4.0);
                        });
                    }
                });
            if selected != manager.active_index {
                manager.set_active(selected);
            }
            ui.separator();
            ui.label(
                egui::RichText::new("Crear un perfil")
                    .size(13.0)
                    .strong()
                    .color(colors::TEXT_PRIMARY),
            );
            let label = ui.label(
                egui::RichText::new("Nombre del nuevo perfil")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.new_profile_name)
                    .hint_text("Ej. Mi configuración de CS2")
                    .char_limit(64)
                    .desired_width(ui.available_width()),
            )
            .labelled_by(label.id);
            let valid_name = !self.new_profile_name.trim().is_empty();
            ui.add_enabled_ui(valid_name, |ui| {
                if primary_button(ui, "+ Crear perfil").clicked() {
                    let name = self.new_profile_name.trim().to_owned();
                    let id = format!(
                        "custom-{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    );
                    manager.add_profile(Profile {
                        id,
                        name: name.clone(),
                        ..Profile::default()
                    });
                    self.new_profile_name.clear();
                    self.status_message = Some((
                        format!("Perfil «{name}» creado. Guarda los cambios para conservarlo."),
                        false,
                    ));
                }
            });
        });
    }

    fn device_settings(&mut self, ui: &mut egui::Ui, profile: &mut Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Ratón y sensibilidad",
                "Los valores de referencia de tu dispositivo.",
            );
            ui.add_space(4.0);
            let name_label = ui.label(
                egui::RichText::new("Nombre del perfil")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.add(
                egui::TextEdit::singleline(&mut profile.name)
                    .char_limit(64)
                    .desired_width(ui.available_width()),
            )
            .labelled_by(name_label.id);
            let dpi_label = ui.label(
                egui::RichText::new("Sensibilidad del sensor")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.horizontal_wrapped(|ui| {
                for dpi in [400, 800, 1600, 3200] {
                    if pill_chip(ui, &dpi.to_string(), profile.dpi == dpi, accent(ui)) {
                        profile.dpi = dpi;
                    }
                }
                ui.add(
                    egui::DragValue::new(&mut profile.dpi)
                        .range(100..=32000)
                        .speed(50.0)
                        .suffix(" DPI"),
                )
                .labelled_by(dpi_label.id);
            });
            ui.label(
                egui::RichText::new("Frecuencia de sondeo configurada")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.horizontal_wrapped(|ui| {
                for rate in [125, 500, 1000, 2000, 4000, 8000] {
                    if pill_chip(
                        ui,
                        &format!("{rate} Hz"),
                        profile.polling_rate_hz == rate,
                        accent(ui),
                    ) {
                        profile.polling_rate_hz = rate;
                    }
                }
            });
            ui.label(
                egui::RichText::new(
                    "Estos ajustes describen tu ratón; no modifican su configuración física.",
                )
                .size(11.0)
                .color(colors::TEXT_FAINT),
            );
        });
    }

    fn exports(&mut self, ui: &mut egui::Ui, profile: &Profile) {
        let width = ui.available_width();
        card_container(ui, |ui| {
            ui.set_width(width - 34.0);
            section_heading(
                ui,
                "Lleva tu curva contigo",
                "Exporta el perfil para una integración externa.",
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Configuración JSON")
                    .strong()
                    .color(colors::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(
                    "Parámetros del perfil, coeficientes de Fourier y tabla direccional.",
                )
                .size(12.0)
                .color(colors::TEXT_MUTED),
            );
            if secondary_button(ui, "Exportar configuración .json").clicked() {
                match profile.export_rawaccel_json() {
                    Ok(data) => self.write_export(profile, "rawaccel", "json", &data),
                    Err(error) => {
                        self.status_message =
                            Some((format!("No se pudo generar el JSON: {error}"), true))
                    }
                }
            }
            ui.separator();
            ui.label(
                egui::RichText::new("Tabla de respuesta CSV")
                    .strong()
                    .color(colors::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new("360 ángulos con sus multiplicadores originales y efectivos.")
                    .size(12.0)
                    .color(colors::TEXT_MUTED),
            );
            if secondary_button(ui, "Exportar tabla de 360° .csv").clicked() {
                self.write_export(profile, "lut", "csv", &profile.export_polar_lut_csv());
            }
            ui.add_space(4.0);
            {
                let directory = Self::export_directory();
                ui.label(
                    egui::RichText::new(format!("Carpeta de exportación: {}", directory.display()))
                        .size(11.0)
                        .color(colors::TEXT_FAINT),
                );
            }
            ui.label(
                egui::RichText::new(
                    "La corrección global requiere una integración compatible con estos datos.",
                )
                .size(11.0)
                .color(colors::TEXT_FAINT),
            );
        });
    }

    fn write_export(&mut self, profile: &Profile, prefix: &str, extension: &str, data: &str) {
        let path = Self::export_path(&profile.id, prefix, extension);
        self.status_message = Some(
            match std::fs::create_dir_all(Self::export_directory())
                .and_then(|()| std::fs::write(&path, data))
            {
                Ok(()) => (format!("Archivo exportado: {}", path.display()), false),
                Err(error) => (format!("No se pudo exportar el archivo: {error}"), true),
            },
        );
    }

    fn export_path(id: &str, prefix: &str, extension: &str) -> std::path::PathBuf {
        // Profile IDs may come from a user-edited file; keep exports in the displayed directory.
        let safe_id: String = id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Self::export_directory().join(format!("{prefix}_{safe_id}.{extension}"))
    }

    fn export_directory() -> std::path::PathBuf {
        ProfileManager::default_storage_path().with_file_name("exports")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exports_stay_in_the_displayed_folder_with_untrusted_profile_ids() {
        for id in [
            "default-neutral",
            "../../outside",
            "C:\\outside\\profile",
            "",
            "a/b:c",
        ] {
            let path = SettingsView::export_path(id, "lut", "csv");
            assert_eq!(
                path.parent(),
                Some(SettingsView::export_directory().as_path())
            );
            assert_eq!(path.extension().unwrap(), "csv");
        }
    }
}
