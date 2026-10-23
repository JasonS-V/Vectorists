# Vectorists (v0.1.0)

> **Motor de calibración biomecánica y sensibilidad direccional 360° para ratones de juego.**  
> Desarrollado en **Rust puro** con interfaz compacta HUD sobre `eframe`/`egui` y lectura directa de hardware con **Windows Raw Input**.

---

## ✦ ¿Qué es Vectorists?

Tradicionalmente, los ratones de ordenador aplican una sensibilidad uniforme o isotrópica: una distancia física recorrida genera la misma cantidad de cuentas (DPI) en cualquier dirección. Sin embargo, **la anatomía de la mano y muñeca humana es asimétrica**:
- El empuje vertical (extensión) tiene distinta inercia y rango articular que la flexión hacia abajo.
- El movimiento horizontal describe naturalmente un arco biomecánico (desviación radial vs cubital).
- Los jugadores sufren de *Directional Bias* (sesgo direccional): suelen sobrepasar objetivos (*overshoot*) en ciertos ángulos y quedarse cortos (*undershoot*) en otros, gastando decenas de milisegundos en micro-ajustes antes del disparo.

**Vectorists** es la alternativa libre, agnóstica de marca y de ultra-bajo consumo a tecnologías propietarias como *PureVector de Corsair*. Funciona con **cualquier ratón del mercado** (Logitech, Razer, Zowie, etc.) sin estar encadenado a ningún hardware de 200 dólares.

---

## ⚡ Características Principales

1. **Ajuste Armónico Polar Continuo (Serie de Fourier):**
   - Modela la sensibilidad en el espacio polar $\theta = \operatorname{atan2}(\Delta y, \Delta x) \in [0, 2\pi)$.
   - Interpola la corrección angular mediante una serie de Fourier truncada a $N=4$ armónicos, garantizando una curva suave sin saltos ni dientes de sierra en movimientos diagonales.

2. **Acumulador de Residuos Sub-Píxel (Anti-Jitter):**
   - Dado que los reportes HID son enteros discretos ($\mathbb{Z}$), multiplicar por flotantes provocaría pérdida de precisión a baja velocidad.
   - Vectorists preserva las fracciones sub-píxel entre paquetes consecutivos, garantizando la **conservación estricta del desplazamiento físico total**.

3. **Calibrador Biomecánico Radial Interactivo:**
   - Test integrado de 16 objetivos radiales. Mide el error longitudinal de flick (*overshoot* / *undershoot*) y el tiempo de asentamiento ($t_s$ en milisegundos).
   - Genera automáticamente el perfil polar compensatorio adaptado a tu anatomía.

4. **Telemetría de Hardware en Vivo:**
   - Hilo dedicado en segundo plano que escucha eventos de ratón sin aceleración mediante `WM_INPUT` y `RIDEV_INPUTSINK`.
   - Medición precisa de la frecuencia de sondeo (*Polling Rate real* en Hz) y cuentas por segundo (CPS).

5. **Interoperabilidad y Exportación:**
   - Exporta perfiles directamente al formato `settings.json` compatible con el driver de kernel **RawAccel** (permitido en torneos y anti-cheats como Vanguard, EAC y BattlEye).
   - Exporta tablas LUT de 360 grados en CSV para su uso en microcontroladores y dongles USB passthrough (Raspberry Pi Pico / RP2040).

---

## 🛠 Arquitectura Técnica

```
Vectorists
├── src/
│   ├── core/
│   │   ├── math.rs         # Modelo polar, descomposición cartesiana, Fourier N=4
│   │   ├── accumulator.rs  # Acumulador sub-píxel de residuos anti-jitter
│   │   ├── profile.rs      # Gestión de perfiles, presets (CS2/Val, Apex) y exportación RawAccel
│   │   └── mod.rs
│   ├── input/
│   │   ├── listener.rs     # Hilo Win32 Raw Input (WM_INPUT, RIDEV_INPUTSINK, telemetría Hz)
│   │   └── mod.rs
│   ├── ui/
│   │   ├── theme.rs        # Paleta Carbón Óptico & Cian Vectorial (WCAG AAA)
│   │   ├── widgets/radar.rs# Widget interactivo de Radar Polar 360° con inspección de ángulos
│   │   ├── views/
│   │   │   ├── dashboard.rs# Vista principal con simulador de vector en tiempo real
│   │   │   ├── calibrator.rs# Test radial de 16 direcciones
│   │   │   └── settings.rs # Gestión de perfiles, presets y exportación
│   │   ├── app.rs          # Implementación de eframe::App y barra de estado
│   │   └── mod.rs
│   └── main.rs
├── Cargo.toml
└── README.md
```

### Métricas de Rendimiento
- **Consumo de Memoria:** ~12 MB de RAM en reposo.
- **Latencia de Procesamiento:** $< 0.02\text{ ms}$ por paquete.
- **Dependencias:** Rust nativo puro (`eframe`/`egui` sobre `wgpu`, `windows-sys`). Sin navegadores ni WebView2.

---

## 🚀 Compilación y Ejecución

### Requisitos
- [Rust](https://www.rust-lang.org/) (edición 2024 o estable reciente).
- Sistema Operativo Windows 10 u 11 (para la captura de `WM_INPUT`).

### Compilar y Ejecutar en Modo Desarrollo
```bash
cargo run
```

### Compilar Binario Optimizado para Competición
```bash
cargo build --release
```
El ejecutable resultante se encontrará en `./target/release/vectorists.exe` (~4 MB, totalmente portable y sin dependencias externas).

---

## 👤 Autor & Licencia

- **Desarrollador:** Aizendev
- **Licencia:** MIT / Apache-2.0
