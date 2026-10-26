# Decisiones tomadas (OnePepe)

Formato: qué elegí y por qué.

## Fase 0

- **Ramas.** `onepepe` se creó desde `main`; cada función va en `feat/<nombre>` y se fusiona en `onepepe` con `--no-ff` cuando compila y pasa sus tests. La rama `claude/modest-cerf-imwi3h` de la sesión no se usa para el trabajo.
- **Compilar la UI sin GTK 4.22.** En la nube solo hay GTK 4.14, así que compruebo la UI con `cargo check` diciéndole a `system-deps` que no use pkg-config (`SYSTEM_DEPS_GTK4_NO_PKG_CONFIG=1`…). Comprueba tipos y sintaxis de Rust, pero no enlaza ni ejecuta: los errores en archivos `.ui` solo se verían al ejecutar.
- **`config.rs` local.** Para el `cargo check` de la UI generé `crates/rnote-ui/src/config.rs` a mano (meson lo genera normalmente). No se sube al repositorio (lo puse en `.git/info/exclude`).
- **Test de hojas infinitas como test de integración** (`crates/rnote-engine/tests/infinite_layout.rs`) en la rama `feat/hojas-infinitas`, para no tocar código existente.
- **`setup-deps.sh` también comprueba `libinput` y `libwacom`** aunque `BUILDING.md` no los pide, porque los pediste tú y GTK los usa con la tableta. Si las versiones de glib/GTK/libadwaita son demasiado viejas, el script no intenta instalar nada raro y recomienda Flatpak.
