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

## Regla

- **Qué hace:** en la barra lateral del **pincel** hay un botón nuevo con un icono de regla (también **Ctrl+Alt+R**). Al activarlo aparece una regla gris semitransparente en el centro de la vista, con marcas y el **ángulo** escrito en el centro.
  - **Mover:** apoya el lápiz en el centro de la regla y arrastra.
  - **Girar:** arrastra el círculo azul de un extremo. El ángulo se "engancha" a múltiplos de 15° (0°, 15°, 30°, 45°, 90°…) cuando estás a menos de 1,5°.
  - **Dibujar recto:** empieza el trazo cerca de uno de los bordes largos (a menos de unos 16 píxeles de pantalla, por fuera o por dentro). Todo el trazo queda pegado a la línea del borde, aunque te separes después. Si empiezas lejos, el trazo es libre aunque cruce la regla.
  - Solo afecta al **pincel**; con el borrador, la selección, etc., la regla no hace nada.
  - Cada pestaña tiene su propia regla. No se guarda en el archivo (formato `.rnote` sin cambios).
- **Rama:** `feat/regla` (fusionada en `onepepe`).
- **Estado:** compilado · tests ✔ (`cargo test -p rnote-compose ruler` y `cargo test -p rnote-engine engine::ruler`; incluyen trazos sintéticos temblorosos en 5 ángulos que quedan rectos) · dibujo de la regla comprobado renderizándolo a imagen · **no ejecutado en la app** · lápiz: pendiente.
- **Cómo lo pruebo yo:** compila `onepepe`. Elige el pincel y pulsa el botón de la regla. Comprueba: (1) que puedes moverla arrastrando por el centro y girarla con el círculo azul, y que el ángulo mostrado cambia; (2) que al dibujar a lo largo del borde, empezando muy cerca, sale una línea recta, a 0°, 45° y a un ángulo cualquiera; (3) que si empiezas lejos de la regla el trazo es normal; (4) que deshacer (Ctrl+Z) quita la línea; (5) que al desactivar el botón desaparece. **Dime si la zona de enganche (16 px) te parece grande o pequeña**, y si el ancho de la regla te va bien.

## Tinta a formas

- **Qué hace:** al terminar un trazo con el **pincel**, si se parece claramente a una **línea, círculo, elipse, triángulo o rectángulo** (también girados), se sustituye por la forma limpia, con el mismo color y grosor. Si no encaja con claridad, el trazo se queda como está.
  - **Activar/desactivar:** botón nuevo en la barra lateral del pincel (icono con un garabato, un círculo y un triángulo). Viene **desactivado**.
  - **Sensibilidad:** en el menú de configuración del pincel (rueda dentada) → "Ink to Shape" → "Sensitivity", de 0 (estricto: convierte menos) a 100 (permisivo: convierte más). Por defecto 50.
  - **Deshacer:** Ctrl+Z justo después devuelve tu trazo original; Ctrl+Shift+Z vuelve a poner la forma.
  - Los trazos pequeños en pantalla (diagonal de menos de 50 píxeles al zoom actual) nunca se convierten, para no tocar la escritura.
- **Resultados con trazos sintéticos** (`cargo test -p rnote-compose shaperecognition -- --nocapture`), sensibilidad 50:

  | Forma | ruido bajo | ruido moderado |
  |---|---|---|
  | línea | 100/100 | 100/100 |
  | círculo | 100/100 | 100/100 |
  | elipse | 100/100 | 94/100 |
  | triángulo | 98/100 | 95/100 |
  | rectángulo | 100/100 | 100/100 |

  Ninguna forma se confundió con otra. **Falsos positivos: 0 de 500** trazos que no son formas (bucles cursivos tipo "eeee", "w/m", "s", "8", espirales, "2", "3", "a" con rabito, "D", garabatos). Con sensibilidad 0 se reconocen 148/200 con ruido moderado; con 100, 199/200, en ambos casos sin falsos positivos. **Ojo:** son trazos generados por mí; tu letra real puede comportarse distinto.
- **Rama:** `feat/formas` (fusionada en `onepepe`).
- **Estado:** compilado · tests ✔ (reconocedor + prueba de extremo a extremo en el motor: dibujar círculo → forma → deshacer → trazo original → rehacer) · **no ejecutado en la app** · lápiz: pendiente.
- **Cómo lo pruebo yo:** compila `onepepe`. Con el pincel, activa el botón de tinta a formas y dibuja: una línea, un círculo, una elipse alargada, un triángulo, un rectángulo y un rectángulo girado; luego escribe unas palabras normales, una "O" grande y una "a" grande. Mira qué se convierte. Prueba Ctrl+Z tras una conversión. Cambia la sensibilidad a 20 y a 80 y repite. **Anota cuántas formas acierta y cuántas letras convierte por error**, y dímelo para ajustar los umbrales.
