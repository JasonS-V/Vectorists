//! Profile management, persistence, and RawAccel interop for OpenVector.

use crate::core::math::{FourierModel, FOURIER_ORDER, TWO_PI};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const NUM_DIRECTIONS: usize = 16;

/// Profile containing biomechanical calibration results and sensitivity parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dpi: u32,
    pub polling_rate_hz: u32,
    /// Global compensation strength α (0.0 = disabled, 0.75 = recommended, 1.0 = 100% full, 2.0 = double).
    pub strength: f32,
    /// Harmonic coefficients for smooth 360° polar continuous evaluation.
    pub fourier: FourierModel,
    /// Discrete target multipliers for the 16 cardinal and inter-cardinal angles.
    /// Index 0 = 0° (Right), Index 4 = 90° (Up), Index 8 = 180° (Left), Index 12 = 270° (Down).
    pub direction_multipliers: [f32; NUM_DIRECTIONS],
    pub is_active: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: "default-neutral".to_string(),
            name: "Predeterminado (1:1)".to_string(),
            description: "Sensibilidad uniforme 1:1 sin corrección direccional.".to_string(),
            dpi: 800,
            polling_rate_hz: 1000,
            strength: 0.75,
            fourier: FourierModel::identity(),
            direction_multipliers: [1.0; NUM_DIRECTIONS],
            is_active: true,
        }
    }
}

impl Profile {
    /// Creates a custom calibrated profile from 16 direction multipliers.
    pub fn from_multipliers(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        dpi: u32,
        multipliers: [f32; NUM_DIRECTIONS],
    ) -> Self {
        let fourier = FourierModel::fit_equidistant(&multipliers);
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            dpi,
            polling_rate_hz: 1000,
            strength: 0.75,
            fourier,
            direction_multipliers: multipliers,
            is_active: false,
        }
    }

    /// Recomputes the Fourier model from the current 16 direction multipliers.
    pub fn recompute_fourier(&mut self) {
        self.fourier = FourierModel::fit_equidistant(&self.direction_multipliers);
    }

    /// Predefined preset for heavy tactical flicking (CS2 / Valorant).
    /// Typically compensates for common wrist arc undershoot on vertical/diagonal axes.
    pub fn preset_tactical_flick() -> Self {
        // Human biomechanics typically overshoot horizontal and undershoot vertical/top-left
        let mut multipliers = [1.0; NUM_DIRECTIONS];
        // 0° (Right): Slight horizontal dampening (-5%)
        multipliers[0] = 0.95;
        // 45° (Up-Right): Neutral
        multipliers[2] = 1.02;
        // 90° (Up): Vertical boost (+8%) to overcome wrist drag
        multipliers[4] = 1.08;
        // 135° (Up-Left): Diagonal wrist biomechanics boost (+6%)
        multipliers[6] = 1.06;
        // 180° (Left): Slight dampening (-4%)
        multipliers[8] = 0.96;
        // 225° (Down-Left): Slight drag compensation (+3%)
        multipliers[10] = 1.03;
        // 270° (Down): Vertical pull compensation (+5%)
        multipliers[12] = 1.05;
        // 315° (Down-Right): Neutral
        multipliers[14] = 1.01;

        // Smooth interpolate adjacent
        for i in (1..NUM_DIRECTIONS).step_by(2) {
            let prev = multipliers[(i + NUM_DIRECTIONS - 1) % NUM_DIRECTIONS];
            let next = multipliers[(i + 1) % NUM_DIRECTIONS];
            multipliers[i] = (prev + next) * 0.5;
        }

        Self::from_multipliers(
            "preset-tactical-flick",
            "Flick Táctico (CS2 / Valorant)",
            "Optimizado para frenada instantánea y compensación de arqueo de muñeca.",
            800,
            multipliers,
        )
    }

    /// Predefined preset for smooth tracking (Apex Legends / Overwatch).
    pub fn preset_smooth_tracking() -> Self {
        let mut multipliers = [1.0; NUM_DIRECTIONS];
        multipliers[4] = 1.04;
        multipliers[12] = 1.03;
        Self::from_multipliers(
            "preset-smooth-tracking",
            "Tracking Suave (Apex / OW2)",
            "Curva polar atenuada para seguimiento continuo a 360° sin tirones angulares.",
            1600,
            multipliers,
        )
    }

    /// Alias for `preset_smooth_tracking`.
    pub fn preset_tracking_smooth() -> Self {
        Self::preset_smooth_tracking()
    }

    /// Exports profile configuration to RawAccel interop JSON.
    pub fn to_rawaccel_config(&self) -> RawAccelConfig {
        RawAccelConfig {
            version: "openvector-v1.0".to_string(),
            profile_name: self.name.clone(),
            base_dpi: self.dpi,
            strength: self.strength,
            fourier_order: FOURIER_ORDER,
            a0: self.fourier.a0,
            harmonic_a: self.fourier.a.to_vec(),
            harmonic_b: self.fourier.b.to_vec(),
            direction_table: self
                .direction_multipliers
                .iter()
                .enumerate()
                .map(|(i, &m)| {
                    let deg = (i as f32) * 360.0 / (NUM_DIRECTIONS as f32);
                    DirectionTableEntry {
                        angle_deg: deg,
                        effective_multiplier: 1.0 + self.strength * (m - 1.0),
                    }
                })
                .collect(),
        }
    }

    /// Exports profile to RawAccel JSON string.
    pub fn export_rawaccel_json(&self) -> Result<String, serde_json::Error> {
        let config = self.to_rawaccel_config();
        serde_json::to_string_pretty(&config)
    }

    /// Exports the 360-degree discrete lookup table to CSV format.
    pub fn export_lut_csv(&self) -> String {
        let mut csv = String::from("angle_degrees,raw_multiplier,effective_multiplier\n");
        for deg in 0..360 {
            let theta = (deg as f32) * TWO_PI / 360.0;
            let raw = self.fourier.evaluate(theta);
            let effective = self.fourier.evaluate_with_strength(theta, self.strength);
            csv.push_str(&format!("{deg},{raw:.4},{effective:.4}\n"));
        }
        csv
    }

    /// Alias for `export_lut_csv`.
    pub fn export_polar_lut_csv(&self) -> String {
        self.export_lut_csv()
    }
}

/// RawAccel interop configuration format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAccelConfig {
    pub version: String,
    pub profile_name: String,
    pub base_dpi: u32,
    pub strength: f32,
    pub fourier_order: usize,
    pub a0: f32,
    pub harmonic_a: Vec<f32>,
    pub harmonic_b: Vec<f32>,
    pub direction_table: Vec<DirectionTableEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionTableEntry {
    pub angle_deg: f32,
    pub effective_multiplier: f32,
}

/// Manages multiple profiles and persists them to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManager {
    pub profiles: Vec<Profile>,
    pub active_index: usize,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self {
            profiles: vec![
                Profile::default(),
                Profile::preset_tactical_flick(),
                Profile::preset_smooth_tracking(),
            ],
            active_index: 0,
        }
    }
}

impl ProfileManager {
    /// Returns the currently active profile.
    pub fn active_profile(&self) -> &Profile {
        self.profiles
            .get(self.active_index)
            .unwrap_or(&self.profiles[0])
    }

    /// Returns a slice of all managed profiles.
    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    /// Returns a mutable reference to the currently active profile.
    pub fn active_profile_mut(&mut self) -> &mut Profile {
        if self.active_index >= self.profiles.len() {
            self.active_index = 0;
        }
        &mut self.profiles[self.active_index]
    }

    /// Sets active profile by index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.profiles.len() {
            for (i, p) in self.profiles.iter_mut().enumerate() {
                p.is_active = i == index;
            }
            self.active_index = index;
        }
    }

    /// Sets active profile by id.
    pub fn set_active_profile(&mut self, id: &str) -> bool {
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            self.set_active(pos);
            true
        } else {
            false
        }
    }

    /// Adds a new profile and selects it.
    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
        self.set_active(self.profiles.len() - 1);
    }

    /// Removes a profile by index (prevents removing the last remaining profile).
    pub fn remove_profile(&mut self, index: usize) -> bool {
        if self.profiles.len() <= 1 || index >= self.profiles.len() {
            return false;
        }
        self.profiles.remove(index);
        if self.active_index >= self.profiles.len() {
            self.active_index = self.profiles.len() - 1;
        }
        self.set_active(self.active_index);
        true
    }

    /// Returns the standard path where profiles are persisted:
    /// `%APPDATA%\OpenVector\profiles.json` on Windows.
    pub fn default_storage_path() -> PathBuf {
        let base_dir = dirs_or_fallback();
        base_dir.join("Vectorists").join("profiles.json")
    }

    /// Saves the current profiles to the default path.
    pub fn save_to_default_file(&self) -> Result<(), String> {
        self.save_to_file(&Self::default_storage_path())
    }

    /// Saves the current profiles to the given path.
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    /// Loads profiles from the default path, or returns default if not found.
    pub fn load_or_default() -> Self {
        Self::load_from_file(&Self::default_storage_path())
    }

    /// Loads profiles from the given path, or returns default if not found.
    pub fn load_from_file(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(manager) = serde_json::from_str::<Self>(&content) {
                return manager;
            }
        }
        Self::default()
    }
}

/// Returns standard config directory or current directory fallback.
fn dirs_or_fallback() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata);
        }
    }
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_manager_defaults() {
        let mut mgr = ProfileManager::default();
        assert_eq!(mgr.profiles.len(), 3);
        assert_eq!(mgr.active_index, 0);

        mgr.set_active(1);
        assert_eq!(mgr.active_profile().name, "Flick Táctico (CS2 / Valorant)");
    }

    #[test]
    fn test_profile_serialization_roundtrip() {
        let original = Profile::preset_tactical_flick();
        let json = serde_json::to_string(&original).expect("Serialization failed");
        let decoded: Profile = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(original.name, decoded.name);
        assert_eq!(original.direction_multipliers, decoded.direction_multipliers);
        assert_eq!(original.fourier.a0, decoded.fourier.a0);
    }

    #[test]
    fn test_rawaccel_export() {
        let profile = Profile::preset_tactical_flick();
        let rawaccel = profile.to_rawaccel_config();
        assert_eq!(rawaccel.direction_table.len(), NUM_DIRECTIONS);
        assert_eq!(rawaccel.fourier_order, FOURIER_ORDER);
        let json = serde_json::to_string_pretty(&rawaccel).expect("RawAccel json failed");
        assert!(json.contains("openvector-v1.0"));
    }

    #[test]
    fn test_lut_csv_export() {
        let profile = Profile::default();
        let csv = profile.export_lut_csv();
        let lines: Vec<&str> = csv.lines().collect();
        // 1 header + 360 lines
        assert_eq!(lines.len(), 361);
        assert_eq!(lines[0], "angle_degrees,raw_multiplier,effective_multiplier");
    }
}
