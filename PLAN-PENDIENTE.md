# Plan de mejora de OnePepe (pendiente)

Actualizado: 2026-10-29. Solo lo que falta por hacer. Lo ya hecho está en [`ESTADO.md`](ESTADO.md) y
[`CAMBIOS-ONEPEPE.md`](CAMBIOS-ONEPEPE.md); el plan original completo queda en el historial de git de este archivo.

## 1. Pruebas en tu equipo (lo primero)

Nada se ha ejecutado todavía en la aplicación ni probado con lápiz real. Seguir `CAMBIOS-ONEPEPE.md` y anotar qué falla:

- Hojas infinitas con zoom y desplazamiento.
- Tinta a formas: cuántas formas acierta y cuántas letras convierte por error (para ajustar la sensibilidad).
- Matemáticas: modo matemáticas, sustitución, corrección por toque, resultado del "=", archivo oculto al reabrir.
- Botones del lápiz y extremo borrador, con la salida de `evtest`.
- Regla (tamaño de la zona de enganche y ancho), colores, capas, cuadernos y herramientas de texto.
- Lo de la tercera tanda (ya se ejecutó en la nube, falta en tu equipo con GTK 4.22 y el lápiz): la cinta, el diseño
  morado, los bolígrafos favoritos, insertar tabla y plantillas, recorte de pantalla, tamaño de hoja, exportar
  ajustado y por secciones, nota rápida, estilos y listas, copias de seguridad.
- El diseño nuevo de tu guía (`DISENO.md`). Ya está hecho; falta probarlo en tu equipo:
  - colores en claro, oscuro y alto contraste;
  - **arrastrar** en el árbol y en los favoritos (en la nube no se pudo);
  - los atajos nuevos (Alt+1…6 herramientas, Ctrl+1…5 favoritos);
  - la regla infinita con el lápiz.

## 2. Matemáticas con tu letra

### Problemas que tienes al usarlo (prioridad alta, sin hacer)

**1. No deja escribir cerca de símbolos reconocidos (lo que más te pasa).**
- **Causa** (`crates/rnote-engine/src/engine/math.rs`, `math_filter_pen_event`): si el lápiz baja dentro de la caja
  de un símbolo reconocido (ampliada 4 px), se toma como un toque para corregirlo y se traga el trazo entero aunque
  luego se mueva. El trazo se pierde.
- **Arreglo propuesto:**
  - Decidir al levantar el lápiz:
    - se guardan los eventos en un búfer;
    - si el lápiz se mueve más de 6 px de pantalla, se reenvían al pincel desde el primer punto;
    - si no se mueve, se abren las candidatas.
    - `math_filter_pen_event` devolvería `Vec<PenEvent>` y `handle_pen_event` (`engine/mod.rs`) enviaría cada uno.
  - La zona del toque sería la tinta del símbolo (`hitboxes()` de sus trazos), no su caja.
  - **Tests:**
    - un trazo que empieza en la caja y se mueve se dibuja entero;
    - un toque sigue abriendo las candidatas;
    - un punto junto a la tinta no cuenta como toque.
  - **Esfuerzo:** bajo-medio.
- **Tampoco deja dibujar donde acabas de borrar.**
  - **Causa:** al borrar un símbolo reconocido, su entrada sigue en `math.symbols`. `math_symbol_index_at` no mira
    si sus trazos siguen vivos, así que el hueco sigue «ocupado» y el trazo se toma como toque.
  - Además, `math_take_request` manda como contexto («known») símbolos borrados. Comprueba
    `get_stroke_ref(...).is_some()`, pero los trazos borrados siguen en el almacén, marcados en la papelera
    (`store.trashed(key)`, `crates/rnote-engine/src/store/trash_comp.rs`).
  - **Arreglo:** un símbolo cuenta solo si alguno de sus trazos existe y no está en la papelera. Se aplica en
    `math_symbol_index_at`, en `known` y en el archivo oculto. Si se deshace el borrado (Ctrl+Z), vuelve a contar.
  - **Test:** reconocer «1», borrarlo y dibujar encima → se dibuja y no hay toque.

**2. Con mucho zoom (escribiendo pequeño) reconoce peor.**
- **Comprobado:** la parte de Python no depende del tamaño. Con 150 fórmulas de MathWriting reducidas hasta 1/200
  (símbolos de unas 3 unidades), el resultado es prácticamente igual: 462 → 460 símbolos bien sustituidos. El fallo
  está en la app (Rust) o en cómo se captura la tinta con zoom.
- **Sospechas por revisar:**
  1. El suavizado del pincel (`crates/rnote-compose/src/builders/penpathmodeledbuilder.rs`, `MODELER_PARAMS`)
     trabaja en unidades del documento: con zoom grande, el mismo gesto es mucho más lento en esas unidades y el
     suavizado puede deformar los trazos pequeños.
  2. La elección de los símbolos del mismo renglón (`math_take_request`, banda de ±0,5 × altura) y otras
     tolerancias en unidades del documento.
  3. El grosor del pincel es fijo en unidades del documento: un símbolo pequeño queda muy gordo para su tamaño, y
     el limpio también.
- **Cómo investigarlo:**
  - en la app de prueba de la nube, escribir el mismo símbolo con xdotool al 100 % y al 800–1600 %;
  - comparar el JSON que se envía al programa auxiliar y su respuesta.
- **Arreglo probable:**
  - pasar a escala de pantalla (dividir por el zoom) lo que dependa de la distancia o la velocidad;
  - o enviar los puntos crudos del lápiz.
- **Mientras tanto:** escribe con un zoom moderado (100–200 %).

**3. Reconocer lo que ya escribiste fuera del modo matemáticas (no existe todavía).**
- **Hoy:** solo se reconocen los trazos escritos con el modo matemáticas activo (`math_after_pen_event` en
  `crates/rnote-engine/src/engine/math.rs`). Activar el modo después no hace nada con lo ya escrito.
- **Propuesta:** un botón **«Reconocer selección»** en la pestaña Matemáticas, también en el menú de la selección.
  - Seleccionas con el lazo lo escrito y lo pulsas.
- **Motor** (`math.rs`), una función nueva `math_queue_strokes(keys)`:
  - toma los trazos de pincel seleccionados que no sean ya símbolos;
  - los agrupa en renglones por su altura (bandas de y que se solapan; el reconocedor trabaja línea a línea);
  - los mete en la cola `pending`, un renglón por petición.
  - Esos trazos no tienen tiempos: se envía `times: null` y el corte usa solo la forma, como en los archivos viejos.
- **Interfaz** (`crates/rnote-ui/src/mathhelper.rs`):
  - la acción `win.math-recognize-selection` llama a `recognize_now` una vez por renglón;
  - activa el modo matemáticas si no lo estaba, para que se vean el borde y los toques.
- **Resultado:** lo reconocido con confianza se sustituye por símbolos limpios, igual que al escribir. Ctrl+Z
  devuelve la tinta original. Lo dudoso se queda como estaba y se puede tocar para corregirlo.
- **Tests:**
  - en Rust, dos renglones seleccionados → dos peticiones con sus trazos; los trazos que ya eran símbolos no se
    repiten;
  - prueba en la app de la nube con un `.rnote` escrito sin el modo.
- **Esfuerzo:** bajo-medio (reutiliza todo el camino actual).

**4. Reconoce mal la i y la j.**
- **Medido** con MathWriting (fórmulas sintéticas, solo los casos en que el corte acertó):

  | Letra | Primera opción | Sustituidas |
  |---|---|---|
  | i | 626 de 640 | 579 (las demás salen como «;») |
  | j | 100 de 244 (41 %) | 81 |

  La j se confunde con i (67), «;» (60) y t (15).
- **Causas probables:**
  - el punto se corta aparte (sale «.» o «·») o se junta mal;
  - la j es casi una i con cola, y la «;» también es un punto sobre un trazo.
- **Propuestas:**
  1. Medir también los fallos de corte (el punto separado).
  2. Regla de contexto: un punto justo encima de un palo corto → i o j; es j si el palo baja de la línea base (la
     calcula ya `layout.py`) e i si no.
  3. Tus correcciones (kNN) y reentrenar mejoran tu letra en concreto.

**5. Escribir tú qué símbolo es cuando no sale ni entre las opciones.**
- **Hoy:** la ventanita de candidatas (`show_candidates` en `crates/rnote-ui/src/mathhelper.rs`) solo ofrece las 8
  primeras, «Unir» y «Separar».
- **Propuesta:** debajo, un campo **«Otro…»**:
  - **Búsqueda por nombre**, en español o en LaTeX y sin barra (`delta`, `alfa`/`alpha`, `sumatorio`, `integral`,
    `flecha`, `distinto`, `menor o igual`, `infinito`…), con sugerencias mientras escribes.
  - También se puede **teclear el carácter** (`δ`, `≤`) **o el LaTeX** (`\delta`).
  - **Atajos cortos** para lo difícil de teclear, mostrados al lado de cada símbolo: `\d` → δ, `\D` → Δ, `\a` → α,
    `\b` → β, `\l` → λ, `\p` → π, `\s` → σ, `\S` → Σ, `->` → →, `=>` → ⇒, `<=` → ≤, `>=` → ≥, `!=` → ≠,
    `inf` → ∞.
  - **Una rejilla plegable** con los símbolos difíciles por grupos (griego, operadores, flechas, relaciones), para
    elegir con el lápiz sin teclado.
- **Por dentro:**
  - La corrección se envía como hoy (`math_correction_requests` → orden `correct`).
  - El kNN personal ya acepta etiquetas que el modelo no tiene, así que desde entonces lo puede proponer.
  - El símbolo limpio sale si hay glifo en `glyphs.py` (griego completo y casi todos los operadores); si no, se
    queda tu tinta.
  - Los nombres y atajos van en un archivo nuevo, `tools/onepepe-math/onepepe_math/nombres.py`, que el programa
    auxiliar envía con una orden nueva, `names`, para no duplicarlos en la app.
- **Tests:**
  - en Python, «delta», «\d» y «δ» dan `\delta`;
  - en Rust, una corrección con una etiqueta fuera de las candidatas se aplica.
- **Esfuerzo:** medio.

**6. El resultado del «=» no sale en casos normales.** Probado en la nube el 2026-10-30: «2+3=» da 5, tanto escrito
de una vez como con el «=» añadido después. Pero no sale en estos casos:
- **Lo escrito antes de activar la opción:** no se recalcula. Solo cuenta lo que escribes después.
- **Al corregir un símbolo** (tocarlo y elegir otro, unir o separar): `math_apply_correction` no vuelve a calcular
  el resultado del renglón.
- **Un símbolo mal reconocido** (un «1» leído como «|», un «=» cortado en dos «−»): el programa auxiliar devuelve
  `result: null` y la app no avisa de nada.

**Propuesta:**
- Recalcular el resultado del renglón tras cada corrección y al activar la opción, con una orden `evaluate` que
  envía los `known` del renglón.
- Si no se puede calcular, mostrar un «?» gris tras el «=». Al tocarlo, explica qué ha leído (el LaTeX), para ver
  qué símbolo corregir.

**7. Funciones (ln, log, sin, cos, exp…): se detectan, con fallos.** Las letras seguidas se unen en función
(`FUNCTIONS` en `tools/onepepe-math/onepepe_math/layout.py`: sin, cos, tan, cot, sec, csc, sinh, cosh, tanh,
arcsin, arccos, arctan, ln, log, exp, lim, max, min, det).

Probado el 2026-10-30 con símbolos ya reconocidos:

| Escrito | Sale | ¿Bien? |
|---|---|---|
| `ln(1)=` | 0 | sí |
| `log(100)=` | 2 | sí |
| `sin(0)=` | 0 | sí |
| `exp(0)=` | 1 | sí |
| `ln 2=` | «log(2) ≈ 0,6931» | el valor sí, pero escribe «log» en vez de «ln» (así lo imprime SymPy) |
| `sin x=` | nada | no: la x sin paréntesis se toma como subíndice (`\sin_{x}`) |
| `log₂(8)=` | 0,9031 | **no**: ignora la base y calcula log₁₀ 8 (debería dar 3) |
| `ln` con la «l» leída como «1» | «n» | **no**, y sin avisar: `1n(1)` se toma como 1·n·1 |

**Arreglos propuestos:**
- El argumento sin paréntesis (`sin x`, `ln x`, `log 8`) es argumento, no subíndice. Tras una función, solo es
  subíndice lo que queda claramente por debajo de la línea base (la base de `log`).
- `log_b(x)` → `log(x, b)` en `evaluate.py`.
- Mostrar `ln` en el resultado, no `log`. En español, `log` sin base es log₁₀ (ya es así).
- Unión tolerante: si «1» o «|» va seguido de `n` o de `og`, y «l» está entre sus candidatas, se une como `ln` o
  `log`. Igual con `0`/`o` en `cos` y `log`.
- Faltan: `sen` (seno en español), `tg` y `arcsen`, `arctg`. También `\sqrt[n]{}` con índice, si no está.
- **Tests en Python** con cada fila de la tabla.

**8. Cuánto escribir para entrenar con tu letra.**
- **Recomendación:**
  - unas **80 expresiones variadas** (cerca de 1 hora), de una lista que te preparo yo para cubrir todos tus
    símbolos;
  - más una **página de símbolos sueltos**: unos 20 de cada símbolo difícil (i, j, δ, ∫, x/×, 1/l, 0/o…).
- **Por qué:** cuentan los ejemplos de cada símbolo (10–20 bastan para aprender tu forma), no el número de
  expresiones. Con unos 40 símbolos frecuentes salen 400–800 apariciones, unas 60–100 expresiones. A partir de ahí,
  repetir lo mismo ayuda poco; ayudan los símbolos raros.
- **Medida actual:** de 0 a 15 expresiones, los símbolos bien pasan del 44 % al 56 % y las sustituciones correctas
  del 68 % al 88 % (`MATEMATICAS-INFORME.md` 0.6). Todavía no se sabe cómo sigue la curva.
- **Después:** reentrenar y medir con una parte reservada. Si al pasar de 40 a 80 expresiones aún mejora bastante,
  merece la pena seguir.
- **Mejor en varias sesiones cortas** que de golpe, y al tamaño y con el zoom de siempre.
- **Pendiente mío:** preparar la lista de expresiones y de símbolos cuando me digas tus asignaturas.

### Con tus datos (instrucciones en `tools/onepepe-math/README.md`)

- ~~Recibir `mis_expresiones_1.rnote`~~: recibido y medido (`MATEMATICAS-INFORME.md`, sección 0.5). Falta
  `mis_simbolos.rnote` (la página de símbolos sueltos), adjunto en el chat o en tu Google Drive, nunca en el repositorio.
- **kNN sobre embeddings con almacén de correcciones: hecho** (versión de modelo, borrar o cambiar ejemplos,
  recalcular embeddings al reentrenar, `--personal` y `evaluar_mi_letra.py`). Falta en la app: una pantalla para ver
  y borrar tus ejemplos.
- **Cortar mejor tu letra: hecho** (sección 0.6 del informe).
  - Reentrenar con tus «trozos», que el «trozo» cuente, unir y separar en la app, tiempos entre trazos y botón
    «Reentrenar» en la app.
  - Pendiente: ajustar el peso de las pausas con tus páginas nuevas (ya guardan los tiempos).
- **Cortar con varias hipótesis** (búsqueda en haz con el modelo de contexto): para más adelante.
  - **Al usarlo:** de 5 ms a unos 10–30 ms por fórmula. No se nota, porque ya se espera 0,8 s.
  - **Para hacerlo:** unas 3 veces el trabajo de los umbrales calibrados; va en un módulo nuevo `beam.py`.
- **Una pantalla para ver y borrar tus ejemplos.** El programa auxiliar ya tiene las órdenes `examples`, `forget` y
  `relabel`.
- Respuestas cortas: signo de multiplicar, coma o punto decimal, unidades en el mismo renglón, asignaturas y símbolos
  para el glosario.
- Medir el reconocimiento con tu letra, ajustar umbrales, agrupación y estructura (potencias, subíndices, fracciones)
  a tu escritura "poco lineal", y entrenar el clasificador personal con tus ejemplos.
- Validar con tu letra, una por una, las etapas: aritmética y álgebra → unidades y constantes → ecuaciones, derivadas e
  integrales → matrices, sumatorios y límites.
- Preparar el glosario de cada asignatura (biofísica y las que me digas).
- **Modo matemáticas solo mientras se mantiene pulsado un botón del lápiz** (estaba en el plan como "si es barato" y
  no está hecho).
- **Reentrenar el modelo en tu equipo cuando tú quieras.** Hoy tus correcciones se aplican al momento con el
  clasificador personal (`tools/onepepe-math/onepepe_math/personal.py`, datos en
  `~/.local/share/onepepe/math/correcciones.jsonl`). El modelo grande (`simbolos.npz`), en cambio, solo se entrena con
  `instalar.sh --entrenar` y solo con MathWriting: tus correcciones no entran. Idea:
  - Un botón **"Reentrenar ahora"** (por ejemplo, para cuando te vas un rato del ordenador). Primero muestra el tiempo
    estimado, calculado con lo que tardó la última vez y el número de muestras, y tú decides si empezar.
  - Funciona en segundo plano, con baja prioridad, y se puede cancelar.
  - Mezcla MathWriting con tus correcciones, dando más peso a las tuyas.
  - Solo sustituye el modelo si el nuevo acierta más con tu letra; si no, se queda el anterior.
  - **Aviso** cuando acumules unas 50 correcciones nuevas: "conviene reentrenar". Nunca empieza solo.
  - Todo queda en tu equipo; el modelo nunca se sube a git.
- **Gráficas de funciones.** No depende de tus datos: se puede hacer antes que el resto de esta sección.
  - **Cómo se pide:** en modo matemáticas escribes `y = x² − 3`, `f(x) = sin x` o `x² − 3`. Después la seleccionas
    con el lazo y pulsas **"Dibujar gráfica"**, o lo eliges en el menú que sale al tocar el resultado. Nunca aparece
    sola.
  - **Qué aparece:** ejes 2D con números y la curva junto a la fórmula.
    - Son trazos normales: se mueven, se cambian de tamaño, se borran y se recolorean.
    - Ctrl+Z la quita. No cambia el formato `.rnote` y también se ve en Rnote.
  - **Rango:** por defecto x de −10 a 10 y la y ajustada a la curva. Se cambia escribiéndolo (`x ∈ [0, 2π]`) o
    desde el menú.
  - **Lo que ya tenemos:**
    - Reconocimiento y estructura de la fórmula (`onepepe_math/recognizer.py`, `layout.py`).
    - Conversión a SymPy con unidades (`onepepe_math/evaluate.py`).
    - Glosario de constantes (`glossary.py`).
    - Comunicación entre la app y el programa auxiliar (`server.py` ↔ `math_apply_response` en
      `crates/rnote-engine/src/engine/math.rs`).
    - Ejes 2D de Rnote (`crates/rnote-compose/src/builders/coordsystem2dbuilder.rs`) y el deshacer normal.
  - **Lo que falta:**
    - Elegir la variable.
    - Calcular los puntos con numpy (`sympy.lambdify`).
    - Cortar la curva donde salta o no existe (1/x, tan x).
    - Elegir el rango.
    - Convertir los puntos en trazos.
  - **Etapas:**
    1. `y = f(x)`, con varias curvas en los mismos ejes y un color cada una.
    2. Constantes del glosario y unidades en los ejes.
    3. Más adelante:
       - Ecuaciones implícitas (`x² + y² = 4`).
       - Cortes y máximos marcados.
       - Un parámetro con deslizador (`y = a·x²`).
       - Regenerar la gráfica si corriges la fórmula (el enlace se guardaría en el archivo oculto).
  - **Pruebas:**
    - Tests en Python con x², 1/x, tan x y √x (rango, cortes, puntos).
    - Un test en Rust de que se inserta como trazos y que Ctrl+Z la quita.
    - Después, la prueba con lápiz.
  - **Esfuerzo:** medio.

- Idea: en un desarrollo con flechas (`2x+4=10 → 2x=6 → x=3`), comprobar con SymPy que cada paso se sigue del
  anterior y marcar en gris el paso que no.

## 3. Que la aplicación se llame OnePepe por dentro

Hoy el README y la documentación ya son de OnePepe, pero la aplicación sigue mostrando "Rnote": título de la ventana,
"Acerca de", icono, nombre en el menú de aplicaciones e identificador (`com.github.flxzt.rnote`). Cambiarlo implica:

- **Nombre visible y "Acerca de": hecho** (`feat/nombre-onepepe`).
  - La app se llama OnePepe en el título de la ventana, en el menú de aplicaciones y en "Acerca de OnePepe".
  - En "Acerca de", el desarrollador es Pepe Flock, los enlaces van a tu repositorio y aparece un apartado "Rnote" con
    sus autores.
  - "Donar" sigue llevando a Rnote.
- **Icono de la app: hecho** con tu logo (`misc/pepeflock/onepepe_icono.svg`). Falta el icono monocromo pequeño
  (`rnote-symbolic.svg`): hace falta una versión del logo de un solo color.
- **Identificador de la app** (p. ej. `io.github.merinodpepe.OnePepe`). Es lo delicado:
  - **Ajustes guardados:** Rnote guarda los ajustes (colores favoritos, botones del lápiz, cuadernos de la barra
    lateral, herramientas…) con su identificador. Con uno nuevo, OnePepe empezaría con los ajustes por defecto. Se
    puede añadir una migración que copie los de Rnote la primera vez.
  - **Instalación en paralelo:** con un identificador propio, OnePepe y Rnote se pueden instalar a la vez sin pisarse
    (Flatpak, menú de aplicaciones, ajustes). Hay que renombrar el manifiesto de Flatpak, el archivo `.desktop`, el
    esquema de ajustes, el icono y los metadatos.
  - **Archivos `.rnote`:** no cambian; ambos programas los abren. Hay que decidir cuál los abre por defecto con doble clic.
  - **Traducciones y textos:** los nombres "Rnote" en las cadenas de la interfaz pasarían a "OnePepe".
- Antes de hacerlo: confirmar contigo el identificador y si quieres migrar los ajustes de Rnote.

## 4. Mejoras opcionales (si sobra tiempo y crédito)

- Árbol desplegable de tres niveles (cuaderno → sección → página) en la barra lateral, en lugar de entrar en carpetas.
- Tinta a texto (Fase 5; poco prioritaria).
- Tablas de texto de verdad (hoy la tabla es una rejilla dibujada para escribir a mano).
- Plantillas propias: guardar una página como plantilla y elegirla al crear páginas (hoy hay 4 plantillas fijas).

### Diseño: detalles que quedan

- Los colores de cada cuaderno se eligen ya en la barra de cuadernos de Rnote. Falta que la fila del cuaderno en el
  árbol muestre su color.
- Una tabla de curvas de animación exacta: no tenía la de la guía y se usó `ease-out` de 100–200 ms.
- Ajustes como ventana aparte (ahora siguen en el panel lateral, más ancho cuando se abren).

## 5. Cosas que haces tú en GitHub

- `main` ya es OnePepe. Si la portada aún muestra otra rama: **Settings → General → Default branch** → `main`.

## 6. Ideas para más adelante (sin implementar)

Ideas comentadas el 2026-10-26, solo apuntadas. Ya hechas en la tercera tanda (28/10) y quitadas de aquí: tamaño de
hoja, exportar ajustado, plantillas, título y fecha, nota rápida, tabla dibujada, bolígrafos favoritos, textos en
español, diseño morado tipo OneNote, exportar una sección a PDF y copias de seguridad. Ninguna cambia el formato `.rnote`: lo que haya que recordar va en el
archivo oculto de cada página, como en el modo matemáticas.

Esto ya existe y no hace falta hacerlo:

- Puntero láser (`crates/rnote-engine/src/pens/tools/laser.rs`).
- Insertar espacio vertical (`pens/tools/verticalspace.rs`).
- Modo concentración y pantalla completa.
- Color de fondo, rayas, cuadrícula y puntos (`document/background.rs`).
- Preajustes de formato.
- Duplicar página.

### 6.2 Más cerca de lo mejor de OneNote

- **Etiquetas** (Pendiente ☐, Importante ★, Pregunta ?, Definición, Para el examen). Un panel "Resumen de
  etiquetas" las lista de todo el cuaderno y salta a cada una. Esfuerzo medio.
- **Búsqueda global** (Ctrl+E) en los títulos de las páginas, el texto escrito con teclado y el LaTeX reconocido en
  modo matemáticas. La tinta normal solo será buscable cuando exista "tinta a texto". Esfuerzo medio.
- **Subpáginas y orden manual** arrastrando en la barra lateral. Va junto al árbol de tres niveles del apartado 4.
  Esfuerzo medio-alto.

### 6.3 Más cómodo y visual

- **Menú radial bajo el lápiz** al mantener pulsado un botón: herramientas, colores y deshacer. Esfuerzo medio.
- **Vista oscura del lienzo**: invierte los colores solo en pantalla; el archivo y la exportación no cambian.
  Esfuerzo medio.
- **Minimapa** de la página infinita para saltar de una zona a otra. Esfuerzo medio.
- **Vista previa del trazo** al cambiar grosor, presión o estilo. Esfuerzo bajo.
- **Transición suave** cuando el modo matemáticas sustituye un símbolo. Esfuerzo bajo.

### 6.4 Más útil para tomar apuntes

- **Índice de la página** con sus títulos (Título 1 y 2, ya existen) para saltar. Esfuerzo medio.
- **Modo repaso** (tarjetas de memoria): se marcan con el lazo una pregunta y su respuesta. En modo repaso la respuesta
  queda tapada hasta tocarla. Esfuerzo medio.
- **Guía de sincronización** con Syncthing o Nextcloud, y aviso si un archivo cambia fuera de la app. Esfuerzo bajo.
- **Insertar del glosario**: un menú para poner una constante o un símbolo de la asignatura. Esfuerzo bajo.
- **Lista "Para el examen"**: todo lo marcado con esa etiqueta en una página o un PDF. Va después de las etiquetas.

Descartado: todo lo de la lista "No hacer". También "reproducir la tinta", porque el formato no guarda el momento de
cada punto y añadirlo cambiaría el formato.

## 7. Plan tipo MyScript Math (tu documento)

El plan completo está en [`PLAN-MYSCRIPT.md`](PLAN-MYSCRIPT.md). Todo en local, sin IA en la nube.

**Criterios:**
- cada fase se usa sola y se prueba antes de pasar a la siguiente;
- se activa desde ajustes;
- lo matemático va en un módulo propio.

| Fase | Prioridad | Qué existe ya | Qué falta |
|---|---|---|---|
| 1. Tachar con zigzag para borrar | máxima | nada (la tinta a formas, `rnote-compose/src/shaperecognition.rs`, sirve de modelo para detectar el gesto) | todo |
| 2. Diccionario y catálogo de símbolos | alta | vocabulario `CORE` (`symbols.py`), glifos (`glyphs.py`), `FUNCTIONS` (`layout.py`), unidades con pint, glosario por cuaderno (`glossary.py`), catálogo para insertar (`feat/catalogo-simbolos`) | un solo fichero de datos con nombre, LaTeX, categoría y alias; buscador; constantes editables. Va junto con el punto 5 de la sección 2 |
| 3. Variables (`x = 3` y se recalcula) | muy alta | árbol (`layout.py`), evaluador con SymPy (`evaluate.py`), glosario con valores | definir al escribir `letra = valor`, recalcular las líneas que la usan y mostrarlo al lado; guardarlo en el archivo oculto |
| 4. Revisión de álgebra paso a paso | — | separadores de pasos `→ ⇒ ⇔ ;` (`STEP_SEPARATORS`) | equivalencia por valores aleatorios, marcar en rojo y explicar errores típicos |
| 5. Exportar a LaTeX | — | `to_latex(tree)` (ya lo devuelve `recognize`) | botón «Copiar como LaTeX» sobre una selección |
| 6. Fracciones y gráficos | — | gráficas ya planeadas (sección 2) | alternar decimal ↔ fracción ↔ mixta |

## No hacer

Copiar formato, adjuntar archivos como icono, búsqueda de imágenes en línea, tinta con texturas, editor de ecuaciones
nativo, enlaces entre páginas y notas, grabación de audio. No cambiar el formato `.rnote` sin avisar, no tocar `rnote-original`
ni otros repositorios ni la licencia, no subir modelos, datasets ni datos personales.

## Orden de prioridad

0) Antes que nada: los problemas del modo matemáticas de la sección 2 (escribir cerca de símbolos y donde has borrado, zoom, reconocer lo ya escrito, i/j, escribir tú el símbolo, el resultado del «=» y las funciones ln/log/sin).
1) Pruebas en tu equipo, 2) matemáticas con tu letra (el entrenamiento, en paralelo, cuando escribas tus
expresiones), 3) plan tipo MyScript (sección 7) en su orden: tachar → diccionario → variables → revisión de álgebra
→ LaTeX → fracciones y gráficos, 4) nombre OnePepe dentro de la app, 5) mejoras opcionales, 6) ideas para más
adelante.

Dentro de las ideas para más adelante, la propuesta (sin decidir) es seguir por:

1. Gráficas de funciones (sección 2).
2. Etiquetas y búsqueda.
3. Índice de la página.
4. Menú radial bajo el lápiz.
