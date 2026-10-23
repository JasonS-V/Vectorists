//! Native workspace shell, profile switching and persistent status.
use crate::core::profile::ProfileManager;
use crate::input::{RawInputListener, SensorTelemetry};
use crate::ui::theme::{ThemeAccent, accent, apply_theme, apply_theme_with_accent, colors};
use crate::ui::views::{CalibratorView, DashboardView, SettingsView};
use crate::ui::widgets::{NavIcon, nav_item, primary_button, secondary_button, toggle_switch};
use eframe::egui::{self, Frame, Stroke};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Dashboard,
    Calibrator,
    Settings,
}

impl ActiveTab {
    fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Mapa de respuesta",
            Self::Calibrator => "Encuentra tu equilibrio",
            Self::Settings => "Perfiles y configuración",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Dashboard => "Una sensibilidad consistente. En todas las direcciones.",
            Self::Calibrator => "Mide tu movimiento natural y crea una curva a tu medida.",
            Self::Settings => "Tu hardware, tus ajustes y tus curvas. En un solo lugar.",
        }
    }
}

pub struct VectoristsApp {
    pub active_tab: ActiveTab,
    pub profile_manager: ProfileManager,
    pub dashboard_view: DashboardView,
    pub calibrator_view: CalibratorView,
    pub settings_view: SettingsView,
    pub raw_listener: Option<RawInputListener>,
    pub theme_accent: ThemeAccent,
    saved_profiles: ProfileManager,
    save_status: Option<(String, bool)>,
}

impl Default for VectoristsApp {
    fn default() -> Self {
        let profile_manager = ProfileManager::load_or_default();
        Self {
            saved_profiles: profile_manager.clone(),
            profile_manager,
            active_tab: ActiveTab::Dashboard,
            dashboard_view: DashboardView::default(),
            calibrator_view: CalibratorView::default(),
            settings_view: SettingsView::default(),
            raw_listener: Some(RawInputListener::start()),
            theme_accent: ThemeAccent::default(),
            save_status: None,
        }
    }
}

impl VectoristsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_theme(&cc.egui_ctx);
        Self::default()
    }

    fn has_changes(&self) -> bool {
        self.profile_manager.active_index != self.saved_profiles.active_index
            || self.profile_manager.profiles != self.saved_profiles.profiles
    }

    fn save(&mut self) {
        self.save_status = Some(match self.profile_manager.save_to_default_file() {
            Ok(()) => {
                self.saved_profiles = self.profile_manager.clone();
                ("Todos los cambios guardados".into(), false)
            }
            Err(error) => (format!("No se pudo guardar: {error}"), true),
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), egui::Sense::hover());
            let c = rect.center();
            let p = ui.painter();
            p.rect_filled(rect, 10, accent(ui));
            p.line_segment(
                [c + egui::vec2(-12.0, -10.0), c + egui::vec2(0.0, 11.0)],
                Stroke::new(2.5, colors::SIDEBAR_BG),
            );
            p.line_segment(
                [c + egui::vec2(0.0, 11.0), c + egui::vec2(12.0, -10.0)],
                Stroke::new(2.5, colors::SIDEBAR_BG),
            );
            p.line_segment(
                [c + egui::vec2(0.0, -5.0), c + egui::vec2(5.0, -14.0)],
                Stroke::new(3.0, colors::TEXT_PRIMARY),
            );
            if !compact {
                ui.label(
                    egui::RichText::new("vectorists")
                        .size(22.0)
                        .strong()
                        .color(colors::TEXT_PRIMARY),
                );
            }
        });
        ui.add_space(32.0);
        if !compact {
            ui.label(
                egui::RichText::new("ESPACIO DE TRABAJO")
                    .size(10.5)
                    .color(colors::TEXT_MUTED),
            );
            ui.add_space(6.0);
        }
        for (tab, label, icon) in [
            (ActiveTab::Dashboard, "Vista general", NavIcon::Polar),
            (ActiveTab::Calibrator, "Calibración", NavIcon::Target),
            (ActiveTab::Settings, "Configuración", NavIcon::Settings),
        ] {
            if nav_item(ui, label, icon, self.active_tab == tab, compact) {
                self.active_tab = tab;
            }
        }
        ui.add_space((ui.available_height() - if compact { 30.0 } else { 270.0 }).max(12.0));
        if !compact {
            ui.label(
                egui::RichText::new("HECHO PARA TU MOVIMIENTO")
                    .size(10.0)
                    .color(colors::TEXT_MUTED),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Tu precisión empieza aquí.")
                    .size(16.0)
                    .color(colors::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(
                    "Calibra 16 direcciones y encuentra una respuesta a tu medida.",
                )
                .size(12.0)
                .color(colors::TEXT_MUTED),
            );
            ui.add_space(8.0);
            if secondary_button(ui, "Calibrar mi ratón  →").clicked() {
                self.active_tab = ActiveTab::Calibrator;
            }
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("●  Windows · Raw Input")
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
            );
        }
        ui.label(
            egui::RichText::new(if compact {
                "v0.1"
            } else {
                "VECTORISTS  /  v0.1.0"
            })
            .size(10.0)
            .color(colors::TEXT_MUTED),
        );
    }

    fn profile_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let mut index = self.profile_manager.active_index;
            egui::ComboBox::from_id_salt("active-profile")
                .width(190.0)
                .truncate()
                .selected_text(&self.profile_manager.active_profile().name)
                .show_ui(ui, |ui| {
                    for (i, profile) in self.profile_manager.profiles().iter().enumerate() {
                        ui.selectable_value(&mut index, i, &profile.name);
                    }
                })
                .response
                .on_hover_text("Perfil activo");
            if index != self.profile_manager.active_index {
                self.profile_manager.set_active(index);
            }
            if primary_button(ui, "Guardar perfil")
                .on_hover_text("Guardar todos los perfiles · Ctrl+S")
                .clicked()
            {
                self.save();
            }
        });
    }

    fn workspace_ui(&mut self, ui: &mut egui::Ui, telemetry: &SensorTelemetry) {
        let compact = ui.available_width() < 1000.0;
        if ui.input_mut(|input| {
            input.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::S,
            ))
        }) {
            self.save();
        }
        egui::Panel::bottom("status")
            .exact_size(34.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(colors::SIDEBAR_BG)
                    .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                    .inner_margin(egui::Margin::symmetric(20, 7)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let streaming = telemetry.polling_rate_hz > 0;
                    ui.label(
                        egui::RichText::new(if streaming {
                            "●  Sensor en movimiento"
                        } else {
                            "○  Sensor en espera"
                        })
                        .size(11.5)
                        .color(if streaming {
                            colors::SUCCESS
                        } else {
                            colors::TEXT_MUTED
                        }),
                    );
                    if ui.max_rect().width() > 950.0 {
                        let profile = self.profile_manager.active_profile();
                        ui.label(
                            egui::RichText::new(format!(
                                "{} DPI  ·  {} Hz configurados",
                                profile.dpi, profile.polling_rate_hz
                            ))
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_MUTED),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some((_, true)) = &self.save_status {
                            ui.label(
                                egui::RichText::new("● Error al guardar")
                                    .size(11.5)
                                    .color(colors::DANGER),
                            );
                        } else if self.has_changes() {
                            ui.label(
                                egui::RichText::new("● Cambios sin guardar")
                                    .size(11.5)
                                    .color(colors::WARNING),
                            );
                        } else if let Some((message, error)) = &self.save_status {
                            ui.label(egui::RichText::new(message).size(11.5).color(if *error {
                                colors::DANGER
                            } else {
                                colors::SUCCESS
                            }));
                        } else {
                            ui.label(
                                egui::RichText::new("Almacenamiento local")
                                    .size(11.5)
                                    .color(colors::TEXT_MUTED),
                            );
                        }
                    });
                });
            });
        egui::Panel::left("navigation")
            .exact_size(if compact { 76.0 } else { 216.0 })
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(colors::SIDEBAR_BG)
                    .stroke(Stroke::new(1.0, colors::BORDER_SUBTLE))
                    .inner_margin(egui::Margin::symmetric(14, 18)),
            )
            .show(ui, |ui| self.sidebar(ui, compact));
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(colors::CHASSIS_BG)
                    .inner_margin(if compact { 20 } else { 28 }),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "Espacio de trabajo  /  ".to_owned()
                                + match self.active_tab {
                                    ActiveTab::Dashboard => "Vista general",
                                    ActiveTab::Calibrator => "Calibración",
                                    ActiveTab::Settings => "Configuración",
                                },
                        )
                        .size(10.5)
                        .color(colors::TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let profile = self.profile_manager.active_profile_mut();
                        toggle_switch(
                            ui,
                            &mut profile.is_active,
                            "Activar corrección en la vista previa",
                        );
                        ui.label(
                            egui::RichText::new(if profile.is_active {
                                "Vista previa activa"
                            } else {
                                "Vista previa en pausa"
                            })
                            .size(12.5)
                            .color(if profile.is_active {
                                accent(ui)
                            } else {
                                colors::TEXT_MUTED
                            }),
                        );
                    });
                });
                ui.add_space(4.0);
                if ui.available_width() >= 850.0 {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 4.0;
                            ui.set_width((ui.available_width() - 390.0).max(250.0));
                            ui.heading(self.active_tab.title());
                            ui.label(
                                egui::RichText::new(self.active_tab.description())
                                    .size(13.5)
                                    .color(colors::TEXT_MUTED),
                            );
                        });
                        self.profile_controls(ui);
                    });
                } else {
                    ui.heading(self.active_tab.title());
                    ui.label(
                        egui::RichText::new(self.active_tab.description())
                            .size(13.5)
                            .color(colors::TEXT_MUTED),
                    );
                    self.profile_controls(ui);
                }
                ui.add_space(4.0);
                if let Some((message, true)) = &self.save_status {
                    Frame::new()
                        .fill(colors::DANGER_FAINT)
                        .stroke(Stroke::new(1.0, colors::DANGER))
                        .corner_radius(10)
                        .inner_margin(14)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(egui::RichText::new(message).color(colors::DANGER));
                        });
                    ui.add_space(8.0);
                }
                // A bounded vertical viewport lets each page reflow instead of hiding controls offscreen.
                egui::ScrollArea::vertical()
                    .id_salt("workspace-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match self.active_tab {
                            ActiveTab::Dashboard => self.dashboard_view.ui(
                                ui,
                                self.profile_manager.active_profile_mut(),
                                telemetry.polling_rate_hz,
                                telemetry.counts_per_sec,
                            ),
                            ActiveTab::Calibrator => self
                                .calibrator_view
                                .ui(ui, self.profile_manager.active_profile_mut()),
                            ActiveTab::Settings => {
                                let old_accent = self.theme_accent;
                                self.settings_view.ui(
                                    ui,
                                    &mut self.profile_manager,
                                    &mut self.theme_accent,
                                );
                                if self.theme_accent != old_accent {
                                    apply_theme_with_accent(ui.ctx(), self.theme_accent.color());
                                }
                            }
                        }
                        ui.add_space(8.0);
                    });
            });
    }
}

impl eframe::App for VectoristsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let telemetry = self
            .raw_listener
            .as_ref()
            .map(|listener| {
                // Consume reports even outside the simulator so the input queue stays bounded.
                let _ = listener.drain_packets();
                listener.telemetry()
            })
            .unwrap_or_default();
        self.workspace_ui(ui, &telemetry);
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_without_hardware() -> VectoristsApp {
        let profiles = ProfileManager::default();
        VectoristsApp {
            active_tab: ActiveTab::Dashboard,
            saved_profiles: profiles.clone(),
            profile_manager: profiles,
            dashboard_view: DashboardView::default(),
            calibrator_view: CalibratorView::default(),
            settings_view: SettingsView::default(),
            raw_listener: None,
            theme_accent: ThemeAccent::default(),
            save_status: None,
        }
    }

    #[test]
    fn unsaved_indicator_tracks_edits_and_profile_selection() {
        let mut app = app_without_hardware();
        assert!(!app.has_changes());
        app.profile_manager.active_profile_mut().strength = 0.9;
        assert!(app.has_changes());
        app.saved_profiles = app.profile_manager.clone();
        assert!(!app.has_changes());
        app.profile_manager.set_active(1);
        assert!(app.has_changes());
    }

    #[test]
    fn long_profile_names_fit_the_workspace() {
        for size in [[680.0, 560.0], [1280.0, 800.0]] {
            let ctx = egui::Context::default();
            apply_theme(&ctx);
            let mut app = app_without_hardware();
            app.profile_manager.active_profile_mut().name = "W".repeat(64);
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, size.into());
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(rect),
                    ..Default::default()
                },
                |ui| app.workspace_ui(ui, &SensorTelemetry::default()),
            );
            output.textures_delta.clear();
            assert!(
                ctx.globally_used_rect().right() <= rect.right() + 1.0,
                "Long profile name overflowed at {size:?}: {:?}",
                ctx.globally_used_rect()
            );
        }
    }

    #[test]
    fn workspace_reflows_without_horizontal_overflow() {
        for size in [
            [680.0, 560.0],
            [900.0, 650.0],
            [1024.0, 768.0],
            [1280.0, 900.0],
            [1440.0, 900.0],
        ] {
            for tab in [
                ActiveTab::Dashboard,
                ActiveTab::Calibrator,
                ActiveTab::Settings,
            ] {
                let ctx = egui::Context::default();
                apply_theme(&ctx);
                let mut app = app_without_hardware();
                app.active_tab = tab;
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, size.into());
                // Let automatic layout settle before inspecting the used bounds.
                for _ in 0..3 {
                    let mut output = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(rect),
                            ..Default::default()
                        },
                        |ui| {
                            app.workspace_ui(ui, &SensorTelemetry::default());
                            assert!(
                                ui.min_rect().right() <= rect.right() + 1.0,
                                "{tab:?} at {size:?} overflowed: {:?}",
                                ui.min_rect()
                            );
                        },
                    );
                    output.textures_delta.clear();
                }
            }
        }
    }

    #[test]
    fn calibration_arena_fits_without_scrolling_to_targets() {
        for size in [[680.0, 560.0], [1200.0, 780.0], [1440.0, 900.0]] {
            for state in [
                crate::ui::views::calibrator::CalibratorState::Idle,
                crate::ui::views::calibrator::CalibratorState::WaitingForCenter,
                crate::ui::views::calibrator::CalibratorState::Completed,
            ] {
                let ctx = egui::Context::default();
                apply_theme(&ctx);
                let mut app = app_without_hardware();
                app.active_tab = ActiveTab::Calibrator;
                app.calibrator_view.state = state;
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size.into())),
                        ..Default::default()
                    },
                    |ui| app.workspace_ui(ui, &SensorTelemetry::default()),
                );
                output.textures_delta.clear();
                let arena = output
                    .shapes
                    .iter()
                    .find_map(|shape| {
                        if let egui::Shape::Rect(rect) = &shape.shape
                            && rect.fill == colors::SIDEBAR_BG
                            && rect.rect.width() > 300.0
                            && rect.rect.height() > 170.0
                        {
                            Some((rect.rect, shape.clip_rect))
                        } else {
                            None
                        }
                    })
                    .expect("The calibration arena should be rendered");
                assert!(
                    arena.0.bottom() <= arena.1.bottom() + 1.0,
                    "{state:?} at {size:?}: {arena:?}"
                );
            }
        }
    }
}
