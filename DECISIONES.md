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

## Fase 1 — Regla

- **El trazo se pega al borde solo si empieza cerca de él** y entonces se queda pegado hasta levantar el lápiz. Si cada punto decidiera por separado, un trazo que roza el borde quedaría a trozos recto y a trozos libre.
- **Mover/girar con el propio lápiz** (centro = mover, círculo = girar), porque con el pincel seleccionado en Rnote el dedo suele servir para desplazar el lienzo. Giro con enganche a múltiplos de 15°.
- **La regla vive en el motor, por pestaña, sin guardarse** en el `.rnote` (campo con `#[serde(skip)]`), para no cambiar el formato. La geometría está en `crates/rnote-compose/src/ruler.rs` (sin GTK, con tests); la integración en `crates/rnote-engine/src/engine/ruler.rs`, interceptando los eventos del lápiz en `Engine::handle_pen_event` antes de pasarlos al pincel.
- **Atajo Ctrl+Alt+R**, porque Ctrl+R y Ctrl+Shift+R ya están ocupados en Rnote. Icono propio sencillo (`onepepe-ruler-symbolic.svg`).

## Fase 2 — Tinta a formas

- **Algoritmo geométrico propio, sin modelos** (`crates/rnote-compose/src/shaperecognition.rs`): remuestreo y suavizado; si el trazo está abierto solo puede ser una línea (ajuste por mínimos cuadrados totales); si está cerrado, se compara un ajuste de elipse por ejes principales con un polígono obtenido por Douglas-Peucker cuyas esquinas se recalculan cruzando rectas ajustadas a cada lado. Gana el de menor error relativo a su tolerancia. Es rápido, predecible y fácil de ajustar.
- **Rechazo de escritura:** (1) tamaño mínimo en pantalla de 50 px; (2) un trazo cerrado debe dar exactamente una vuelta (giro neto ≈ 360° y giro total acotado), lo que descarta "8", "e", espirales…; (3) si tras cerrar el trazo sigue y se aleja del contorno (el rabito de una "a"), no es una forma. Preferí perder alguna forma dudosa antes que convertir letras.
- **Cuadriláteros:** solo se convierten si sus ángulos están cerca de 90° (rectángulo, también girado o en rombo-cuadrado). Un paralelogramo o trapecio se deja como está, porque no estaba en tu lista.
- **El círculo/elipse se decide por la proporción de ejes** (> 0,82 → círculo).
- **Deshacer:** el trazo original se guarda en el historial antes de sustituirlo, así que un Ctrl+Z lo recupera. No hace falta cambiar el formato: la forma se guarda como `ShapeStroke`, que ya existe en `.rnote`.
- **Los ajustes (activado, sensibilidad) van en la configuración del pincel**, que Rnote guarda en los ajustes de la app, no en los documentos. Viene desactivado por defecto.
- **La forma usa estilo liso** con el color y grosor del pincel (también con el rotulador, conservando su transparencia).

## Fase 3 — Matemáticas (primer entregable)

- **Reconocimiento por símbolos y no por imagen completa**, para poder dar candidatas por símbolo, corregir por grupos y
  aprender de tu letra. Los modelos "imagen → LaTeX" quedan como posible hipótesis extra más adelante.
- **Programa auxiliar persistente** (se lanza una vez y habla por JSON línea a línea) en vez de uno por petición, para
  no cargar el modelo en cada "=".
- **La expresión de SymPy se construye desde el árbol de símbolos**, no desde el LaTeX, porque `parse_latex` solo
  entendió 19 de las 30 expresiones de prueba.
- **MathWriting como dataset base** porque es el único grande con trazos (no imágenes) que se puede descargar desde
  aquí; su licencia (CC BY-NC-SA) permite uso personal. Se descarga fuera del repositorio con un script.
- **Tus datos no van al repositorio:** `.gitignore` en `tools/onepepe-math/` y envío por chat o Google Drive.
- **El extractor solo usa la biblioteca estándar de Python**, para que lo puedas ejecutar sin instalar nada. Separa
  renglones por posición vertical; el `.rnote` no guarda tiempos por punto, solo el orden de los trazos.
- **Fixture sintético** (`tests/fixtures/dos_mas_tres.rnote`, 1,7 KB) generado por el motor con
  `crates/rnote-engine/tests/rnote_fixture.rs`, para que el test use un archivo con el formato real.

## Fase 4 — Cuadernos

- **No reescribí el explorador como un árbol de tres niveles.** El explorador de Rnote ya navega carpetas (entrar con
  doble clic, salir con la flecha), así que cuaderno → sección → página funciona sin cambiar su estructura. Si
  prefieres ver el árbol desplegable completo, es un cambio mayor (sustituir la lista por un `TreeListModel`): dímelo.
- **"Nueva página" escribe un `.rnote` vacío** con el formato de documento por defecto de tus ajustes y lo abre. El
  nombre propuesto es el primer "Page N" libre (en inglés porque la app no está traducida al español para textos nuevos).
- **"Mover a…" usa el selector de carpetas del sistema** que empieza en el cuaderno, en lugar de arrastrar y soltar:
  es más simple y más difícil de usar por error. Usa un simple renombrado de ruta (igual que "renombrar" de Rnote), así
  que no copia ni borra nada; si el destino ya existe, no hace nada.
- **Borrar ya pedía confirmación** y manda a la papelera; no lo he tocado.

## Fase 1b — Botones del lápiz

- **Variantes en lugar de herramientas nuevas:** las opciones "Eraser (Whole Strokes)", "Eraser (Area)" y
  "Selector (Lasso)" son un campo nuevo `variant` de la acción del atajo (en los ajustes de la app, no en los `.rnote`).
  Al activarse cambian el estilo del borrador/selección y lo restauran al terminar. Los ajustes antiguos sin ese campo
  cargan como antes (test).
- **Los valores por defecto nuevos solo afectan a instalaciones nuevas**: no sobrescribo los ajustes guardados.
- **Extremo borrador / "Eraser Tool":** no se puede arreglar desde la app para lápices cuyos botones envían eventos de
  goma, porque GTK no distingue ese caso; lo documenté con la alternativa. Las Wacom no tienen ese problema.
- **Tamaños del borrador por área 8/24/64**: los de Rnote (2/8/16) son los del pincel y se quedan cortos para borrar.

## Fase 3 — Matemáticas sin datos del usuario

- **Clasificador pequeño evaluado con numpy** (red de 2 capas, ~3 MB) en vez de PyTorch/ONNX: se entrena en un par de
  minutos en CPU con scikit-learn y en tu equipo solo hace falta numpy. MathWriting solo trae 6 423 símbolos sueltos
  etiquetados (~30 por clase), así que se entrena con deformaciones aleatorias.
- **Clase "trozo"** (trazos sueltos de símbolos de varios trazos y parejas de símbolos juntadas) para que la agrupación
  no parta el "=" ni junte símbolos vecinos: subió la agrupación del 81 % al 90 %.
- **20 % de los símbolos reservado** (por identificador) que el clasificador nunca ve; las pruebas de fórmulas usan solo
  esos, para no inflar los números.
- **Vocabulario básico por defecto** (cifras, letras, griegas, operadores y símbolos de análisis); los ~100 símbolos
  exóticos de MathWriting (ℕ, ⊨, ℘…) se ignoran.
- **Contexto por bigramas + Viterbi** (pesos elegidos midiendo en fórmulas sintéticas) y **grupos por forma con
  similitud ≥ 0,85** de la representación interna (medido: agrupa el 64 % de parejas del mismo símbolo de una misma
  persona y ninguna de símbolos distintos en 400 pruebas).
- **La sustitución usa la confianza tras el contexto** y un umbral alto (95 %) por defecto. El modo matemáticas viene
  apagado, así que nada cambia hasta que lo activas.
- **El motor de Rnote no sabe matemáticas**: guarda el estado (cola de trazos, símbolos reconocidos para corregir,
  resultados) y aplica las respuestas; todo lo demás está en el programa auxiliar. La relación símbolo ↔ trazos se
  recuerda solo durante la sesión (no hay archivo oculto junto a la página hasta que lo confirmes; ver
  `MATEMATICAS-INFORME.md`, 0.1).
- **Corregir tocando**: en modo matemáticas, un toque sin arrastrar sobre un símbolo reconocido abre las candidatas en
  vez de dibujar un punto. Escribir encima de un símbolo reconocido empieza como un toque: es el precio de no añadir
  otro botón.
- **Flatpak:** no añadí el permiso `org.freedesktop.Flatpak` al manifiesto (permite salir de la caja de arena); queda
  documentado como paso opcional que decides tú.
- **Créditos de las fuentes Hershey** en `tools/onepepe-math/onepepe_math/glyphs.py`: A. V. Hershey (U.S. National
  Bureau of Standards) y James Hurt (formato de los datos); se usan a través del paquete `Hershey-Fonts` (MIT).
