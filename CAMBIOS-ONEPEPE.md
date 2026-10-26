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

## Reordenar capas (y PDFs/imágenes por encima de la tinta)

- **Qué hace:** con la herramienta de selección, en la barra lateral de la selección hay 4 botones nuevos: **traer al frente** (flecha a tope arriba), **subir un nivel** (flecha arriba), **bajar un nivel** (flecha abajo) y **enviar al fondo** (flecha a tope abajo). "Subir/bajar un nivel" salta por encima/debajo del siguiente objeto que **se solapa** con la selección (los que están lejos no cuentan, porque moverse respecto a ellos no se vería).
- **Cambio de comportamiento:** antes, en Rnote, al seleccionar o deseleccionar algo se subía al frente. Ahora seleccionar no cambia el orden; si no, "enviar al fondo" se desharía al deseleccionar.
- **PDFs e imágenes:** ya se podían mover, girar y redimensionar con la selección (eso no lo he tocado). Lo nuevo: antes siempre quedaban debajo de la tinta; ahora, al usar cualquiera de los 4 botones con una imagen o página PDF seleccionada, pasa a la capa de la tinta y puedes ponerla encima o entre trazos. Las imágenes que no toques siguen como antes.
- **Deshacer:** cada cambio de orden se puede deshacer (Ctrl+Z).
- **Formato `.rnote`:** sin cambios (se usan valores de capa y orden que el formato ya tenía).
- **Rama:** `feat/capas` (fusionada en `onepepe`).
- **Estado:** compilado · tests del motor ✔ (`cargo test -p rnote-engine chrono_comp`) · **no ejecutado** · lápiz: pendiente.
- **Cómo lo pruebo yo:** compila `onepepe`. Dibuja tres rectángulos rellenos de colores distintos que se solapen (herramienta de formas, color de relleno). Selecciona el de abajo y pulsa "traer al frente"; luego "enviar al fondo"; luego "subir un nivel" dos veces. Deselecciona y comprueba que el orden se mantiene. Importa una imagen o un PDF, ponlo encima de lo escrito con "traer al frente", deshaz con Ctrl+Z, guarda, cierra y vuelve a abrir el archivo para ver que el orden se conserva.
