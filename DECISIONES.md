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
  guarda en un archivo oculto junto a la página (confirmado por ti; ver la sección siguiente).

## Matemáticas — archivo oculto junto a la página

- **Nombre `.<página>.rnote.onepepe.json`** en la misma carpeta: oculto (el explorador de Rnote no lo muestra) y fácil
  de asociar a su página.
- **Identificación de los trazos por su clave interna** (Rnote conserva las claves al guardar y abrir, comprobado con un
  test) **más su caja**: si al abrir no coinciden, ese símbolo se ignora. Así un archivo oculto desfasado nunca cambia
  la página, como mucho se pierde la corrección de esos símbolos.
- **Se escribe al guardar** (también con el autoguardado) y **se lee al abrir**; acompaña a la página al renombrar,
  mover, duplicar o borrar desde el explorador de OnePepe. Las carpetas enteras se mueven con todo su contenido.
- **Corregir tocando**: en modo matemáticas, un toque sin arrastrar sobre un símbolo reconocido abre las candidatas en
  vez de dibujar un punto. Escribir encima de un símbolo reconocido empieza como un toque: es el precio de no añadir
  otro botón.
- **Flatpak:** al principio no añadí el permiso `org.freedesktop.Flatpak` al manifiesto (permite salir de la caja de
  arena). **Lo añadiste tú explícitamente** después ("añade el permiso org.freedesktop.Flatpak al manifiesto de
  Flatpak"): está en `build-aux/com.github.flxzt.rnote.Devel.yaml`, rama `feat/flatpak-matematicas`, con un comentario
  que explica el riesgo.
- **Créditos de las fuentes Hershey** en `tools/onepepe-math/onepepe_math/glyphs.py`: A. V. Hershey (U.S. National
  Bureau of Standards) y James Hurt (formato de los datos); se usan a través del paquete `Hershey-Fonts` (MIT).

## Fase 1c — Texto, símbolos y utilidades

- **Subíndice/superíndice con caracteres Unicode** (², ₂, ⁻¹…): el texto de Rnote no tiene atributo de elevar y
  añadirlo cambiaría el formato `.rnote` y el dibujo del texto. Limitación aceptada: hay letras sin versión Unicode.
  Botones en vez de atajos de teclado porque Ctrl+= ya es "acercar".
- **Listas como prefijos de párrafo** ("• ", "1. ", "☐ "/"☑ "): son caracteres normales, sin cambio de formato. No
  continúan solas al pulsar Intro (habría que tocar el manejo del teclado del editor de texto).
- **Fecha y hora en formato español** (dd/mm/aaaa, HH:MM) con la hora local del equipo.
- **Catálogo de símbolos** como un desplegable junto al de emojis, reutilizando la inserción de texto de Rnote.
- **No hice** estilos rápidos ni recorte de pantalla (opcionales en el plan).

## README de OnePepe

- **README propio en español** con el nombre, el logo (ahora el de Pepe Flock, en `misc/pepeflock/`) y las funciones
  de OnePepe. Se conserva, en "Licencia" y "Créditos", el aviso de que se basa en Rnote y el copyright de sus autores:
  la licencia GPL obliga a mantener esos avisos.
- **Sin capturas de pantalla** por ahora: las de Rnote mostrarían otro programa y en la nube no puedo hacer capturas de
  OnePepe. Se pueden añadir cuando lo ejecutes en tu equipo.

## Nombre OnePepe dentro de la app (marca Pepe Flock)

- Solo cambia el **nombre visible** (`app_name_capitalized = 'OnePepe'` en `meson.build`): título de la ventana,
  "Acerca de OnePepe", menú de aplicaciones y metadatos. El nombre interno (`rnote`) y el identificador
  (`com.github.flxzt.rnote`) **no cambian**, para no perder ajustes ni romper Flatpak, la ruta de datos o las
  traducciones. Cambiarlos sigue en `PLAN-PENDIENTE.md` (sección 3).
- "Acerca de": desarrollador "Pepe Flock", enlaces a `merinodpepe/OnePepe` y un apartado "Rnote" con los autores de
  `AUTHORS` (crédito de la base, GPL). "Donar" se quitó del menú el 2026-10-29 (llevaba a Rnote).
- El nombre del repositorio en GitHub no se cambia: no hace falta y cambiarlo exige actualizar enlaces y clones.
- (Sustituido el 2026-10-29, ver «`main` = OnePepe y sin Rnote a la vista».)

## Logos de Pepe Flock

- Los logos que subiste se movieron de la raíz a `misc/pepeflock/` (`git mv`, sin perder historial).
- `onepepe_icono.svg` es ahora el icono de la app (`crates/rnote-ui/data/icons/scalable/apps/rnote.svg` y
  `rnote-devel.svg`; mantienen el nombre de archivo porque meson los instala con el identificador de la app).
- El README usa `onepepe_lateral.png` (OnePepe en grande, Pepe Flock debajo).
- Se borró el logo provisional `misc/onepepe/logo.svg`.
- El icono monocromo pequeño (`icons/symbolic/apps/rnote-symbolic.svg`) y los `.ico` de Windows siguen siendo los de
  Rnote: falta una versión de una sola tinta del logo.

## Tercera tanda: cinta tipo OneNote y mejoras sencillas (28/10/2026)

- **Cinta en código, sin archivos `.ui`** (`crates/rnote-ui/src/ribbon.rs`): un error en un `.ui` solo se ve al
  ejecutar, mientras que en código lo comprueba el compilador. La cinta usa las acciones de la ventana (`win.*`) que ya
  existían y añade las nuevas. Va encima de las pestañas de páginas y no sustituye a la barra de herramientas de Rnote
  (la cinta es para encontrar las cosas; la barra lateral de cada herramienta tiene los ajustes finos).
- **Probar ejecutando en la nube:** se compiló una copia aparte contra GTK 4.14 / libadwaita 1.5 (las del sistema en
  la nube), quitando de los `.ui` una propiedad que esa versión no conoce, y se usó con `xdotool` en una pantalla
  virtual. Esa copia no se sube. Así se encontraron y arreglaron 4 fallos que solo se ven usando la app:
  - Los botones quitaban el foco del teclado al texto.
  - "Título 1" perdía el tamaño (`replace_attr_for_range` descarta los demás atributos del rango).
  - Quitar un bolígrafo favorito no hacía nada (ventanita anclada dentro del botón).
  - La cinta crecía en vertical.
- **Bolígrafos favoritos** en `~/.config/onepepe/boligrafos.json` y no en los ajustes de GSettings: añadir una clave al
  esquema obliga a reinstalar el esquema, y una clave que falta hace que la app se cierre al arrancar.
- **Tabla dibujada** (rejilla de líneas) en lugar de tabla de texto: en una app de escritura a mano se rellena con el
  lápiz, y es barata. Una tabla de texto de verdad queda en el plan.
- **Plantillas fijas** (Cornell, clase, problemas, título y fecha) dibujadas en la hoja que se ve. Plantillas propias,
  en el plan.
- **Recorte de pantalla** por el portal del escritorio (`org.freedesktop.portal.Screenshot`) con D-Bus de GIO, sin
  crates nuevas (el Flatpak compila sin red, y una crate nueva obligaría a regenerar sus fuentes).
- **Copias de seguridad** en `~/.local/share/onepepe/copias/`, fuera de los cuadernos, para no llenarlos de archivos.
  Como mucho una cada 20 minutos (el guardado automático es frecuente) y 10 por página.
- **Exportar ajustado** no cambia la página, solo el PDF, y nunca agranda (escala ≤ 1). Avisa si la escala baja de 0,5.
- **Diseño:** morado #7719AA (OneNote y Pepe Flock) en la barra de título y en la banda de pestañas, con la pestaña
  activa como lengüeta clara. Los colores se fijan en CSS (`data/style.css`) con clases propias, para no tocar el
  resto del tema. Variante para el modo oscuro.
- **Barra lateral:** deja de listar archivos de texto (`.md`, `.txt`, código). Rnote los mostraba porque se pueden
  abrir; en un cuaderno tipo OneNote solo estorban.
- **Traducción:** cadenas en inglés con `gettext` (como el resto de Rnote) y su traducción en `es.po`. Se quitaron 7
  entradas obsoletas de `es.po` que repetían textos que vuelven a usarse.

## Diseño estilo Fluent (28/10/2026)

- **Referencias:** el rediseño Fluent de OneNote (cinta simplificada, pestañas con subrayado, menos espacio vertical) y,
  como apps libres de escritura a mano, Xournal++ y Saber.
- **Colores dentro de la cinta:** el selector de colores de Rnote se saca de la capa flotante y se mete en la cinta,
  en lugar de copiarlo, para no tener dos selectores sincronizados.
- **La barra de herramientas inferior** repite la cinta: se oculta mientras la cinta está visible y vuelve si la ocultas.
- **Todo el diseño va en CSS** con clases propias (`onepepe-*`) y con la prioridad de la aplicación; no toca el tema
  del sistema.

## Cortar tu letra en símbolos y reentrenar desde la app (28/10/2026)

- **Umbrales de distancia calibrados: no se quedan** (no mejoran con tus expresiones). Quedan como parámetros, sin
  script.
- **Corte forzado** para sacar ejemplos de tus páginas etiquetadas: con 1–4 trazos seguidos por símbolo y exactamente
  los símbolos esperados. Las etiquetas pueden tener algo de ruido (p. ej. el punto del «·» pegado al «5»), pero
  medido sobre expresiones no vistas mejora mucho.
- **«Trozos» en el almacén** con `kind: "junk"`: entran en el reentrenamiento (clase «trozo»), no en el kNN.
- **Reentrenar compara con tus ejemplos más recientes reservados** (30 %): así la comparación es honesta. Esos
  ejemplos siguen en el kNN y entran en el siguiente reentrenamiento.
- **Cancelar envía SIGTERM y no un kill:** `flatpak-spawn` reenvía la señal al programa del equipo, que para también el
  entrenamiento. Además, el proceso de entrenamiento muere solo si muere su padre (`PR_SET_PDEATHSIG`).
- **Tiempos de los trazos en el archivo oculto** (versión 2, que sigue leyendo la 1), no en el `.rnote`: no se rompe
  la compatibilidad con Rnote ni con tus archivos.
- **Pestaña Matemáticas** en la cinta: Dibujar ya no cabía en pantallas de 14".
- **«Unir con el de al lado»** junta con el símbolo más cercano por cualquier lado: los trozos de un «=» o de una «i»
  quedan uno encima del otro, no a la derecha.

## Diseño según tu guía (29/10/2026)

Detalle, capturas y contrastes en [`DISENO.md`](DISENO.md).

- **Una sola tabla de colores** (`theme.rs`). De ella sale el CSS de claro, oscuro y alto contraste, y los colores
  de libadwaita.
  - Se usa `@define-color` y no `var(--…)`: así funciona en GTK 4.14 (donde lo pruebo) y en 4.22. Las variables de
    libadwaita 1.6+ solo se escriben con GTK 4.16 o más nueva.
- **Lienzo oscuro solo en pantalla**, con una matriz de color que conserva el tono. Ni el `.rnote` ni lo exportado
  cambian.
- **Atajos:**
  - Ctrl+1…5 son los bolis favoritos; las herramientas pasan a Alt+1…6.
  - Ctrl+Alt+1/2/0 son los estilos de texto; los colores 1 y 2 pierden su atajo (del 3 al 9 siguen).
- **Favoritos:** 5 huecos. Una lista antigua más larga se guarda entera en `boligrafos.anterior.json`.
- **Estilos de texto:** un texto «tiene» un estilo si coincide en tamaño y grosor. No se guarda nada aparte, y vale
  con páginas de Rnote. El interlineado de la guía no se aplica: el `.rnote` no lo guarda por párrafo.
- **Árbol y subpáginas:** el orden y quién es subpágina de quién van en un archivo oculto por carpeta,
  `.onepepe-indice.json`. Los archivos no se mueven ni se renombran.
  - Si borras ese archivo, se vuelve al orden por nombre (con números en orden natural: Tema 2 antes que Tema 10).
- **Regla:**
  - Infinita = una línea que cruza todo el lienzo; gira desde el centro, y con Mayús de 15° en 15°.
  - Su estado va en el archivo oculto de cada página, no en el `.rnote`.
- **Renglones:**
  - Tipo, separación y colores son el fondo del `.rnote`, como en Rnote.
  - El grosor y la línea de margen no existen en el formato: van en el archivo oculto. Se ven en pantalla y al
    exportar, pero no si abres la página en Rnote.
  - «Toda la sección» reescribe las demás páginas de la carpeta. Antes guarda una copia de seguridad de cada una y
    escribe por archivo temporal, para no dejar nunca una página a medias.
- **Ajustes:** se mantienen los ajustes inmediatos de Rnote. «Revertir» vuelve a como estaban al abrirlos o al último
  «Aplicar»; cerrar sin aplicar los deja puestos.

## Regla, zoom y mano (29/10/2026)

- **Regla:** el ángulo es libre por defecto. Antes se pegaba a múltiplos de 15° cuando estaba a menos de 1,5°; ahora
  eso solo pasa con Mayús.
  - En modo infinito, al empezar a tocarla, su centro se recoloca en el punto de la regla más cercano al centro de la
    vista (en múltiplos de 80 px, para que los números no bailen). Así el círculo de giro siempre está a mano.
- **Zoom:**
  - Límites del 5 % al 1600 %; con «Zoom vectorial», hasta el 100 000 %. Es un ajuste de la app, no del documento.
  - Para que funcione sin agotar la memoria:
    - las imágenes de cada trazo se recortan a la parte visible (también las de cada tramo);
    - ninguna imagen pasa de 4096 px de lado;
    - la tesela del fondo tiene la resolución limitada;
    - muy alejado, la cuadrícula no se dibuja.
  - El zoom guardado en un `.rnote` puede pasar del 600 %; Rnote lo recorta al abrirlo, no se rompe nada.
- **Mano:** es la herramienta «desplazar vista» que ya tenía Rnote (dentro de Herramientas), con un botón y el atajo
  Alt+7, siguiendo a Alt+1…6.

## Flechas y «;» en matemáticas (29/10/2026)

- `→ ⇒ ⇔ ;` separan pasos o expresiones. Para el resultado del «=» solo cuenta el último paso: si es solo una letra
  (`→ x =`), se despeja esa letra de la ecuación anterior.
- No se comprueba todavía si cada paso se sigue del anterior. Sería posible con SymPy y queda como idea en
  `PLAN-PENDIENTE.md`.
- `⟶` es la misma flecha que `→` (alias), y `⟹` la misma que `⇒`.

## `main` = OnePepe y sin Rnote a la vista (2026-10-29)

Petición del usuario: «sacar todas las referencias grandes a Rnote del repositorio; el `main` siendo de Rnote me
molesta, ya le hago referencia en mi md». Opciones elegidas: «main = OnePepe» y «lo visible».

- **Ramas:** el Rnote original (`6bbc9b2`) se guarda en la rama **`rnote-original`**. `main` avanza hasta `onepepe`
  con `git push origin onepepe:main` (avance rápido, sin force push; ninguna rama borrada). Desde ahora, tras cada
  fusión en `onepepe` se avanza también `main`. La antigua regla «no toques `main`» queda sustituida.
- **Borrado (sigue en `rnote-original`):** `CONTRIBUTING.md`, `MAINTAINING.md`, `docs/` (arquitectura y
  compilación en macOS/Windows), `.github/` (donaciones, plantillas de incidencias y los flujos de CI, nightly,
  instalador de Windows y traducciones de Rnote), `build-aux/deploy-remote.sh` y las capturas de Rnote
  (`crates/rnote-ui/data/screenshots/`). Los flujos de CI se quitaron también: eran los de Rnote (publicaban
  nightlies, instaladores…) y no se han probado con OnePepe; se pueden recuperar de `rnote-original` si se quiere CI.
- **Reescrito:** `BUILDING.md` (guía corta en español). Metadatos de tienda (`app.metainfo.xml.in.in`): resumen,
  capturas de OnePepe (`misc/onepepe/diseno/`, enlazadas desde `main`), una sola versión 0.15.0 de OnePepe en lugar
  del historial de Rnote, sin contacto ni enlace de donación de Rnote. `.desktop`: comentario y palabras clave.
  Tipo de archivo: «OnePepe notes file». «Acerca de»: el mismo resumen. Menú: sin «Donar». Icono monocromo de la
  app: el logo de OnePepe. `es.po` al día.
- **Se queda (GPL y compatibilidad):** `LICENSE`, `AUTHORS`, cabeceras, el apartado «Rnote» de créditos en «Acerca
  de», la nota de crédito del README y los comentarios del código. El id `com.github.flxzt.rnote`, el binario
  `rnote`, los crates y la extensión `.rnote` no cambian (ver «Nombre»).
- **Se queda (lo necesita meson):** `build-aux/inno_build.py` y `rnote_inno.iss.in` (meson los pide aunque solo
  se usan en Windows) y `dist-vendor.sh`.


## Arreglos del modo matemáticas y plan tipo MyScript (2026-10-30)

- **Toque o trazo (fallo 1):** los eventos que empiezan sobre un símbolo se guardan hasta saber qué son; si el lápiz
  se mueve más de 6 px de pantalla se dan al pincel todos juntos (el trazo sale entero desde su primer punto). El
  toque cuenta sobre la tinta (las cajas de los trazos), no sobre la caja del símbolo. Un símbolo con todos sus
  trazos en la papelera no cuenta (toque, contexto, archivo oculto) y vuelve con deshacer.
- **Zoom (fallo 2):** medido con un test, el suavizado del pincel da la misma forma a cualquier zoom salvo el final
  del trazo con zoom ≥ 5000 % (umbral fijo de 0,01 unidades → 0,0001). En la app de la nube, al 1600 %, el
  reconocimiento leía bien los símbolos, pero un renglón pequeño junto a letra grande recibía de contexto los
  símbolos grandes («2+3=3»). Ahora el contexto y los renglones solo juntan símbolos de tamaño parecido (hasta 4
  veces, así entran exponentes y rayas de fracción) y cercanos de lado. Después, al 1600 % «2+3=» salió bien 3 de
  4 veces (una vez el «3» como «y»). Queda una diferencia pequeña en la tinta que llega: el mismo gesto al 100 % y al
  1600 % difiere un ~12 % de la altura del símbolo (dos veces al mismo zoom, un 2–5 %). No viene de Python (da lo
  mismo a cualquier escala) ni del suavizado (medido); no se encontró la causa sin datos de un lápiz real.
- **i y j (fallo 4), «si no mejora, no se queda»:** con 400 fórmulas de MathWriting con i o j, unir el punto cortado
  aparte mejora la i (141 → 147 de 358) y no cambia lo demás: se queda. La regla «la j baja de la línea base» empeoró
  la i (147 → 88 con la línea del renglón; 147 → 123 con la de los vecinos, porque la i de un subíndice también baja)
  y se quitó.
- **Funciones (fallo 7):** tras una función, solo es subíndice lo que queda por debajo de la línea base de la palabra
  (sin contar la «g» de log). Los nombres en español (sen, tg…) se guardan con el nombre de LaTeX (\sin, \tan). La
  unión tolerante («1n» → ln) solo si la letra estaba entre las candidatas; con símbolos que vienen de la app sin
  candidatas, la app ahora las envía.
- **Una sola orden «page»** para el «=», el «?», las variables y la revisión de pasos: la app envía todos los
  renglones reconocidos tras cada cambio (400 ms después) y sustituye las anotaciones. Así se recalcula al corregir,
  borrar, deshacer o cambiar opciones, sin estado en Python. El «=» ya no se calcula en `recognize`.
- **Renglones:** se agrupan por altura en el motor (cajas que se solapan, con un margen del 30 % del tamaño típico).
- **Variables:** valen en toda la página; cada renglón usa la última definición de encima y, si no hay, la primera
  de la página. Con unidades vía pint (un solo registro de unidades). Una «x» entre dos números es «por».
- **Revisar pasos:** equivalencia con valores al azar (expresiones) o mismas soluciones (ecuaciones de una variable;
  si no, proporcionales). Errores típicos probados sobre el paso anterior: signo, distribuir, pasar sin cambiar el
  signo, dividir mal, potencia o raíz de una suma, sumar fracciones. Solo se marca lo que no es equivalente.
- **Tachar:** zigzag = 4 o más vueltas en alguna dirección (una «w» da 3, una «z» 2), y tiene que cubrir la mayor
  parte de trazos anteriores y tocar su tinta. Por defecto solo en modo matemáticas. Un solo deshacer: se reemplaza
  la entrada del historial del zigzag (`update_latest_history_entry`).
- **Gráficas y fracciones:** la gráfica son trazos normales (no cambia el `.rnote`, se ve en Rnote); 10 veces la
  altura de los símbolos. La forma elegida de una fracción se guarda en el archivo oculto (campos nuevos con valor
  por defecto: los archivos de antes se siguen leyendo).
- **Formato `.rnote`: sin cambios.** Lo nuevo va a los ajustes de la app (`variables`, `check`, `scratch_out`,
  `scratch_out_sensitivity`, con valor por defecto) y al archivo oculto (`kind`, `formats`, `format` y `keys` de cada
  anotación).

## Cierre al tocar el «?» (versión 1.3.3, 2026-10-31)

- Causa (informe del usuario: «RefCell already borrowed», `canvas/mod.rs`, al abrir una ventanita): `if let
  Some(..) = canvas.engine_ref().math_overlay_at(pos) { explain_overlay(..) }` mantiene el préstamo del motor
  durante todo el bloque. Al abrir la ventanita cambia el foco y un manejador del lienzo pide `engine_mut()`.
- Arreglo: guardar el resultado en una variable antes (`let overlay = ...;`) y abrir las ventanitas en
  `idle_add_local_once`. Regla para el futuro en `CONTEXTO-IA.md` §14.

## Informe de fallos (versión 1.3.2, 2026-10-31)

- El usuario vio que OnePepe se cerraba al escribir un número tras el «?» con Variables y «=» activos. No se
  reproduce en la nube y revisando el código nuevo (préstamos del motor, índices, `unwrap`, cerrojos de la
  configuración) no aparece la causa. En vez de adivinar, se guarda un informe de cada cierre por error
  (`std::panic::set_hook`, en `crashreport.rs`, antes de todo lo demás en `main`) y se avisa al abrir. Así el
  próximo cierre trae el mensaje y la traza exactos.

## Programa de matemáticas en Flatpak (versión 1.3.1, 2026-10-31)

- Tu prueba: «Nuevo entreno» no hacía nada. La causa más probable: dentro de Flatpak la app lanzaba
  `flatpak-spawn --host onepepe-math` por su nombre, y el PATH del escritorio no suele tener `~/.local/bin`; el
  proceso arrancaba, moría sin responder y la app callaba (el aviso solo salía si ni siquiera se podía lanzar).
  Ahora va por la ruta completa (`~/.local/bin/onepepe-math`, donde lo deja `instalar.sh`; `ONEPEPE_MATH_CMD` sigue
  mandando) y cualquier fallo del programa se avisa (como mucho una vez por minuto).
- Los avisos de «Revisar pasos» van debajo del renglón: a la derecha chocaban con el resultado del «=» y con el valor
  de las variables.

## Tus asignaturas y estadística (versión 1.3, 2026-10-30)

- **Vocabulario:** se añaden a `CORE` los símbolos de estadística que el modelo ya tenía y no se usaban (\hat,
  \overline, \tilde, 𝔼, ℙ, ℝ, ⊤, ⟨ ⟩, ∩, ∪, ⇌). Medido con 400 fórmulas de MathWriting: el resto se reconoce igual
  (1725 de 2641 en los dos casos; sustituciones correctas 84,8 % → 84,6 %). Se quedan.
- **Acentos en la estructura:** un sombrero, raya o virgulilla pequeños justo encima de **un** símbolo es su acento
  (nodo `accent`). Una raya es acento solo si es corta como su letra y no hay nada encima (si no, es fracción o «=»).
  ŷ y x̄ son variables distintas de y y x (nombres `yhat`, `xbar`).
- **ε:** el modelo solo tiene `\epsilon`; en el entreno, `\varepsilon` se guarda como `\epsilon` (a mano es igual).
  `\ell` no está en el modelo: en el set se escribe `l`.
- **Set de asignaturas:** 200 = 40 genética de poblaciones + 40 biofísica + 45 aprendizaje estadístico + 15 cálculo
  y álgebra + 30 símbolos sueltos + 30 desarrollos de varios pasos. Medido con letra ajena: 199 de 200 encajan y el
  88 % de los símbolos sale exacto (más bajo que el general, 93,7 %: fórmulas más largas). Los ids empiezan por «a»
  (los del general por «g»): no se mezclan.
- **Glosario de biofísica** como plantilla para copiar, no activado solo: los nombres del glosario valen en toda la
  carpeta y `e` o `R` chocarían con e^x o R² en estadística. Por eso la carga del electrón es `q_e`.

## Modo entreno y versiones (2026-10-30)

- **Versión de OnePepe**: se empezó en **1.1** (lo hecho hasta el 30/10) y el modo entreno es la **1.2**. El número
  está en `meson.build` y en los metadatos; cada versión se describe en **`VERSIONES.md`**. Se muestra en la barra de
  estado y en «Acerca de» con el commit de git (`build.rs`; vacío si no hay git al compilar). Los crates siguen con
  la versión interna de Rnote (0.15.0): no se ven.
- **Archivo de entreno propio** (`*.entreno.json`), no un `.rnote`: así el formato `.rnote` no cambia y Rnote no ve
  archivos que no entiende. Va en `Documentos/OnePepe entrenos`, a la vista y fácil de copiar, y donde Flatpak puede
  escribir (`--filesystem=xdg-documents`). Lleva el set entero (para que valga aunque el set cambie), por dónde vas,
  tus trazos con tiempos, el tamaño del recuadro, y la versión de la app que lo escribió.
- **El recuadro es un lienzo propio** (`gtk::DrawingArea`), no un lienzo de Rnote: más simple, sin herramientas que
  confundan, y los trazos llegan crudos (sin el suavizado del pincel).
- **Renglones de los cálculos seguidos**: el recuadro se divide en bandas iguales, una por paso, con su línea guía;
  cada trazo va al paso de su altura. Así no hay que adivinar renglones.
- **La app envía el contenido, no la ruta**, al programa auxiliar: dentro de Flatpak las rutas del selector de
  archivos no valen fuera.
- **Importar** usa el corte forzado de siempre. Medido con letra de MathWriting y el modelo genérico: las 200
  encajan y el 93,7 % de los ~1700 símbolos sale con corte y etiqueta exactos; el resto son ejemplos con ruido
  (pocos frente a los buenos). Con tu letra y tras reentrenar, debería mejorar.
- **Dibujar las expresiones** con matplotlib (en el programa auxiliar, del color del texto para el tema oscuro).
  Es una librería nueva: hay que pasar `./instalar.sh` una vez.
- **El set de 200** sin tus asignaturas: general (aritmética, fracciones, potencias, álgebra, ecuaciones,
  funciones, cálculo, física y biofísica) más símbolos sueltos y cálculos seguidos. Cuando digas tus asignaturas se
  añade otro set en `tools/onepepe-math/entrenos/`.
