# Decisiones tomadas (OnePepe)

Formato: qué elegí y por qué.

## Fase 0

- **Ramas.** `onepepe` se creó desde `main`; cada función va en `feat/<nombre>` y se fusiona en `onepepe` con `--no-ff` cuando compila y pasa sus tests. La rama `claude/modest-cerf-imwi3h` de la sesión no se usa para el trabajo.
- **Compilar la UI sin GTK 4.22.** En la nube solo hay GTK 4.14, así que compruebo la UI con `cargo check` diciéndole a `system-deps` que no use pkg-config (`SYSTEM_DEPS_GTK4_NO_PKG_CONFIG=1`…). Comprueba tipos y sintaxis de Rust, pero no enlaza ni ejecuta: los errores en archivos `.ui` solo se verían al ejecutar.
- **`config.rs` local.** Para el `cargo check` de la UI generé `crates/rnote-ui/src/config.rs` a mano (meson lo genera normalmente). No se sube al repositorio (lo puse en `.git/info/exclude`).
- **Test de hojas infinitas como test de integración** (`crates/rnote-engine/tests/infinite_layout.rs`) en la rama `feat/hojas-infinitas`, para no tocar código existente.
- **`setup-deps.sh` también comprueba `libinput` y `libwacom`** aunque `BUILDING.md` no los pide, porque los pediste tú y GTK los usa con la tableta. Si las versiones de glib/GTK/libadwaita son demasiado viejas, el script no intenta instalar nada raro y recomienda Flatpak.

## Fase 1 — Colores

- **Paleta propia de 48 colores** en `crates/rnote-compose/src/palette.rs` (sin GTK, con tests) en lugar de modificar las constantes `GNOME_*` de Rnote, que tienen tonos muy parecidos en claridad y no traen oscuros ni rosas.
- **Desplegable en vez de más casillas fijas:** 48 casillas no caben en la barra; un botón ▾ con una cuadrícula ocupa poco y reutiliza el widget `RnColorSetter` existente.
- **Elegir de la paleta sustituye la casilla favorita seleccionada**, igual que hace ya el selector personalizado de Rnote. Así los favoritos se guardan sin código nuevo (Rnote ya los guarda en sus ajustes).

## Fase 1 — Reordenar capas

- **Subir/bajar un nivel respecto a objetos que se solapan** (como Inkscape), porque cambiar el orden respecto a un trazo lejano no tiene efecto visible y parecería que el botón no hace nada.
- **Seleccionar ya no sube al frente** (`store/selection_comp.rs`): era incompatible con "enviar al fondo".
- **Imágenes y PDF pasan a la capa de tinta (`UserLayer(0)`) solo al reordenarlos**, empezando desde el fondo de esa capa. Es un valor que el formato `.rnote` ya admite, así que no hay cambio de formato. Los botones se pusieron en la barra lateral de la selección, junto a "duplicar" y "borrar", con iconos estándar `go-top/up/down/bottom` (no he podido verlos en pantalla).
