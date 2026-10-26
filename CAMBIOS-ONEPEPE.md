# Cambios de OnePepe y cómo probarlos

Estado de cada función: **compilado** (compila), **tests** (pasa tests automáticos en la nube), **lápiz** (probado por ti con la tableta; solo tú puedes marcarlo).

## Cómo compilar y ejecutar una rama en tu equipo

Hazlo una sola vez:

```bash
git clone https://github.com/merinodpepe/OnePepe.git
cd OnePepe
git submodule update --init --recursive
./setup-deps.sh --check      # dice qué falta y si tu sistema es demasiado viejo
```

Rnote necesita GNOME 50 (GTK ≥ 4.22, libadwaita ≥ 1.9, glib ≥ 2.88). Si `--check` dice "DEMASIADO VIEJA", usa **Flatpak** (recomendado):

```bash
./setup-deps.sh --flatpak    # instala el entorno de compilación de Flatpak (solo la primera vez)
```

Después, para probar una rama concreta (cambia `onepepe` por la rama que quieras, por ejemplo `feat/colores`):

```bash
cd OnePepe
git fetch origin
git checkout onepepe
git pull
flatpak-builder --user --install --force-clean _flatpak_app build-aux/com.github.flxzt.rnote.Devel.yaml
flatpak run com.github.flxzt.rnote.Devel
```

La primera compilación tarda bastante (descarga y compila todas las dependencias de Rust); las siguientes son más rápidas.

Si tu sistema sí tiene las versiones necesarias, la vía nativa es:

```bash
./setup-deps.sh
meson setup --prefix=/usr _mesonbuild
meson compile -C _mesonbuild
sudo meson install -C _mesonbuild
rnote
```

Para ejecutar los tests del motor (no necesitan GTK):

```bash
cargo test -p rnote-engine
cargo test -p rnote-compose
```

---

## Hojas infinitas (comprobación)

- **Qué hace:** no cambia nada de la app. Añade tests que comprueban que el modo infinito (el que viene por defecto) amplía el documento al desplazarte y al hacer zoom.
- **Rama:** `feat/hojas-infinitas` (fusionada en `onepepe`).
- **Estado:** tests ✔ · lápiz: pendiente.
- **Cómo lo pruebo yo:**
  ```bash
  cargo test -p rnote-engine --test infinite_layout
  ```
  Con el lápiz: abre un documento nuevo; en el menú del documento comprueba que el formato de página es "Infinito". Escribe, aléjate mucho en las cuatro direcciones arrastrando con dos dedos o con la herramienta de mover, haz zoom fuerte hacia fuera y hacia dentro, y escribe lejos del origen. Mira que nunca aparezca un borde que te impida seguir y que lo escrito lejos no desaparezca al volver.

## Más colores

- **Qué hace:** junto a las 9 casillas de colores favoritos aparece un botón con una flecha (▾). Al pulsarlo se abre una paleta de **48 colores**: una fila de grises (de blanco a negro) y 8 tonos (rojo, naranja, amarillo, verde, verde azulado, azul, morado, rosa) en 5 intensidades, de muy claro a muy oscuro. El selector personalizado (botón de la rueda de colores) sigue igual.
- **Colores favoritos:** ya existían en Rnote: las 9 casillas se guardan entre sesiones. Si tienes una casilla seleccionada y eliges un color de la paleta (o del selector personalizado), esa casilla pasa a guardar ese color.
- **Rama:** `feat/colores` (fusionada en `onepepe`).
- **Estado:** compilado (comprobación de tipos de la UI) · tests de la paleta ✔ · **no ejecutado** (no hay pantalla en la nube) · lápiz: pendiente.
- **Cómo lo pruebo yo:** compila la rama `onepepe` (ver arriba) y además `cargo test -p rnote-compose palette`. En la app: elige el lápiz, pulsa la flecha ▾ al lado de los colores, elige un color oscuro y uno claro y escribe con cada uno. Comprueba que la paleta se cierra al elegir, que el color de los trazos es el elegido y, si tenías una casilla favorita seleccionada, que ha cambiado a ese color y se mantiene al cerrar y abrir la app.
