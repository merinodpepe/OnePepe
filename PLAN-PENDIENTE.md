# Plan de mejora de OnePepe (pendiente)

Actualizado: 2026-10-26. Solo lo que falta por hacer. Lo ya hecho está en [`ESTADO.md`](ESTADO.md) y
[`CAMBIOS-ONEPEPE.md`](CAMBIOS-ONEPEPE.md); el plan original completo queda en el historial de git de este archivo.

## 1. Pruebas en tu equipo (lo primero)

Nada se ha ejecutado todavía en la aplicación ni probado con lápiz real. Seguir `CAMBIOS-ONEPEPE.md` y anotar qué falla:

- Hojas infinitas con zoom y desplazamiento.
- Tinta a formas: cuántas formas acierta y cuántas letras convierte por error (para ajustar la sensibilidad).
- Matemáticas: modo matemáticas, sustitución, corrección por toque, resultado del "=", archivo oculto al reabrir.
- Botones del lápiz y extremo borrador, con la salida de `evtest`.
- Regla (tamaño de la zona de enganche y ancho), colores, capas, cuadernos y herramientas de texto.

## 2. Matemáticas con tu letra

Bloqueado hasta tener tus datos (instrucciones en `tools/onepepe-math/README.md`):

- Recibir `mis_expresiones_1.rnote` (las 30 expresiones de `tools/onepepe-math/expresiones.txt`) y `mis_simbolos.rnote`,
  adjuntos en el chat o en tu Google Drive, nunca en el repositorio.
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

## 3. Que la aplicación se llame OnePepe por dentro

Hoy el README y la documentación ya son de OnePepe, pero la aplicación sigue mostrando "Rnote": título de la ventana,
"Acerca de", icono, nombre en el menú de aplicaciones e identificador (`com.github.flxzt.rnote`). Cambiarlo implica:

- **Nombre visible, "Acerca de" e icono:** cambio sencillo (textos, logo de `misc/onepepe/logo.svg` como icono de la app,
  autores y enlaces apuntando a tu repositorio, conservando el crédito a Rnote que exige la GPL).
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

- Listas que continúan solas al pulsar Intro (ahora hay que pulsar el botón en cada párrafo).
- Estilos rápidos de texto: Título 1, Título 2, cita, código.
- Recorte de pantalla.
- Árbol desplegable de tres niveles (cuaderno → sección → página) en la barra lateral, en lugar de entrar en carpetas.
- Capturas de pantalla de OnePepe para el README (cuando funcione en tu equipo).
- Tinta a texto (Fase 5; poco prioritaria).

## 5. Cosas que haces tú en GitHub

- Para que la portada del repositorio muestre el README de OnePepe sin tocar `main`: **Settings → General → Default
  branch** → `onepepe`.

## 6. Ideas para más adelante (sin implementar)

Ideas comentadas el 2026-10-26, solo apuntadas. Ninguna cambia el formato `.rnote`: lo que haya que recordar va en el
archivo oculto de cada página, como en el modo matemáticas.

Esto ya existe y no hace falta hacerlo:

- Puntero láser (`crates/rnote-engine/src/pens/tools/laser.rs`).
- Insertar espacio vertical (`pens/tools/verticalspace.rs`).
- Modo concentración y pantalla completa.
- Color de fondo, rayas, cuadrícula y puntos (`document/background.rs`).
- Preajustes de formato.
- Duplicar página.

### 6.1 Tamaño de hoja y exportación

- **Selector rápido de tamaño de hoja** (DIN A4, A3…) en la barra de arriba, sin entrar en Ajustes.
  - Ya existen A6, A5, A4, A3, A2, Carta, Legal y personalizado (`crates/rnote-engine/src/document/format.rs`, en
    Ajustes → formato; por defecto A3).
  - Se añadirían A1 y A0. Eso solo toca la lista de la interfaz: el `.rnote` guarda el ancho y el alto en mm, así
    que no cambia el formato. Esfuerzo bajo.
- **Exportar ajustado al tamaño pedido.** Hoy el PDF corta lo dibujado en hojas a tamaño real (`engine/export.rs`).
  - Opciones al exportar: "tamaño real (varias hojas)", "encoger para que quepa en 1 hoja" o "encoger hasta N hojas".
    Solo afecta al PDF, no a la página. Esfuerzo bajo-medio.
  - **Aviso:** pasar un A0 a un A4 lo reduce a 1/4 de ancho y de alto, y una letra de 5 mm queda en ~1,2 mm, ilegible.
    La exportación debe avisar si el texto queda demasiado pequeño y proponer A3 o repartir en 2–4 hojas.

### 6.2 Más cerca de lo mejor de OneNote

- **Etiquetas** (Pendiente ☐, Importante ★, Pregunta ?, Definición, Para el examen). Un panel "Resumen de
  etiquetas" las lista de todo el cuaderno y salta a cada una. Esfuerzo medio.
- **Búsqueda global** (Ctrl+E) en los títulos de las páginas, el texto escrito con teclado y el LaTeX reconocido en
  modo matemáticas. La tinta normal solo será buscable cuando exista "tinta a texto". Esfuerzo medio.
- **Plantillas de página con contenido**: Cornell, "apuntes de clase", hoja de problemas, laboratorio, milimetrado.
  - Se eligen al crear la página.
  - Son archivos `.rnote` normales en una carpeta.
  - Esfuerzo bajo.
- **Título y fecha automáticos** arriba de cada página nueva (reutiliza la función de fecha y hora). Esfuerzo bajo.
- **Subpáginas y orden manual** arrastrando en la barra lateral. Va junto al árbol de tres niveles del apartado 4.
  Esfuerzo medio-alto.
- **Notas rápidas**: una sección especial más un atajo (`onepepe --nota-rapida`) que abre una página en blanco al
  momento. Esfuerzo bajo.
- **Tablas sencillas**: una rejilla de cuadros de texto que se mueve junta. Esfuerzo alto.

### 6.3 Más cómodo y visual

- **Menú radial bajo el lápiz** al mantener pulsado un botón: herramientas, colores y deshacer. Esfuerzo medio.
- **Bolígrafos favoritos**: combinaciones de color, grosor y tipo de trazo que se eligen con un clic. Hoy solo hay
  colores favoritos y grosores sueltos. Esfuerzo bajo-medio.
- **Vista oscura del lienzo**: invierte los colores solo en pantalla; el archivo y la exportación no cambian.
  Esfuerzo medio.
- **Minimapa** de la página infinita para saltar de una zona a otra. Esfuerzo medio.
- **Textos nuevos en español**: las cadenas añadidas por OnePepe ("Ink to Shape", "Math Mode"…) están en inglés.
  Hay que traducirlas. Esfuerzo bajo.
- **Tema OnePepe**: color de acento e iconos a juego con el logo. Va junto al apartado 3. Esfuerzo bajo.
- **Vista previa del trazo** al cambiar grosor, presión o estilo. Esfuerzo bajo.
- **Transición suave** cuando el modo matemáticas sustituye un símbolo. Esfuerzo bajo.

### 6.4 Más útil para tomar apuntes

- **Índice de la página** con sus títulos para saltar. Depende de los estilos rápidos del apartado 4. Esfuerzo medio.
- **Modo repaso** (tarjetas de memoria): se marcan con el lazo una pregunta y su respuesta. En modo repaso la respuesta
  queda tapada hasta tocarla. Esfuerzo medio.
- **Exportar una sección o un cuaderno entero a un solo PDF.** Esfuerzo bajo-medio.
- **Copias de seguridad automáticas con versiones**: las últimas N versiones de cada página en una carpeta oculta.
  Esfuerzo bajo.
- **Guía de sincronización** con Syncthing o Nextcloud, y aviso si un archivo cambia fuera de la app. Esfuerzo bajo.
- **Insertar del glosario**: un menú para poner una constante o un símbolo de la asignatura. Esfuerzo bajo.
- **Lista "Para el examen"**: todo lo marcado con esa etiqueta en una página o un PDF. Va después de las etiquetas.

Descartado: todo lo de la lista "No hacer". También "reproducir la tinta", porque el formato no guarda el momento de
cada punto y añadirlo cambiaría el formato.

## No hacer

Copiar formato, adjuntar archivos como icono, búsqueda de imágenes en línea, tinta con texturas, editor de ecuaciones
nativo, enlaces entre páginas y notas, grabación de audio. No cambiar el formato `.rnote` sin avisar, no tocar `main`
ni otros repositorios ni la licencia, no subir modelos, datasets ni datos personales.

## Orden de prioridad

1) Pruebas en tu equipo, 2) matemáticas con tu letra, 3) nombre OnePepe dentro de la app, 4) mejoras opcionales,
5) ideas para más adelante.

Dentro de las ideas para más adelante, la propuesta (sin decidir) es empezar por las baratas y útiles:

1. Selector de tamaño de hoja.
2. Exportar ajustado.
3. Bolígrafos favoritos.
4. Plantillas, con título y fecha.
5. Copias de seguridad.
6. Textos en español.
7. Etiquetas y búsqueda.
