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

## Matemáticas — primer entregable (informe y extractor)

- **Qué hay:** [`MATEMATICAS-INFORME.md`](MATEMATICAS-INFORME.md) (arquitectura, librerías, datos, qué necesito de ti) y
  [`tools/onepepe-math/`](tools/onepepe-math/README.md): extractor de trazos de `.rnote`, lista de 30 expresiones para
  que las escribas y script para descargar el dataset MathWriting fuera del repositorio. **Aún no hay nada de
  matemáticas dentro de la app.**
- **Rama:** `feat/matematicas-prototipo` (fusionada en `onepepe`).
- **Estado:** tests del extractor ✔ (con un `.rnote` sintético generado por el propio motor) · probado con lápiz: no aplica.
- **Cómo lo pruebo yo:** sigue `tools/onepepe-math/README.md` ("Lo que te pido"). En resumen:
  ```bash
  cd OnePepe/tools/onepepe-math
  python3 -m unittest discover -s tests
  python3 rnote_extract.py ~/ruta/mis_expresiones_1.rnote -o expresiones_1.json --svg svg_1 --labels expresiones.txt
  ```
  y abre los `svg_1/renglon_XX.svg` para ver que cada uno contiene una expresión completa.

## Cuadernos, secciones y páginas

- **Cómo se organiza** (con el explorador de archivos de trabajo que ya tenía Rnote, en la barra lateral):
  - **Cuaderno = espacio de trabajo** (una carpeta). Se añaden en la barra vertical de espacios de trabajo, con su
    nombre, color e icono (esto ya existía).
  - **Sección = subcarpeta.** Menú del explorador (botón de los tres puntos arriba) → "Create new Folder" (ya existía).
    Doble clic en la carpeta para entrar; flecha "arriba" para volver.
  - **Página = archivo `.rnote`.** Doble clic para abrirla en una pestaña (ya existía).
- **Lo nuevo:**
  - **"Create new Page"** en el menú del explorador: crea una página vacía en la carpeta actual con el nombre
    "Page 1", "Page 2"… (puedes cambiarlo) usando tu formato de documento por defecto, y la abre en una pestaña nueva.
  - **"Move to…"** en el menú de cada página o sección (botón ⋮ de la fila): elige la carpeta de destino (empieza en el
    cuaderno) y la mueve allí. No sobrescribe nada: si ya existe algo con ese nombre, avisa y no hace nada.
- **Ya existía y lo he comprobado en el código:** renombrar, duplicar y **borrar con confirmación** (se manda a la
  papelera del sistema, no se borra para siempre: `crates/rnote-ui/src/dialogs/mod.rs:592`).
- **Rama:** `feat/cuadernos` (fusionada en `onepepe`).
- **Estado:** compilado · test ✔ (una página vacía guardada así se vuelve a abrir) · **no ejecutado en la app** · lápiz: no aplica.
- **Cómo lo pruebo yo:** compila `onepepe`. Crea una carpeta `~/Apuntes/Biofisica` y añádela como espacio de trabajo
  (botón "+" de la barra de espacios). Crea dos secciones ("Tema 1", "Tema 2"). Entra en "Tema 1", crea una página
  nueva, escribe algo, guarda (Ctrl+S). Muévela a "Tema 2" con "Move to…", comprueba que la pestaña abierta sigue
  funcionando y guardando, renómbrala y bórrala (debe pedir confirmación). **Pruébalo con una carpeta de prueba, no con
  tus apuntes reales**, hasta que confirmes que funciona.

## Botones del lápiz Wacom (Fase 1b)

- **Ya existía en Rnote** (comprobado en el código): reasignar los botones del lápiz en Ajustes → "Stylus Primary/Secondary
  Button Action", con los modos **"Temporary"** (= mientras pulso), **"Toggle"** (= alterna al pulsar), "Permanent" y
  "Disabled" (`crates/rnote-engine/src/pens/shortcuts.rs`); el **extremo borrador** de las Wacom (GTK lo marca como
  herramienta "Eraser", `crates/rnote-ui/src/canvas/input.rs:480`) con su herramienta elegible en Ajustes →
  "Tool for the eraser action"; y en la barra del borrador los dos estilos: **"Trash Strokes" (trazo entero)** y
  **"Split Strokes" (por área)**.
- **Lo nuevo:**
  - En el desplegable de cada botón hay tres opciones más: **"Eraser (Whole Strokes)"**, **"Eraser (Area)"** y
    **"Selector (Lasso)"**. Usan ese tipo de borrador/selección **aunque en la barra tengas otro**, y al soltar el botón (o
    al volver a pulsarlo en modo "Toggle") se restaura lo que tenías.
  - **Valores por defecto nuevos:** botón 1 → borrador de trazo entero mientras está pulsado; botón 2 → lazo mientras
    está pulsado. **Ojo:** si ya usabas Rnote, tus ajustes guardados mandan; cámbialos en Ajustes con las opciones nuevas.
  - En la barra del borrador, los tres tamaños rápidos son ahora **8, 24 y 64** (pequeño, mediano, grande) en vez de 2, 8 y 16.
- **"Eraser Tool" (Pitfalls del README):** el problema que describe el README es de lápices cuyos **botones** envían el
  evento de "goma" (HP MPP 2.0, Surface Pen), no de las Wacom, cuyos botones envían `BTN_STYLUS`/`BTN_STYLUS2` y cuyo
  extremo borrador envía `BTN_TOOL_RUBBER`. Desde la aplicación no se puede distinguir un "botón que dice ser goma" de
  una goma de verdad: GTK solo entrega "la herramienta es una goma". Alternativa: elegir en Ajustes → "Tool for the
  eraser action" qué hace ese evento (p. ej. borrador o selección). Para remapear el botón a nivel de sistema existen
  `xsetwacom` (solo X11) o, en Wayland, el panel de tableta de GNOME/KDE.
- **Rama:** `feat/botones-lapiz`.
- **Estado:** compilado · tests ✔ (`cargo test -p rnote-engine penholder`: botón 1, botón 2, modo alternar, y que los
  ajustes antiguos sin la opción nueva siguen cargando) · **no ejecutado en la app** · lápiz: pendiente.
- **Cómo verificar qué envía cada botón (`evtest`):**
  ```bash
  sudo apt install evtest     # Fedora: sudo dnf install evtest · Arch: sudo pacman -S evtest
  sudo evtest                 # elige el dispositivo cuyo nombre acaba en "Pen" (o "Stylus")
  ```
  Acerca el lápiz a la tableta y pulsa cada botón. Lo esperado en una Wacom: botón inferior → `BTN_STYLUS`, superior →
  `BTN_STYLUS2`, extremo borrador → `BTN_TOOL_RUBBER` (y la punta → `BTN_TOOL_PEN` + `BTN_TOUCH`). En Rnote,
  `BTN_STYLUS` es "Primary Button" y `BTN_STYLUS2` "Secondary Button". Si un botón envía `BTN_TOOL_RUBBER`, se comporta
  como el extremo borrador (ver el párrafo anterior).
- **Cómo lo pruebo yo:** compila `onepepe`. En Ajustes, comprueba los dos botones (o ponlos en "Eraser (Whole Strokes)" y
  "Selector (Lasso)", modo "Temporary"). Con el pincel: mantén el botón 1 y pasa por encima de un trazo (debe borrarse
  entero); suelta y sigue escribiendo. Mantén el botón 2 y rodea varios trazos (lazo); suelta. Pon el borrador de la
  barra en "Split Strokes" y repite con el botón 1: debe seguir borrando trazos enteros y, al soltar, la barra debe
  volver a "Split Strokes". Prueba el modo "Toggle". Prueba el extremo borrador.

## Matemáticas: modo matemáticas, símbolos limpios, correcciones y resultado del "=" (Fase 3, sin tus datos)

- **Qué hace:**
  - **Modo matemáticas**: botón con "Σ×" en la barra del pincel, o **Ctrl+Alt+M**. Mientras está activo, la vista tiene
    un **borde naranja**. Lo que escribes con el pincel se reconoce tras una pausa de 0,8 s. Al desactivarlo, la tinta
    se queda como está y no se vuelve a analizar.
  - **Sustitución por símbolos limpios** (activada por defecto dentro del modo): cada símbolo reconocido con confianza
    ≥ umbral se redibuja con trazos limpios (fuente Hershey) en la misma posición y tamaño, con tu color y grosor. Son
    trazos de pincel normales (se seleccionan y borran como siempre). Los dudosos se quedan con tu letra.
    **Ctrl+Z** devuelve tu letra original.
  - **Corregir**: en modo matemáticas, **toca** (sin arrastrar) un símbolo reconocido: aparece la lista de candidatas.
    Elige la buena; con "Apply to the whole group" se corrigen todos los símbolos que se le parecen. El símbolo limpio
    se redibuja, la corrección se guarda como ejemplo de tu letra en `~/.local/share/onepepe/math/correcciones.jsonl` y,
    si el cuaderno tiene glosario, el símbolo se añade a él.
  - **Resultado del "="** (desactivado por defecto): en gris junto al "=", con el LaTeX entendido debajo. No es tinta.
    Calcula valores, despeja (`2x+1=7, x=`), simplifica, deriva (`d/dx(...)`), integra, sumatorios y límites; con
    unidades y constantes del glosario, comprobando dimensiones. Si algo no es válido, no muestra nada.
  - **Ajustes**: menú del pincel (rueda dentada) → "Math Mode": sustituir sí/no, umbral de confianza (50–99 %, por
    defecto 95 %) y resultado del "=" sí/no.
  - **Glosario por cuaderno**: archivo `.onepepe-glosario.txt` en la carpeta del cuaderno o sección, p. ej.:
    ```
    k_B = 1.380649e-23 J/K
    N_A = 6.02214076e23 1/mol
    T = 310 K
    \Delta G
    ```
- **Cómo funciona por dentro**: un programa auxiliar en Python (`tools/onepepe-math`, orden `onepepe-math`) que la
  app arranca una vez y con el que habla por JSON. Reconoce por símbolos: agrupa trazos, clasifica (red pequeña
  entrenada con MathWriting), usa el contexto (pares de símbolos frecuentes en 230 000 fórmulas reales), agrupa símbolos
  de forma casi igual, aplica tu glosario y tus correcciones, analiza la estructura y calcula con SymPy y pint.
- **Lo medido aquí (con fórmulas sintéticas hechas con símbolos de escritores que el modelo NO vio, vocabulario
  básico; no con tu letra):**
  - un símbolo suelto: el bueno es el primero el 61 % de las veces y está entre los 5 primeros el 92 %;
  - agrupación de trazos en símbolos: 90 % correcta;
  - con contexto: 66 % de símbolos correctos (86 % entre los 3 primeros);
  - sustitución con umbral 95 %: se sustituye ~50 % de los símbolos y ~84 % de esos se ven bien;
  - estructura (potencias, subíndices, fracciones) con símbolos correctos: 76 % de fórmulas exactas;
  - fórmula completa perfecta de principio a fin: solo ~10 %.
  **Conclusión honesta:** con el modelo genérico, sin tu letra, la sustitución todavía se equivoca demasiado para
  dejarla siempre activa. Lo que más lo mejora son tus correcciones (el clasificador personal las usa al instante) y
  tus 30 expresiones, cuando me las pases.
- **Instalación del programa auxiliar en tu equipo** (una vez):
  ```bash
  cd OnePepe/tools/onepepe-math
  ./instalar.sh --entrenar     # entorno de Python, orden onepepe-math, descarga de MathWriting (2,9 GB) y entrenamiento
  echo '{"id":1,"cmd":"ping"}' | onepepe-math      # debe responder {"ok": true, ...}
  ```
  El modelo (~3 MB) y tus correcciones quedan en `~/.local/share/onepepe/`, nunca en el repositorio.
- **Si usas Flatpak:** la app, dentro de su caja de arena, no ve el Python de tu sistema. La app intenta lanzar
  `flatpak-spawn --host onepepe-math`, pero eso necesita un permiso que **no he añadido** porque permite a la app
  ejecutar programas fuera de la caja de arena: decide tú. Si lo quieres, en tu equipo:
  ```bash
  flatpak override --user --talk-name=org.freedesktop.Flatpak com.github.flxzt.rnote.Devel
  ```
  (se quita con `flatpak override --user --reset com.github.flxzt.rnote.Devel`). Alternativa sin ese permiso: compilar
  OnePepe de forma nativa (sin Flatpak). También puedes indicar otra orden con la variable `ONEPEPE_MATH_CMD`.
- **Ramas:** `feat/matematicas-limpias` (fusionada en `onepepe`).
- **Estado:** compilado · tests ✔ (Rust: modo matemáticas con respuestas simuladas, deshacer, toque y corrección de grupo,
  dibujo del borde y del resultado; Python: 18 tests de protocolo, glifos, estructura, cálculo, unidades y aprendizaje
  de correcciones) · programa auxiliar probado de punta a punta con el modelo real · **no ejecutado en la app** ·
  lápiz: pendiente.
- **Cómo lo pruebo yo:** instala el programa auxiliar (arriba) y compila `onepepe`. Activa el modo matemáticas
  (Ctrl+Alt+M: debe aparecer el borde naranja). Escribe `2+3` y espera un segundo: los símbolos seguros se redibujan
  limpios. Pulsa Ctrl+Z para recuperar tu letra. Toca un símbolo limpio: elige otra candidata, y prueba "Apply to the
  whole group" escribiendo varias veces la misma letra. Activa "Show Result After =" y escribe `2x+1=7, x=`. Crea un
  `.onepepe-glosario.txt` con `T = 310 K` y `k_B = 1.380649e-23 J/K` en la carpeta del documento y escribe `k_B T =`.
  Si no pasa nada, mira si salió el aviso "Could not start the math helper".

## Subíndice y superíndice en el texto (Fase 1c)

- **Qué hace:** en la barra de la herramienta de texto hay dos botones nuevos, **"x²"** y **"x₂"**. Selecciona texto
  escrito a teclado y pulsa uno: se convierte a superíndice o subíndice; pulsarlo otra vez lo devuelve a normal.
- **Cómo:** con los caracteres Unicode de superíndice/subíndice (⁰¹²³…, ₀₁₂…, ⁺⁻⁼⁽⁾, muchas letras y algunas griegas),
  porque el texto de Rnote no tiene un atributo de "elevar". **Limitación:** algunos caracteres no tienen versión
  (p. ej. no hay "q" en superíndice ni casi mayúsculas en subíndice): esos se quedan igual. No cambia el formato `.rnote`.
- **Rama:** `feat/subindices` · **Estado:** compilado · tests ✔ (`cargo test -p rnote-engine scripts`) · no ejecutado · lápiz: pendiente.
- **Cómo lo pruebo yo:** herramienta de texto, escribe `H2O x2 10-3`, selecciona el `2` de H2O → "x₂"; selecciona el
  `2` de x2 → "x²"; selecciona `-3` → "x²"; vuelve a pulsar para deshacer la conversión; Ctrl+Z también funciona.

## Catálogo de símbolos (Fase 1c)

- **Qué hace:** en la barra de la herramienta de texto, junto al botón de emojis, hay un botón **"Ω"**. Abre una
  cuadrícula con griego (minúsculas y mayúsculas), operadores (± × ÷ · ≈ ≠ ≤ ≥ ≡ ∝ → ⇒…), análisis (∂ ∇ ∫ ∮ ∑ ∏ √ ∞ °…),
  conjuntos y lógica (∈ ⊂ ∪ ∩ ∅ ∀ ∃ ℝ…) y física (ℏ Å µ Ω ⊥…). Un clic inserta el símbolo en el texto (o crea un cuadro
  de texto nuevo si no estabas escribiendo).
- **Rama:** `feat/catalogo-simbolos` · **Estado:** compilado · no ejecutado · lápiz: pendiente (no tiene tests: es solo interfaz).
- **Cómo lo pruebo yo:** herramienta de texto, haz clic en el lienzo, pulsa "Ω" y elige ∂, luego α, luego ≈. Comprueba
  que se insertan donde está el cursor.

## Estampar fecha y hora (Fase 1c)

- **Qué hace:** en la barra de la herramienta de texto, un botón con un **calendario** ofrece "Date", "Time" y
  "Date and Time". Inserta `26/10/2026`, `15:42` o `26/10/2026 15:42` en el cursor de texto, o en un cuadro de texto
  nuevo arriba a la izquierda de la vista.
- **Rama:** `feat/fecha-hora` · **Estado:** compilado · tests ✔ (formato: `cargo test -p rnote-engine date_tests`) ·
  no ejecutado · lápiz: pendiente.
- **Cómo lo pruebo yo:** herramienta de texto → botón del calendario → "Date and Time". Comprueba la hora de tu equipo.

## Viñetas, listas numeradas y casillas (Fase 1c)

- **Qué hace:** en la barra de la herramienta de texto, tres botones nuevos actúan sobre el **párrafo donde está el
  cursor**: **viñeta** ("• "), **lista numerada** ("1. ", y si el párrafo de arriba es "3. …", pone "4. ") y
  **casilla** ("☐ "; al volver a pulsar, se marca "☑ " y se desmarca "☐ "). Pulsar otra vez viñeta o numerada la quita;
  cambiar de un estilo a otro lo sustituye.
- **Limitación:** al pulsar Intro no se continúa la lista sola (hay que pulsar el botón en cada párrafo). Son caracteres
  normales del texto: no cambia el formato `.rnote`.
- **Rama:** `feat/listas` · **Estado:** compilado · tests ✔ (`cargo test -p rnote-engine list_tests engine_tests`) ·
  no ejecutado · lápiz: pendiente.
- **Cómo lo pruebo yo:** herramienta de texto: escribe tres líneas (Intro entre ellas). Pon el cursor en la primera →
  numerada; en la segunda → numerada (debe salir "2."). En la tercera → casilla, y pulsa casilla otra vez para marcarla.
