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

Bloqueado hasta tener tus datos (instrucciones en `tools/onepepe-math/README.md`):

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

## No hacer

Copiar formato, adjuntar archivos como icono, búsqueda de imágenes en línea, tinta con texturas, editor de ecuaciones
nativo, enlaces entre páginas y notas, grabación de audio. No cambiar el formato `.rnote` sin avisar, no tocar `rnote-original`
ni otros repositorios ni la licencia, no subir modelos, datasets ni datos personales.

## Orden de prioridad

0) Antes que nada: los dos problemas del modo matemáticas de la sección 2 (escribir cerca de símbolos y zoom).
1) Pruebas en tu equipo, 2) matemáticas con tu letra, 3) nombre OnePepe dentro de la app, 4) mejoras opcionales,
5) ideas para más adelante.

Dentro de las ideas para más adelante, la propuesta (sin decidir) es seguir por:

1. Gráficas de funciones (sección 2).
2. Etiquetas y búsqueda.
3. Índice de la página.
4. Menú radial bajo el lápiz.
