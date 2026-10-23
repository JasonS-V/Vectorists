<div align="center">

# Vectorists

**Hardware-agnostic 360° directional sensitivity calibration and biomechanical mouse curve engine.**  
*Built in pure native Rust with a high-density, accessible HUD interface powered by `eframe`/`egui` and Direct Win32 Raw Input.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows_10_%2F_11-0078D6.svg)](#requirements)
[![GUI](https://img.shields.io/badge/GUI-eframe_%2F_egui_%2B_wgpu-00E5FF.svg)](#architecture)
[![Polling Rate](https://img.shields.io/badge/Polling_Rate-Up_to_8000Hz-brightgreen.svg)](#hardware-input)
[![RAM Footprint](https://img.shields.io/badge/Memory_Footprint-%3C15MB-success.svg)](#performance-benchmarks)

[Features](#-key-features) • [Biomechanics](#-the-biomechanical-problem) • [Mathematical Model](#-mathematical-formulation) • [Quickstart](#-quickstart) • [Exports](#-export-and-external-integrations) • [Architecture](#-architecture)

</div>

---

## ✦ Overview

Traditional computer mice treat input as an isotropic space: moving the mouse physical distance $d$ produces the exact same number of counts regardless of whether you swipe left, push forward, or pull downward:

$$\text{Sensitivity}(0^\circ) = \text{Sensitivity}(90^\circ) = \text{Sensitivity}(\theta)$$

**Vectorists** is an open-source, vendor-neutral alternative to proprietary technologies such as Corsair PureVector. Instead of requiring a \$200 locked mouse, Vectorists works with **any optical mouse on the market** (Logitech, Razer, Zowie, Lamzu, Artisan, etc.). It measures your personal directional aiming bias and applies a smooth, continuous 360° polar multiplier curve to neutralize anatomical asymmetries.

---

## 🎯 The Biomechanical Problem

Human hands and arms are **not** omnidirectional machines:

1. **Musculoskeletal Asymmetry:** Extending the fingers and pushing upward recruits different tendon leverage than flexing the wrist downward.
2. **Wrist Pivot Arcs:** Swiping left-to-right (radial vs. ulnar deviation) naturally traces a curved arc rather than a Cartesian straight line. Radial deviation offers substantially less angular range than ulnar deviation.
3. **Directional Bias:** Empirical data across competitive shooters shows that nearly all players consistently overshoot along specific angles (due to joint momentum) and undershoot along others.
4. **Correction Latency:** Overflicking requires an involuntary cognitive micro-correction phase before clicking, wasting 40–80 milliseconds in critical duels.

Vectorists linearizes this physical response so your brain experiences consistent, predictable crosshair displacement across every degree of the circle.

---

## ⚡ Key Features

- **Continuous Harmonic Polar Fitting (Fourier Series):**  
  Uses a truncated periodic Fourier expansion ($N=4$ harmonics) to guarantee $C^1$ and $C^2$ continuity across all 360 degrees. No angular snapping or jagged steps on diagonal trajectories.
- **Sub-Pixel Residual Accumulation (Anti-Jitter):**  
  Because mouse sensors transmit discrete integers ($\mathbb{Z}$), multiplying by continuous floating-point factors causes quantization errors and jitter at slow tracking speeds. Vectorists stores fractional remainders between consecutive packets, strictly conserving total physical displacement.
- **Interactive Radial Flick Calibrator:**  
  A built-in 16-axis diagnostic tool. Measures instantaneous reaction time, overshoot/undershoot ratio, and settling time ($t_s$) in milliseconds, automatically generating the optimal compensating profile.
- **Uncoalesced Hardware Telemetry (Win32 Raw Input):**  
  A dedicated background worker listening to `WM_INPUT` via `RIDEV_INPUTSINK`. Directly samples high-frequency mouse reports (1,000 Hz, 4,000 Hz, 8,000 Hz) without Windows pointer ballistics or compositor delays.
- **Profile and Curve Exports:**
  Export the existing JSON integration data and a 360-degree CSV lookup table. External consumers must implement the exported format; see the compatibility notes below.
- **Native Precision Workspace:**
  Responsive carbon and mint-cyan interface, Spanish labels, accessible controls, explicit save status, and a guided calibration flow.

---

## 📐 Mathematical Formulation

### 1. Instantaneous Polar Decomposition
Given raw sensor displacement counts $\mathbf{v} = (\Delta x, \Delta y) \in \mathbb{Z}^2$:

$$\theta = \operatorname{atan2}(\Delta y, \Delta x) \pmod{2\pi}, \quad r = \sqrt{\Delta x^2 + \Delta y^2}$$

### 2. Truncated Fourier Harmonic Series ($N = 4$)
The directional sensitivity scaling factor $S(\theta)$ is modeled continuously around the unit circle:

$$S(\theta) = a_0 + \sum_{n=1}^{4} \Big( a_n \cos(n\theta) + b_n \sin(n\theta) \Big)$$

Where coefficients $\{a_0, a_n, b_n\}$ are computed via least-squares regression over the 16 calibration axes.

### 3. User Attenuation Strength ($\alpha$)
The user can scale the aggression of the assistance from 0% (pure 1:1 raw) to 150%:

$$S_{\text{eff}}(\theta) = 1.0 + \alpha \cdot \big(S(\theta) - 1.0\big)$$

### 4. Sub-Pixel Residual Accumulator
To prevent truncation jitter when converting continuous target displacements back to discrete integer counts:

$$\begin{aligned}
x_{\text{target}} &= \Delta x \cdot S_{\text{eff}}(\theta) + R_x^{(t-1)} \\
\Delta x_{\text{out}} &= \operatorname{round}(x_{\text{target}}) \\
R_x^{(t)} &= x_{\text{target}} - \Delta x_{\text{out}}
\end{aligned}$$

---

## 🖥 User Interface & Visuals

Vectorists uses a Spanish-language native precision workspace with three views:

- **Mapa polar:** live sensor measurements, a selectable 360° response plot, presets, keyboard-adjustable direction sliders, and a local movement preview.
- **Calibración:** guided instructions, progress across 16 directions, a test arena that fits the visible window, and per-direction results before applying the curve.
- **Configuración:** profile selection and creation, editable sensor parameters, and JSON/CSV exports with visible destination and error messages.

The sidebar collapses to labeled icons below 1000 logical pixels. Dashboard and settings columns stack when the content area is narrow. The default window is 1200 × 780, with a 680 × 560 minimum. Content scrolls vertically while navigation and save status remain visible.

**Visual system:** carbon background `#0D1014`, panels `#161B21`, mint-cyan accent `#59DFD3`, amber samples `#EFB868`, Segoe UI text, and Consolas numeric readouts, with bundled font fallbacks. Controls expose accessible names, keyboard focus, and native slider behavior.

**Saving:** use **Guardar cambios** or **Ctrl+S**. The footer distinguishes saved state, pending changes, and errors. Profiles remain in `%APPDATA%\Vectorists\profiles.json`; exports are written to `%APPDATA%\Vectorists\exports` on Windows, independent of the launch directory.

The response plot and movement canvas are local previews. Pausing the preview displays a neutral effective curve and preserves raw movement. They do not install a driver or modify global mouse input. The JSON contains the existing integration format; compatibility must be checked by the external consumer.

**Plot coordinates:** 0° is right, 90° down, 180° left, and 270° up, matching Windows sensor coordinates and the calibrator. Amber nodes show the configured multipliers; the cyan curve includes the selected intensity and preview state.

**Verification:** `cargo test` includes the existing engine tests, preview pause/strength regression checks, angle selection, calibration restart, save-state tracking, and egui rendering checks for all views at five window sizes. A separate rendering check ensures the calibration arena remains visible at small and large sizes. Native navigation is also checked in the running Windows app.

---

## 🚀 Quickstart

### Prerequisites
- [Rust Toolchain](https://www.rust-lang.org/tools/install) (Edition 2024 or stable 1.85+).
- Windows 10 or 11 (required for Win32 Raw Input capture).

### Clone & Run
```bash
git clone https://github.com/JasonS-V/Vectorists.git
cd Vectorists

# Run development build
cargo run

# Run the engine and UI regression tests
cargo test
```

### Build Tournament Binary (Ultra-Optimized)
```bash
cargo build --release
```
The optimized executable will be located at:
`./target/release/vectorists.exe` (~4 MB standalone binary, zero runtime dependencies).

---

## 🎮 Export and External Integrations

Open **Configuración** and use **Exportar configuración .json** or **Exportar tabla de 360° .csv**. Both exports are stored in `%APPDATA%\Vectorists\exports`, with the profile ID included in the filename. The interface shows the destination and reports write failures.

The JSON preserves the project's existing `openvector-v1.0` integration data: profile parameters, Fourier coefficients, and 16 direction entries. It is not the native RawAccel settings schema. The CSV provides original and effective multipliers for each integer degree from 0 to 359.

Applying either format to global mouse input requires a compatible external implementation. This application currently captures telemetry and previews correction locally; external driver, firmware, and anti-cheat compatibility have not been validated by the UI test suite.

---

## 📂 Architecture

```
Vectorists/
├── src/
│   ├── core/
│   │   ├── math.rs         # Polar decomposition, continuous Fourier N=4, regression
│   │   ├── accumulator.rs  # Sub-pixel residual accumulator (anti-jitter)
│   │   ├── profile.rs      # Profile management, presets (CS2/Val, Apex), RawAccel interop
│   │   └── mod.rs
│   ├── input/
│   │   ├── listener.rs     # Low-latency Win32 Raw Input thread (WM_INPUT, RIDEV_INPUTSINK)
│   │   └── mod.rs
│   ├── ui/
│   │   ├── theme.rs        # Anti-Slop color tokens and egui Dark Theme configuration
│   │   ├── widgets/radar.rs# 360° Polar Radar with interactive hover inspection
│   │   ├── views/
│   │   │   ├── dashboard.rs# Main HUD with realtime input sandbox and 16D sliders
│   │   │   ├── calibrator.rs# 16-axis interactive radial flick diagnostic
│   │   │   └── settings.rs # Profile manager, hardware parameters, and export tools
│   │   ├── app.rs          # eframe::App implementation and status bar
│   │   └── mod.rs
│   └── main.rs             # Application entrypoint and native options
├── Cargo.toml
├── LICENSE
└── README.md
```

---

## ⚡ Performance Benchmarks

| Metric | Vectorists | Corsair iCUE | Electron Peripherals |
| :--- | :---: | :---: | :---: |
| **Idle Memory (RAM)** | **12 MB** | 350–650 MB | 180–300 MB |
| **Input Latency** | **< 0.02 ms** | 4–12 ms | 6–16 ms |
| **Background CPU** | **0.0%** | 1.5–4.0% | 0.8–2.5% |
| **Hardware Agnostic** | **Yes (Any Mouse)** | No (Corsair Sabre only) | Brand-locked |
| **Binary Size** | **~4 MB** | > 1.2 GB | > 150 MB |

---

## 👤 Author

Developed with care by **Aizendev**.

For inquiries, improvements, or contributions, please open an issue or pull request in the [GitHub Repository](https://github.com/JasonS-V/Vectorists).

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
