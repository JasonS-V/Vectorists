# Rediseño de Vectorists para Windows

Objetivo: renovar las tres vistas de la aplicación nativa a partir de las referencias del usuario: navegación lateral, superficies de grafito, acento azul, tarjetas sobrias y tipografía Segoe UI. Conservar perfiles, calibración, exportaciones, atajos y motor de entrada.

Implementación: primero tema y estructura; después panel de respuesta y controles; finalmente configuración, calibración y verificación visual en Windows. Los controles de ventana son los nativos del sistema. Las métricas deben mostrar datos reales o un estado de espera.

Código: Rust 2024 y egui existente, sin dependencias nuevas. Los cambios quedan en `src/ui`, el tamaño inicial en `src/main.rs` y la descripción en `README.md`. Seguir los controles y el formato existentes; los colores de acento se obtienen del tema activo.

Criterios: navegación y controles accesibles con teclado, las tres vistas sin desbordamiento horizontal desde 680 × 560, mapa polar seleccionable, vista previa y controles de perfil operativos. Comprobar con `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo build` e inspección de la aplicación ejecutándose en Windows. No modificar archivos de perfiles del usuario durante las pruebas ni alterar el motor de corrección.

Validación final: `cargo test` pasa las 27 pruebas y `cargo build` compila el ejecutable. El formato de todos los archivos Rust modificados pasa `rustfmt --check --edition 2024 --config skip_children=true`, y `git diff --check` pasa. La inspección nativa confirma navegación, nombres accesibles, cambio de acento e inicio de calibración. No se guardaron cambios en los perfiles durante estas comprobaciones.

Limitaciones previas: el formato global tiene diferencias en archivos sin modificar de `src/core` y `src/input`. `cargo clippy --all-targets` termina correctamente con siete avisos existentes en `core/math.rs`, `core/profile.rs` e `input/listener.rs`; por eso su variante con `-D warnings` no pasa. No se amplió el rediseño para modificar el motor.
