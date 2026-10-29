# CONTEXTO PARA OTRAS IA — OnePepe

> Documento de traspaso, pensado para que otra IA (asistente de chat, agente de código u otra sesión) entienda
> **todo** el proyecto sin haber visto las conversaciones anteriores. Está escrito en español porque el usuario y el
> resto de documentos del repositorio lo están; los nombres de código, rutas y comandos van tal cual.
>
> Actualizado: 2026-10-30. Rama de trabajo: `onepepe` (y `main`, que avanza igual). Si algo de aquí contradice al código, manda el código; si
> contradice una petición nueva del usuario, manda el usuario (y actualiza este documento).

> **Lo último (2026-10-29/30), por si solo lees esto:**
>
> - **`main` ya es OnePepe.** El Rnote original está en la rama `rnote-original`. Se quitó lo visible de Rnote:
>   - borrados `CONTRIBUTING.md`, `MAINTAINING.md`, `docs/`, `.github/` (también el CI), las capturas de Rnote y
>     «Donar»;
>   - reescritos `BUILDING.md`, los metadatos de tienda, el `.desktop` y el icono monocromo.
>   - Ver §12, «Sin Rnote a la vista».
> - El usuario **probó el modo matemáticas** y encontró fallos. **Están apuntados, no arreglados**, en
>   `PLAN-PENDIENTE.md` §2, puntos 1–8, con causa y arreglo propuesto. Ver aquí §13.1:
>   - no deja escribir cerca de símbolos reconocidos ni donde has borrado;
>   - va peor con mucho zoom;
>   - no reconoce lo ya escrito;
>   - i y j;
>   - escribir tú el símbolo;
>   - el resultado del «=» en algunos casos;
>   - funciones `sin x` y `log_b`.
> - El usuario mandó su **plan tipo MyScript Math** (`PLAN-MYSCRIPT.md`): tachar, diccionario, variables, revisión
>   de álgebra, LaTeX, fracciones y gráficos. Está en `PLAN-PENDIENTE.md` §7 y aquí en §13.2.
> - **Últimamente pide «apúntalo, no lo hagas»:** documentar en `PLAN-PENDIENTE.md` sin tocar código. Respétalo:
>   no implementes nada de §13 sin que lo pida.

---

## Índice

1. [Resumen en 20 líneas](#1-resumen-en-20-líneas)
2. [Quién es el usuario y cómo trabajar con él](#2-quién-es-el-usuario-y-cómo-trabajar-con-él)
3. [Reglas obligatorias](#3-reglas-obligatorias)
4. [Mapa del repositorio](#4-mapa-del-repositorio)
5. [Conceptos de Rnote que hay que conocer](#5-conceptos-de-rnote-que-hay-que-conocer)
6. [Funciones de OnePepe, una por una](#6-funciones-de-onepepe-una-por-una)
7. [Matemáticas en detalle](#7-matemáticas-en-detalle)
8. [Formatos y archivos que escribe OnePepe](#8-formatos-y-archivos-que-escribe-onepepe)
9. [Atajos de teclado](#9-atajos-de-teclado)
10. [Compilar, instalar y probar](#10-compilar-instalar-y-probar)
11. [Estado: ramas, pruebas y lo que falta probar](#11-estado-ramas-pruebas-y-lo-que-falta-probar)
12. [Decisiones clave y por qué](#12-decisiones-clave-y-por-qué)
13. [Pendiente y prioridades](#13-pendiente-y-prioridades)
14. [Errores ya resueltos y trampas conocidas](#14-errores-ya-resueltos-y-trampas-conocidas)
15. [Glosario español ↔ código](#15-glosario-español--código)
16. [Cómo empezar una sesión nueva (lista de comprobación)](#16-cómo-empezar-una-sesión-nueva-lista-de-comprobación)

---

## 1. Resumen en 20 líneas

- **OnePepe** es la app personal de apuntes a mano del usuario, parte de la familia de proyectos **«Pepe Flock»**.
- Es un **fork de [Rnote](https://github.com/flxzt/rnote)** (Rust + GTK4 + libadwaita, licencia GPL-3.0-or-later),
  en el repositorio **`merinodpepe/OnePepe`** de GitHub.
- Uso: **Linux / Wayland con una tableta Wacom**. Apuntes de clase al estilo **OneNote** (cuadernos, secciones,
  páginas, cinta de herramientas) y, sobre todo, **matemáticas**: escribe fórmulas a mano y un «modo matemáticas» las
  reconoce, las sustituye por símbolos limpios y calcula el resultado del «=».
- El reconocimiento lo hace un **programa auxiliar en Python** (`tools/onepepe-math`) que la app lanza y con el que
  habla por JSON línea a línea. Aprende de las correcciones del usuario (kNN) y se puede **reentrenar** desde la app.
- Rama de integración **`onepepe`**; **`main` = OnePepe** (avanza igual que `onepepe`). El Rnote original está en la
  rama **`rnote-original`** (`6bbc9b2`). Cada función va en una rama `feat/*` fusionada con `--no-ff`.
- Unos **156 archivos cambiados**, **~21 700 líneas añadidas** respecto a `rnote-original`.
- **Estado:**
  - Todo compila y los tests pasan: motor de Rust 64 + 4 + 2 (+1 ignorado), `rnote-compose` 19 (+1 ignorado) y
    Python 34 (+1 saltado sin modelo).
  - Casi todo se ha **ejecutado en una app de prueba en la nube** (GTK 4.14 sobre Xvfb, manejada con xdotool).
  - **Nada está probado todavía con el lápiz real** del usuario ni con GTK 4.22.
- **El formato `.rnote` no se ha cambiado**: lo nuevo que no cabe va a un **archivo oculto** por página, a un índice
  oculto por carpeta o a los ajustes de la app.
- **Documentos humanos** (en la raíz): `ESTADO.md` (qué está hecho), `PLAN-PENDIENTE.md` (qué falta),
  `DECISIONES.md` (por qué), `CAMBIOS-ONEPEPE.md` (cómo probar cada cosa con el lápiz), `DISENO.md` (diseño y
  capturas), `MATEMATICAS-INFORME.md` (matemáticas, medidas), `AUDITORIA.md` (qué traía Rnote).
  `PLAN-MYSCRIPT.md` (plan del usuario tipo MyScript Math). `BUILDING.md` (cómo compilar, corto).

---

## 2. Quién es el usuario y cómo trabajar con él

- **Idioma:** habla **español** (de España). Responde **siempre en español**, **simple**, sin jerga innecesaria, y con
  **comandos exactos que pueda copiar y pegar**. Escribe rápido y con faltas («entocnes», «trabjanado»); entiéndelo
  sin corregirle.
- **Su equipo:** Linux con Wayland y una Wacom.
  - Carpeta del proyecto: **`/home/pepe/Pepe_Flock/One_Pepe`**. Es un clon de `merinodpepe/OnePepe` en la rama
    `onepepe`; se hizo con `git init` + `remote add` + `fetch` + `checkout`, porque la carpeta ya tenía archivos
    suyos. Su README y su informe antiguos se renombraron a `README-viejo.md` y `MATEMATICAS-INFORME-viejo.md`.
  - Tiene un script **`~/instalar-onepepe.sh`** (creado en una sesión anterior, no está en el repositorio). Trae lo
    último, compila con Flatpak, instala, abre OnePepe y avisa con `notify-send` si algo falla.
  - Desde el 2026-10-29, `main` = OnePepe. Para actualizar, siempre se le da:
    ```bash
    cd /home/pepe/Pepe_Flock/One_Pepe
    git checkout main
    git pull
    ~/instalar-onepepe.sh
    ```
    (Su copia estaba en `onepepe`; las dos ramas tienen lo mismo.)
- **Cómo pide las cosas:**
  - A veces manda planes largos y estructurados (por ejemplo, la guía de diseño con tokens de color).
  - A veces deja a la IA trabajando sola: «no me preguntes nada», «yo no voy a estar presente». En ese caso hay que
    **terminar el plan entero** y dejar por escrito (`ESTADO.md` → «Qué necesito de ti») lo que necesite de él.
  - Cuando dice «breve», quiere respuestas cortas.
- **Qué valora:**
  - Honestidad sobre qué está probado y qué no.
  - Que no se rompan sus archivos.
  - Un diseño parecido al **OneNote actual (Fluent)** en el morado de Pepe Flock.
  - Matemáticas que entiendan **su letra** («poco lineal»).
- **Plazo:** el crédito de la nube del usuario caduca el **5 de noviembre de 2026 a las 8:59 (hora de España)**. Al
  planificar, prioriza.
- **Sus datos:**
  - Le ha pasado a la IA `mis_expresiones_1.rnote`: 30 expresiones escritas a mano, la lista de
    `tools/onepepe-math/expresiones.txt`; luego una versión con un sumatorio.
  - Esos datos **solo se usan en la nube de la sesión** (carpeta de trabajo temporal) y **nunca se suben al
    repositorio**.
  - Le falta enviar `mis_simbolos.rnote` (página de símbolos sueltos; ver `tools/onepepe-math/README.md`).

---

## 3. Reglas obligatorias

Literales o casi literales del usuario. **No se negocian** sin preguntarle.

### 3.1 Git

- **«Trabaja solo en mi fork. No toques ningún otro repositorio.»** Repositorio: `merinodpepe/OnePepe`.
- **`main` es OnePepe** (decisión del usuario del 2026-10-29, sustituye a la antigua regla «no toques `main`»).
  Tras fusionar en `onepepe`, avanza `main` con `git push origin onepepe:main` (siempre avance rápido, nunca force).
- **`rnote-original`** guarda el Rnote de partida (`6bbc9b2`) para comparar o sincronizar con el proyecto de origen.
  **No se toca.** Comprueba con `git rev-parse origin/rnote-original`.
- **Rama de integración `onepepe`** (creada desde el Rnote original).
- **Una rama por función**, `feat/<nombre-en-español>` (por ejemplo `feat/diseno-arbol`, `feat/regla-angulo`). Se
  fusiona en `onepepe` con **`git merge --no-ff`** cuando **compila y pasan sus tests** (y, si toca la interfaz, se ha
  visto funcionando en la app de prueba).
- **Commits pequeños, mensajes claros en español.** Formato habitual: `feat(área): qué hace`, un cuerpo que explica
  el porqué y, al final, las líneas de atribución que pida el entorno.
- **Nunca `git push --force`. Nunca borrar ramas** (ni locales ni remotas).
- Subir con `git push -u origin <rama> onepepe`. El entorno de la nube puede pedir además subir a una rama de sesión
  (por ejemplo `claude/modest-cerf-imwi3h`); se ha hecho como copia de `onepepe`, pero el trabajo vive en `onepepe`.
- **No crear pull requests** salvo que el usuario lo pida.

### 3.2 Contenido

- **Mantén la licencia GPL y las cabeceras existentes.** Los archivos nuevos llevan
  `// SPDX-License-Identifier: GPL-3.0-or-later` (Rust) o `# SPDX-License-Identifier: GPL-3.0-or-later` (Python).
  Los créditos de Rnote se conservan (README, «Acerca de»).
- **No cambies el formato `.rnote`.** Es inestable y rompería sus archivos (y la compatibilidad con Rnote). **Si una
  función lo necesita, avísale antes y explica la alternativa.** Las alternativas usadas hasta ahora:
  - el archivo oculto por página;
  - el índice oculto por carpeta;
  - los ajustes de la app;
  - campos `#[serde(skip)]`, que no se escriben al guardar.
- **No subas archivos grandes ni modelos de IA** ni datasets ni datos personales: usa un script de descarga
  (`descargar_mathwriting.sh`, `instalar.sh`). Comprueba con `git ls-files` antes de subir.
- Las **correcciones de su letra** se guardan en un archivo local de su equipo y **nunca se envían a ningún
  servicio**.

### 3.3 Lista «NO HACER» (pedida por el usuario)

- Copiar formato (estilo de un texto a otro).
- Adjuntar archivos como icono.
- Búsqueda de imágenes en línea.
- Tinta con texturas (nuevas).
- Editor de ecuaciones nativo.
- Enlaces entre páginas.
- Grabación de audio.

### 3.4 Forma de trabajar

- **Distingue siempre** entre:
  - compilado (con `cargo check` en la interfaz);
  - tests automáticos;
  - ejecutado en la app de prueba de la nube;
  - probado con el lápiz real.

  **No des por probado algo que solo compila.**
- **Apunta las decisiones** en `DECISIONES.md`, el estado en `ESTADO.md`, cómo probar en `CAMBIOS-ONEPEPE.md` y lo
  que falte en `PLAN-PENDIENTE.md`.
- **«Si no mejora, no se queda»:** las mejoras de reconocimiento se miden. Lo que no mejora se descarta y se documenta
  (por ejemplo, los umbrales de corte calibrados).
- **Interfaz en español:**
  - Los textos van en inglés dentro de `gettext("…")`, como el resto de Rnote.
  - La traducción se añade a `crates/rnote-ui/po/es.po`. Comprueba con `msgfmt -c -o /dev/null po/es.po`.
  - Los archivos nuevos con textos se añaden a `crates/rnote-ui/po/POTFILES`.
  - Ojo con entradas obsoletas (`#~ msgid`) que dupliquen: `msgfmt` falla.
- **Planes:** para cambios grandes, primero un plan breve en español con las decisiones que se tomarán; luego se
  ejecuta entero.

---

## 4. Mapa del repositorio

```
OnePepe/
├── Cargo.toml                 workspace; versiones de gtk4 (feature v4_22) y libadwaita (v1_9)
├── meson.build                app_name_capitalized = 'OnePepe', autor «Pepe Flock», URLs del fork
├── build-aux/com.github.flxzt.rnote.Devel.yaml   manifiesto Flatpak (+ permiso org.freedesktop.Flatpak)
├── setup-deps.sh              comprueba/instala dependencias; recomienda Flatpak si el sistema es viejo
├── crates/
│   ├── rnote-compose/         geometría y estilos, SIN GTK (testeable en cualquier sitio)
│   ├── rnote-engine/          motor: documento, trazos, plumas, render, export (feature "ui" = parte GTK)
│   ├── rnote-ui/              la app GTK4/libadwaita (binario `rnote`)
│   └── rnote-cli/             herramienta de línea de comandos de Rnote (sin cambios de OnePepe)
├── tools/onepepe-math/        programa auxiliar de matemáticas (Python)
├── misc/
│   ├── pepeflock/             logos del usuario (onepepe_icono.svg = icono de la app, onepepe_lateral.png…)
│   └── onepepe/               capturas (captura-cinta.png, diseno/*.png)
└── *.md                       documentos (ver sección 1)
```

### 4.1 Versiones y *features*

- **GTK 4.22** (`gtk4 = { version = "0.11.5", features = ["v4_22"] }`) y **libadwaita 1.9**
  (`libadwaita 0.9.2, features = ["v1_9"]`): las del equipo del usuario, vía Flatpak (GNOME SDK).
- La nube solo tiene **GTK 4.14 / libadwaita 1.5**: ver la sección 10.3.
- `rnote-engine` tiene la *feature* `ui` (GTK, render a `gtk4::Snapshot`). Los tests del motor se ejecutan **sin**
  ella, así que el código `#[cfg(feature = "ui")]` (por ejemplo, el dibujo de la regla o del margen) solo lo comprueba
  `cargo check` de la interfaz.

### 4.2 Archivos de OnePepe (añadidos, `A`) y los más tocados (`M`)

**`rnote-compose/src`:**

| Archivo | Qué |
|---|---|
| `palette.rs` (A) | paleta de 48 colores |
| `ruler.rs` (A) | geometría de la regla: `Ruler` (center, angle, length, width, `infinite`, `snap_to_steps` con skip), `RulerInteraction`, `visible_part`, `anchor_near`, `TICK_STEP` = 8, `NUMBER_STEP` = 80 |
| `shaperecognition.rs` (A) y `shaperecognition/tests.rs` (A) | tinta a formas |
| `color.rs` (M) | `ONEPEPE_BRAND`, `ONEPEPE_BRAND_TINT`, `ONEPEPE_FG_3` |
| `style/indicators.rs` (M) | asas en el color de marca |

**`rnote-engine/src`:**

| Archivo | Qué |
|---|---|
| `backup.rs` (A) | copias de seguridad con versiones: `backup_before_save`, `KEEP_DEFAULT` = 10, `MIN_INTERVAL_DEFAULT` = 20 min |
| `onepepe_index.rs` (A) | índice oculto por carpeta (orden y subpáginas) |
| `engine/math.rs` (A) | estado del modo matemáticas, peticiones y respuestas, archivo oculto |
| `engine/ruler.rs` (A) | regla en el motor: interceptar eventos, dibujar, ángulo |
| `engine/onepepe_insert.rs` (A) | tamaño de hoja, tabla, plantillas, título y fecha, PDF ajustado, `DocStatus` |
| `engine/onepepe_lines.rs` (A) | «Fondo y renglones» y reescritura de páginas cerradas |
| `engine/onepepe_notebook.rs` (A) | exportar varias páginas en un PDF |
| `pens/penpresets.rs` (A) | bolis favoritos |
| `pens/typewriter/scripts.rs` (A) | sub/superíndices, listas, Intro que sigue la lista, estilos de texto |
| `tests/infinite_layout.rs` (A) y `tests/rnote_fixture.rs` (A) | tests de integración |
| `camera.rs` (M) | límites de zoom y zoom vectorial |
| `image.rs` (M) | tope de 4096 px por imagen |
| `document/background.rs` (M) | grosor de línea y línea de margen con `#[serde(skip)]` |
| `document/format.rs` (M) | guías y cruz del origen ocultas por defecto, color de las guías |
| `engine/rendering.rs` (M) | margen en pantalla, teselas del fondo limitadas, cruz de 12 px |
| `pens/brush.rs` (M) | tinta a formas |
| `pens/selector/mod.rs` (M) | selección en morado, asas de 8 px |
| `pens/shortcuts.rs` (M) | variantes de los botones del lápiz |
| `strokes/brushstroke.rs` (M) | imágenes por tramo recortadas a la vista |
| `store/selection_comp.rs` (M) | seleccionar ya no sube al frente |
| `store/chrono_comp.rs` (M) | capas |

**`rnote-ui/src`:**

| Archivo | Qué |
|---|---|
| `ribbon.rs` (A, ~1900 líneas) | **la cinta** y la mayoría de acciones nuevas `win.*` |
| `theme.rs` (A) | tokens de color, CSS de los temas, lienzo oscuro, tests de contraste |
| `prefs.rs` (A) | ajustes propios en `~/.config/onepepe/ajustes.json` |
| `statusbar.rs` (A) | logo de la cabecera, barra de estado, logo tenue en el documento vacío |
| `optionsbar.rs` (A) | barra de opciones flotante |
| `favorites.rs` (A) | bolis favoritos en la cinta |
| `textstyles.rs` (A) | estilos de texto editables |
| `lines.rs` (A) | ventana «Fondo y renglones» |
| `settingslayout.rs` (A) | ajustes en pestañas con Revertir/Aplicar |
| `mathhelper.rs` (A) | lanzar el programa auxiliar, enviar y recibir, candidatas, unir/separar, reentrenar, archivo oculto |
| `workspacebrowser/tree.rs` (A) | árbol del cuaderno |
| `workspacebrowser/workspaceactions/createpage.rs` (A) | nueva página |
| `workspacebrowser/filerow/actions/move_to.rs` (A) | «Mover a…» |
| `appwindow/actions.rs` (M) | atajos, `math-mode`, `ruler` |
| `appwindow/imp.rs` (M) | `theme::setup` |
| `canvas/mod.rs` (M) | matriz de color del lienzo oscuro |
| `canvas/imexport.rs` (M) | copia de seguridad al guardar, archivo oculto |
| `canvaswrapper.rs` (M) | límites de zoom |
| `dialogs/mod.rs` (M) | «Acerca de» |
| `utils.rs` (M) | `onepepe_backup_root()` |
| `workspacebrowser/mod.rs` (M) | el árbol sustituye a la lista plana |
| `penssidebar/*` (M) | catálogo de símbolos, botones de capas, sub/superíndice |

**`rnote-ui/data`:**

| Archivo | Qué |
|---|---|
| `style.css` (M) | **solo colores `@op_*`**: un test lo comprueba |
| `icons/scalable/actions/onepepe-*.svg` (A) | logo monocromo, regla, tinta a formas, matemáticas |
| `icons/scalable/apps/rnote*.svg` (M) | el icono de Pepe Flock |
| `resources.gresource.xml` y `meson.build` (M) | los iconos nuevos tienen que estar en **los dos** |
| `ui/shortcuts.ui` (M) | la ventana de atajos |
| `ui/*.ui` (M) | penssidebar, filerow, workspacebrowser… |

**`rnote-ui/po`:** `es.po` (M) con más de 300 textos de OnePepe; `POTFILES` (M).

**`tools/onepepe-math`:** todo es nuevo; ver la sección 7.

---

## 5. Conceptos de Rnote que hay que conocer

- **`Engine`** (`rnote-engine/src/engine/mod.rs`): un motor por pestaña. Tiene `document`, `store` (trazos),
  `camera`, `penholder` (plumas), `config` (`EngineConfigShared`, compartida entre pestañas), `audioplayer`, etc.
  OnePepe añadió `ruler: Option<Ruler>`, `ruler_interaction` y `math: MathState`.
- **`WidgetFlags`:** lo que devuelve casi toda operación del motor; la interfaz lo aplica con
  `appwindow.handle_widget_flags(flags, &canvas)`. Campos: `redraw`, `resize`, `refresh_ui`, `store_modified`,
  `zoomed`, …
- **`EngineView` / `EngineViewMut`** (macros `engine_view!`, `engine_view_mut!`): vista prestada de varias partes del
  motor para las plumas.
- **`StrokeStore`:** trazos en un `SlotMap` con **`StrokeKey` estables** (se conservan al guardar y abrir; el archivo
  oculto depende de ello).
  - `record(Instant)` crea un punto de deshacer.
  - `update_geometry_for_stroke` y `regenerate_rendering_for_stroke(key, viewport, image_scale)`.
- **Trazos** (`strokes/`): `BrushStroke`, `ShapeStroke`, `TextStroke` (con `text_style.ranged_text_attributes`),
  `VectorImage`, `BitmapImage`. `Content::gen_images(viewport, image_scale)` genera las imágenes de pantalla, ya
  **recortadas a la vista ampliada** (`VIEWPORT_EXTENTS_MARGIN_FACTOR` = 0.4).
- **`Camera`:** `zoom` (permanente), `temporary_zoom` (mientras haces zoom), `total_zoom()`, `image_scale()` =
  `zoom` × `scale_factor`, `viewport()` en coordenadas del documento. `zoom_w_timeout` aplica el zoom definitivo a los
  400 ms.
- **`Document`:** `config: DocumentConfig { format: Format, background: Background, layout: Layout }`, más `x`, `y`,
  `width` y `height`.
  - `Layout`: `FixedSize`, `ContinuousVertical`, `SemiInfinite`, `Infinite`.
  - `Format`: tamaño de página en px a `dpi` (96 por defecto), `show_borders` (guías), `show_origin_indicator`
    (cruz), `border_color`.
  - `Background`: `color`, `pattern: PatternStyle` (`None`, `Lines`, `Grid`, `Dots`, `IsometricGrid`,
    `IsometricDots`), `pattern_size`, `pattern_color`.
- **Plumas** (`pens/`):
  - `PenStyle`: `Brush`, `Shaper`, `Typewriter`, `Eraser`, `Selector`, `Tools`.
  - `ToolStyle` (dentro de `Tools`): `VerticalSpace`, `OffsetCamera` (la «Mano» de OnePepe), `Zoom`, `Laser`.
  - La configuración de todas está en `EngineConfig.pens_config` (por ejemplo `brush_config`, donde OnePepe metió
    `shape_recognition`, `shape_recognition_sensitivity` y `math: MathOptions`).
  - `EngineConfig` se guarda en GSettings como JSON (los ajustes de la app, no los documentos).
- **Instantáneas:** `EngineSnapshot { document, camera, stroke_components, chrono_components, chrono_counter }`.
  - `EngineSnapshot::load_from_rnote_bytes(bytes).await`.
  - `Engine::save_as_rnote_bytes(name)` devuelve un `oneshot::Receiver`.
  - `RnoteFile` (= `RnoteFileMaj0Min15`) envuelve la instantánea en `ijson`.
- **Render:**
  - `Engine::draw_to_gtk_snapshot` dibuja fondo (teselas de textura), trazos (imágenes), bordes, cruz, regla,
    selección…
  - El lienzo es `RnCanvas` (`canvas/mod.rs`), dentro de `RnCanvasWrapper` (con scroll), dentro de una
    `adw::TabPage`.
  - `appwindow.tabs_snapshot()` da las pestañas; `appwindow.active_tab_canvas()` da el lienzo activo.
- **Acciones:**
  - `win.*` registradas en `appwindow/actions.rs` (Rnote) y en `ribbon.rs`, `favorites.rs`, etc. (OnePepe).
  - Los atajos se ponen con `app.set_accels_for_action`.
  - Ayudantes de `ribbon.rs`: `string_action`, `plain_action`, `with_canvas` (aplica una función al motor activo y
    gestiona los flags) y `give_focus_back`.
- **Plantillas `.ui`:** los widgets de Rnote se definen con `CompositeTemplate`; los hijos con id son `TemplateChild`
  accesibles con `.imp()`.
  - **Ojo:** a los widgets sin `TemplateChild` no se puede llegar por id en tiempo de ejecución (sección 14).
  - Los widgets nuevos de OnePepe se crean **en código, no en `.ui`**: así el compilador detecta los errores.
- **Ajustes de Rnote:** `RnSettingsPanel` (`settingspanel/`), en la barra lateral, en un `adw::ViewStack` junto al
  explorador de cuadernos. Los ajustes de la ventana son propiedades de `RnAppWindow` (`autosave`, `righthanded`, …)
  y se guardan en GSettings al cerrar.
- **Explorador de cuadernos:**
  - «Áreas de trabajo» (workspaces) = **cuadernos**: barra vertical `RnWorkspacesBar`, con color e icono.
  - Carpetas = **secciones**; archivos `.rnote` = **páginas**.
  - `RnWorkspaceBrowser` tiene un `DirectoryList` (`dir_list`) con la carpeta del cuaderno; `dir_list_dir()` da su
    ruta.

---

## 6. Funciones de OnePepe, una por una

Formato de cada ficha: **Qué** · **Código** · **Acciones / atajos** · **Estado guardado en** · **Tests** · **Probado**.
«Nube» = ejecutado en la app de prueba con GTK 4.14; «lápiz» = pendiente en todas.

### 6.1 Fases 0–4 (primeras tandas)

**Hojas infinitas** (`feat/hojas-infinitas`)
- **Qué:** ya existían en Rnote (`Layout::Infinite`); se añadió un test.
- **Código:** `rnote-engine/tests/infinite_layout.rs`.
- **Tests:** 4.

**Paleta de 48 colores** (`feat/colores`)
- **Qué:** un desplegable ▾ junto a los colores. Elegir sustituye la casilla favorita seleccionada (Rnote ya guarda
  esas casillas).
- **Código:** `rnote-compose/src/palette.rs`, `rnote-ui/src/colorpicker/`.
- **Tests:** 3.

**Capas** (`feat/capas`)
- **Qué:**
  - Subir y bajar un nivel respecto a los objetos que se solapan, o al frente y al fondo.
  - Las imágenes y los PDF pasan a la capa de tinta (`UserLayer(0)`) solo al reordenarlos. Es un valor que el formato
    ya admite.
  - Seleccionar ya no sube al frente.
- **Código:** `store/chrono_comp.rs`, `store/selection_comp.rs`, `penssidebar/selectorpage.rs`.
- **Acciones / atajos:** acción `win.selection-z-order` (raise, lower, top, bottom).
- **Tests:** 5.

**Regla** (`feat/regla`, `feat/diseno-regla-renglones`, `feat/regla-angulo`)
- **Qué:**
  - Regla virtual; solo actúa sobre el pincel.
  - El trazo se pega al borde si **empieza** cerca y se queda pegado hasta levantar el lápiz.
  - Centro = mover; círculo = girar.
  - **Ángulo libre**: con **Mayús**, de 15° en 15° (`Ruler::snap_to_steps`, que el motor pone según
    `ModifierKey::KeyboardShift`).
  - **Modo infinito:** una línea que cruza todo el lienzo; se dibuja con `visible_part`. Al empezar a tocarla, su
    centro se recoloca con `anchor_near` en el punto de la línea más cercano al centro de la vista, en múltiplos de
    80, así que el círculo de giro siempre está a la vista.
  - Marcas cada 8 px (más largas cada 40), números cada 80 y una banda brand-tint de 24 px en cada borde, todo en los
    colores de la marca.
  - Dibujar → **Ángulo**: un número exacto, o 0/30/45/60/90/135°.
- **Código:**
  - modelo en `rnote-compose/src/ruler.rs`;
  - motor en `rnote-engine/src/engine/ruler.rs`: `set_ruler_enabled`, `set_ruler_infinite`, `ruler_angle_degrees`,
    `set_ruler_angle_degrees`, `apply_ruler_to_pen_event` y `draw_ruler(cx, ruler, zoom, handle)`;
  - interfaz: acciones en `ribbon.rs` (`ruler_angle_popover`).
- **Acciones / atajos:** `win.ruler` (Ctrl+Alt+R), `win.ruler-infinite`.
- **Estado guardado en:** el **archivo oculto** (campo `ruler`); se guarda al guardar la página.
- **Tests:** compose 12, motor 4.
- **Probado:** nube, girada a mano y puesta a 30°.

**Tinta a formas** (`feat/formas`)
- **Qué:**
  - Al terminar un trazo, si encaja claramente en una línea, círculo/elipse, triángulo o rectángulo, se sustituye por
    un `ShapeStroke` (Ctrl+Z vuelve al trazo).
  - Rechaza la escritura: tamaño mínimo de 50 px en pantalla, una sola vuelta, sin rabito.
  - Viene **desactivada por defecto**.
- **Código:** algoritmo en `rnote-compose/src/shaperecognition.rs`; integración en `pens/brush.rs`
  (`convert_to_shape`).
- **Acciones / atajos:** `win.shape-recognition`.
- **Tests:** 9.
- **OJO:** **el modo matemáticas no la desactiva**. Si está encendida, un «0» grande puede convertirse en círculo.
  Al usuario se le aconsejó apagarla para matemáticas y se le ofreció que el modo matemáticas la ignore (no hecho
  todavía).

**Botones del lápiz** (`feat/botones-lapiz`)
- **Qué:**
  - Variantes «Borrador (trazos enteros)», «Borrador (área)» y «Selección (lazo)» mientras pulsas un botón.
  - Campo `variant` en la acción del atajo (ajustes de la app).
  - Tamaños del borrador 8/24/64.
- **Código:** `pens/shortcuts.rs`, `settingspanel/penshortcut*.rs`.
- **Tests:** 4.

**Texto y utilidades** (`feat/subindices`, `feat/catalogo-simbolos`, `feat/fecha-hora`, `feat/listas`)
- **Qué:**
  - Sub y superíndice con caracteres Unicode (², ₂).
  - Catálogo de símbolos (desplegable).
  - Fecha y hora (dd/mm/aaaa, HH:MM).
  - Viñetas, listas numeradas y casillas: prefijos «• », «1. », «☐ »/«☑ ».
- **Código:** `pens/typewriter/scripts.rs`, `penssidebar/typewriterpage.rs`.

**Cuadernos** (`feat/cuadernos`)
- **Qué:** «Nueva página» (`.rnote` vacío con el formato por defecto) y «Mover a…» (selector de carpetas, renombrado
  de ruta). Lo demás ya existía.
- **Código:** `workspaceactions/createpage.rs` (`write_empty_page` es `pub(crate)`) y `filerow/actions/move_to.rs`.
- **Nota:** el archivo oculto acompaña a la página al renombrar, mover, duplicar o borrar desde el explorador.

### 6.2 Tercera tanda (cinta y utilidades)

**Cinta tipo OneNote** (`feat/cinta`, `feat/diseno-onenote`, `feat/diseno-fluent`, `feat/diseno-cinta`)
- **Código:** `rnote-ui/src/ribbon.rs`.
  - `setup(appwindow)` construye un `gtk::Stack` con pestañas y un `StackSwitcher` en la banda morada, y lo
    antepone a la caja que contiene `tabbar` y `overlays`.
  - Al final llama a `statusbar::setup`, `optionsbar::setup` y `settingslayout::setup`.
  - `refresh(appwindow)` se llama desde `refresh_ui` y sincroniza los botones conmutadores (tinta a formas, guías,
    regla infinita, favorito activo).
- **Pestañas:**
  - **Archivo:** Documento, Exportar e imprimir, Copias de seguridad, App.
  - **Inicio:** Portapapeles, Deshacer, Texto, Estilos (galería), Selección, Fecha.
  - **Insertar:** Espacio, Tablas, Archivos, Contenido, Fecha y hora, Páginas, Símbolos.
  - **Dibujar:**
    - Herramientas: Bolígrafo, Borrador, Lazo, Formas, Texto, Láser, Mano.
    - Colores: el `RnColorPicker` movido a la cinta.
    - Bolígrafos favoritos.
    - Ayudas: Regla, Regla infinita, Ángulo, Tinta a formas.
  - **Matemáticas:** Modo matemáticas, Tu letra (Reentrenar…).
  - **Vista:**
    - Zoom: Ampliar, Alejar, 100 %, Ancho de página, Zoom vectorial, Mano.
    - Página: Tamaño de hoja, Ajustar al contenido, Volver al inicio.
    - Fondo y renglones: Líneas, Regla infinita.
    - Ventana: Modo concentración, Pantalla completa, Cinta, Guías, Lienzo oscuro.
- **Bloques de construcción:** `big_button`, `small_button`, `text_button`, `big_toggle`, `big_menu`, `group`,
  `rows`, `page`, `list_popover` y `available_icon` (si falta un icono, pone `dot-symbolic`). Todos con
  `set_focus_on_click(false)` para no quitar el foco al texto.
- **Acciones / atajos:** `win.ribbon` (Ctrl+F1) oculta la cinta. `sync_ribbon_visibility`: la barra de herramientas
  flotante de Rnote (penpicker) solo se ve si la cinta está oculta, y el modo concentración oculta la cinta.

**Insertar** (`feat/insertar`, `feat/notas-rapidas-y-seccion-pdf`)
- **Qué y acciones:**
  - Tabla dibujada (`win.insert-table`, rejilla de líneas).
  - Plantillas (`win.insert-template`): Cornell, clase, problemas.
  - Título y fecha.
  - Recorte de pantalla (`win.insert-screenshot`, portal `org.freedesktop.portal.Screenshot` por D-Bus).
  - Nota rápida (`win.quick-note`: una página nueva en la carpeta «Notas rápidas» de la sección, con título y fecha).
  - Tamaño de hoja (`win.page-size`): A5–A0 y Carta, vertical u horizontal.
  - Exportar PDF ajustado (`win.export-fitted`): a una hoja o al ancho, nunca agranda y avisa si la escala baja de
    0,5.
  - Exportar la sección a un PDF (`win.export-section`, en el **orden del árbol**: `section_pages_in_order`).
- **Código:** `engine/onepepe_insert.rs` (`PageSize`, `PageTemplate`, `FitMode`, `fit_scale`,
  `export_doc_as_pdf_fitted`, `DocStatus`, `PageSize::matching`) y `engine/onepepe_notebook.rs`
  (`export_pages_as_one_pdf`).

**Estilos y listas** (`feat/estilos-listas`)
- **Qué:** Intro continúa la lista (`enter_edit`); una viñeta vacía termina la lista.

**Copias de seguridad** (`feat/copias-seguridad`)
- **Qué:** antes de guardar, se copia la versión anterior. Como mucho una cada 20 minutos y 10 por página.
- **Código:** `backup.rs`; la llamada está en `canvas/imexport.rs`.
- **Estado guardado en:** `~/.local/share/onepepe/copias/`.
- **Acciones / atajos:** `win.open-backups` abre esa carpeta.

**Traducción** (`feat/traduccion-es`): ver la sección 3.4.

**Marca OnePepe** (`feat/nombre-onepepe`, `feat/logos-pepe-flock`, `feat/readme`)
- **Qué:**
  - Nombre visible «OnePepe»; el identificador **sigue siendo `com.github.flxzt.rnote`**, para no perder ajustes ni
    romper Flatpak.
  - «Acerca de»: desarrollador «Pepe Flock», con un apartado de créditos a Rnote.
  - Icono de Pepe Flock.
  - README propio.
- **Pendiente:**
  - El usuario debe poner `onepepe` como rama por defecto en GitHub.
  - Cambiar el identificador interno está en el plan.

### 6.3 Diseño según la guía del usuario (29/10/2026)

La guía pedía: jerarquía por tono, un solo acento morado, poco marco, escala de espacios 2/4/6/8/12/16/24/32, radios
4/6/8/12, sombras mínimas, movimiento de 100–200 ms y contraste medido. Detalle y capturas en `DISENO.md`.

**Tokens y temas** (`feat/diseno-tokens`)
- **Código:** `rnote-ui/src/theme.rs`.
  - `TOKENS: &[(nombre, claro, oscuro)]`: la tabla de la guía.

    | Token | Claro | Oscuro |
    |---|---|---|
    | bg_canvas | FFFFFF | 292929 |
    | bg_surface | FAFAFA | 1F1F1F |
    | bg_frame | F0F0F0 | 141414 |
    | bg_raised | FFFFFF | 333333 |
    | fg_1 | 242424 | FFFFFF |
    | fg_2 | 424242 | D6D6D6 |
    | fg_3 | 616161 | ADADAD |
    | fg_disabled | 9E9E9E | 6B6B6B |
    | stroke_1 | D1D1D1 | 666666 |
    | stroke_2 | E0E0E0 | 4A4A4A |
    | header | 5B1785 | 2A1638 |
    | brand | 7719AA | B889D6 |
    | brand_hover | 5F1388 | CBA6E3 |
    | brand_tint | F3E8F9 | 3A2447 |
    | hover | F5F5F5 | 383838 |

    Más los de OnePepe: `on_header`, `on_brand`, `canvas_cross` y `checker` (#0F0F0F en los dos temas, para el
    damero de transparencia).
  - `HIGH_CONTRAST`: refuerza fg_3, stroke_1 y stroke_2.
  - `ADW_COLORS`: colores de libadwaita → token.
  - `theme_css(dark, hc)`:
    - escribe `@define-color op_<token> …;` y `@define-color <color_adw> @op_<token>;`;
    - y, **solo si GTK ≥ 4.16** (`gtk4::check_version`), un bloque `:root { --window-bg-color: … }` para
      libadwaita ≥ 1.6.
  - `setup(appwindow)`:
    - pone un `CssProvider` con prioridad `APPLICATION - 1`, por debajo de `style.css`;
    - escucha `StyleManager::dark` y `high-contrast`;
    - marca la ventana con las clases `dark` y `high-contrast`.
  - `canvas_dark()` y `canvas_dark_matrix()`: matriz de color que invierte la luminosidad y conserva el tono. Blanco
    → #292929, negro → #E8E8E8. Se aplica en `canvas/mod.rs` con `snapshot.push_color_matrix`, solo en pantalla.
- **Tests** (solo se ejecutan en el árbol de prueba con GTK, `cargo test -p rnote theme`):
  - `contrast_of_the_themes`: 4,5:1 texto y 3:1 controles, en claro, oscuro y alto contraste;
  - `no_loose_colors_in_the_stylesheet`: ni `#hex` ni `rgb()` en `style.css`, salvo líneas marcadas
    `op-allow-literal`;
  - `dark_matrix_maps_paper_and_ink`.
- **Acciones / atajos:** `win.dark-canvas`, que guarda `lienzo-oscuro`.
- **Probado:** nube, en los tres temas (`ADW_DEBUG_COLOR_SCHEME=prefer-dark`, `ADW_DEBUG_HIGH_CONTRAST=1`). La matriz
  se verificó con un render real fuera de pantalla.

**Cabecera, barra de estado y lienzo** (`feat/diseno-cinta`)
- **Qué:**
  - Logo monocromo de 24 px en la cabecera (`onepepe-logo-symbolic.svg`).
  - Cabecera y pestañas en `@op_header`; la pestaña activa se une a la cinta.
  - Cinta sin marco, con una línea de 1 px.
  - **Barra de estado de 24 px:** tamaño de hoja, páginas, guardado y zoom, refrescada cada 250 ms con
    `Engine::doc_status()`.
  - Logo tenue en el documento vacío.
  - Lienzo:
    - selección y asas en morado (`ONEPEPE_BRAND`, asas dibujadas de 8 px aunque se agarran con 18);
    - cruz del origen de 12 px gris y **oculta por defecto**;
    - **guías** (bordes de página) **ocultas por defecto** (`Format::default`), activables con `win.doc-guides`.
- **Código:** `rnote-ui/src/statusbar.rs`.

**Barra de opciones flotante** (`feat/diseno-barra`)
- **Qué:**
  - Es la `penssidebar` de Rnote (opciones de la herramienta activa) dentro de `overlays().sidebar_box()`.
  - Encima: un asa para arrastrarla (arriba y abajo, y al otro lado si la sueltas en la otra mitad), «plegar» y
    «Personalizar».
  - «Personalizar»: hasta 3 herramientas en orden, que pueden ser pincel, formas, texto, borrador, selección,
    herramientas o «mano»; y el lado (automático = sigue la propiedad `righthanded`, izquierda o derecha).
- **Código:** `rnote-ui/src/optionsbar.rs`.
- **Estado guardado en:** claves `barra-herramientas`, `barra-lado`, `barra-plegada` y `barra-y`.

**Bolis favoritos** (`feat/boligrafos-favoritos`, `feat/diseno-favoritos`)
- **Qué:**
  - **5 huecos** (`PenPresets::MAX`).
  - Cada `PenPreset` guarda `name`, `style` (Marker/Solid/Textured), `color` (con su alfa = opacidad), `width` y
    `pressure_curve`.
  - Círculos de 28 px con una onda del trazo; anillo morado en el activo (`position_of`, que compara el color con
    tolerancia de 1/255).
  - Clic elige; mantener pulsado guarda aquí; «+» guarda en el primer hueco libre; clic derecho → guardar aquí,
    mover a la izquierda o la derecha, borrar; arrastrar para reordenar (sin probar en la nube).
- **Código:** `rnote-engine/src/pens/penpresets.rs` y `rnote-ui/src/favorites.rs`.
- **Acciones / atajos:** `win.pen-preset(i)` (**Ctrl+1…Ctrl+5**), `pen-preset-save`, `pen-preset-save-at`,
  `pen-preset-move` (destino = desde×100 + hasta) y `pen-preset-remove`.
- **Estado guardado en:** `~/.config/onepepe/boligrafos.json`. Una lista antigua de más de 5 se guarda entera en
  `boligrafos.anterior.json`.

**Estilos de texto** (`feat/diseno-favoritos`)
- **Qué:**
  - `TextStyles { heading1, heading2, normal }`, cada uno un `TextStyleDef { size, line_height, weight }`.
  - Por defecto: 28/36 600, 22/30 600 y 16/24 400.
  - Galería con vista previa en Inicio y un lápiz para editar tamaño y grosor.
  - **Editar** llama a `Engine::restyle_texts(old, new)`: cambia, en todos los documentos abiertos, el texto cuyo
    rango tiene **exactamente** el tamaño y grosor del estilo antiguo (`restyle_attributes`), con deshacer.
- **Código:** `pens/typewriter/scripts.rs` (`TextStyles`, `quick_style_attributes(style, styles, base_size,
  base_weight)`) y `rnote-ui/src/textstyles.rs`.
- **Acciones / atajos:** `win.text-quick-style::heading1/heading2/normal` (**Ctrl+Alt+1/2/0**).
- **Estado guardado en:** clave `estilos-texto`.
- **Nota:** el interlineado no se aplica, porque el `.rnote` no lo guarda por párrafo.

**Árbol del cuaderno** (`feat/diseno-arbol`)
- **Qué:**
  - Sustituye la lista plana de Rnote por un `gtk::TreeListModel`: secciones → páginas → **subpáginas**.
  - Filas de 32 px y sangría de 16 px. Las filas son `TreeExpander` + `RnFileRow`, para reutilizar el menú de Rnote
    (abrir, renombrar, papelera, duplicar, mover a…).
  - El menú de fila añade «Subir», «Bajar», «Convertir en subpágina» y «Subir de nivel».
  - Arrastrar una página sobre otra la pone delante y a su nivel; sobre una sección, la mueve allí (sin probar).
  - Búsqueda (lista plana de coincidencias).
  - «+ Página» crea «Página sin título N» en la sección de la página abierta, justo detrás de ella.
  - Marca la página abierta (brand-tint con una barra de 3 px).
  - Recuerda las filas abiertas.
  - Vigila las carpetas con `FileMonitor` y se reconstruye a los 250 ms.
  - Panel de 260–320 px, plegable a la barra de cuadernos (48–64 px). Con los ajustes visibles se ensancha a
    580–640 px.
- **Código:** `rnote-ui/src/workspacebrowser/tree.rs` y `rnote-engine/src/onepepe_index.rs` (`FolderIndex`).
  - `read_entries` ignora los ocultos.
  - `sort`: primero el orden del usuario; luego secciones y después orden natural, «Tema 2» antes de «Tema 10».
  - También `children`, `parent_of`, `move_before`, `set_parent` (rechaza ciclos), `make_subpage`, `promote` y
    `rename`.
  - `tree::current_section(appwindow)` = carpeta de la página abierta dentro del cuaderno; si no hay, la raíz. La usa
    también `ribbon.rs`.
- **Estado guardado en:** el orden y las subpáginas en **`.onepepe-indice.json`** de cada carpeta; las filas abiertas
  en `arbol-abiertos` y el panel plegado en `panel-plegado` (ajustes).
- **Tests:** índice, 4.

**Fondo y renglones** (`feat/diseno-regla-renglones`)
- **Qué:**
  - Vista → Líneas abre una ventana con tipo, separación (mm o px), grosor, color de líneas, color de papel, línea de
    margen y el ámbito (esta página o todas las páginas de la sección).
  - Valores rápidos: liso, rayado 7 y 8,7 mm, cuadrícula y puntos de 5 mm. También ajustes con nombre del usuario.
  - **Vista previa en vivo**; cerrar sin «Aplicar» vuelve atrás.
  - «Toda la sección» aplica en vivo a las páginas abiertas y **reescribe las cerradas**:
    - antes, copia de seguridad con intervalo 0;
    - escribe en `.rnote.tmp` y lo renombra;
    - actualiza su archivo oculto.
  - **El grosor y la línea de margen no existen en el formato `.rnote`.** Son campos `#[serde(skip)]` de
    `Background`: `onepepe_line_width` y `onepepe_margin: Option<MarginLine { x, color, page_width }>`.
    - Se guardan en el archivo oculto.
    - El margen se dibuja en pantalla con `append_color` por cada columna de páginas (`MarginLine::positions`).
    - Al exportar va dentro del SVG del fondo (`gen_svg_inner(..., with_margin)`); nunca en la tesela repetida.
- **Código:** `rnote-ui/src/lines.rs` y `rnote-engine/src/engine/onepepe_lines.rs` (`LinesSettings`,
  `apply_lines_settings`, `rewrite_page_lines`).
- **Estado guardado en:** los ajustes con nombre, en la clave `presets-fondo`.
- **Tests:** 3 + margen 1 + archivo oculto 1.

**Ajustes en pestañas** (`feat/diseno-ajustes`)
- **Qué:**
  - Reorganiza **las filas de Rnote sin cambiar su lógica**: vacía la caja del `AdwClamp` del panel (5 grupos, en el
    orden del `.ui`).
  - Crea 5 páginas en un `gtk::Stack`:
    - **Lienzo y documento:** General, Formato y Documento.
    - **Fondo y renglones:** las filas de fondo, movidas **buscándolas por título traducido** («Color», «Pattern»,
      etc.).
    - **Entrada y lápiz:** Atajos de botones y Modos del lápiz.
    - **Atajos:** una lista y un acceso a la ventana de atajos.
    - **Apariencia:**
      - Tema: Sistema/Claro/Oscuro, con la acción `app.color-scheme` y `ADW color-scheme`;
      - Lienzo oscuro;
      - Diestro (`win.righthanded`);
      - Cinta.
  - Las pestañas son ToggleButton propios con subrayado; el `StackSwitcher` las hacía de ancho homogéneo.
  - Cabecera con logo y una barra fija con **Revertir/Aplicar**:
    - Al mostrarse el panel toma una «línea base»: `EngineConfig`, `DocumentConfig` de la página activa, propiedades
      propias de `RnAppWindow` y el valor de **todos los controles**. Los controles son
      SwitchRow/Switch/SpinRow/SpinButton/ComboRow/ToggleButton/ColorDialogButton; las pestañas no cuentan.
    - Cada 500 ms compara una huella para mostrar «Cambios sin aplicar». En la huella se quita `seed`, la semilla del
      pincel, que cambia en cada trazo.
    - **Revertir** restaura primero los controles (los ToggleButton con acción se restauran con `emit_clicked`) y
      luego las configuraciones. **Aplicar** toma una línea base nueva.
- **Código:** `rnote-ui/src/settingslayout.rs`.

### 6.4 Último lote (29/10/2026)

**Ángulo libre de la regla** (`feat/regla-angulo`): ver «Regla» en 6.1.

**Zoom amplio y zoom vectorial** (`feat/zoom-amplio`)
- **Qué:**
  - `Camera::ZOOM_MIN` = **0,05** (5 %) y `ZOOM_MAX` = **16** (1600 %); antes, 0,2 y 6.
  - `ZOOM_MAX_VECTOR` = **1000** (100 000 %) con Vista → «Zoom vectorial».
    - `Camera::set_vector_zoom` usa un `AtomicBool` global; `Camera::zoom_max()` sustituye a la constante en todo el
      código, incluidos `canvaswrapper.rs` y `pens/tools/zoom.rs`.
    - Es un ajuste de la app, guardado en `zoom-vectorial`. Al apagarlo, las pestañas con más de 1600 % vuelven a
      1600 %.
  - Para no agotar la memoria:
    - las imágenes por tramo del `BrushStroke` se recortan a la vista ampliada (y el tramo saltado avanza la semilla
      del texturizado);
    - `Image::gen_with_cairo` limita cada imagen a **4096 px** de lado bajando la escala;
    - la tesela del fondo limita su resolución a 4096/tamaño;
    - con más de 4000 teselas visibles (muy alejado) no se dibuja la cuadrícula, solo el color del papel.
- **Tests:** `onepepe_zoom_tests::images_at_extreme_zoom`.
- **Probado:** nube, 5 % y 91 710 % sin errores.
- **Limitaciones:**
  - En la nube los trazos salen como cuadrados rosas (sección 10.3), así que no se pudo ver si siguen nítidos.
  - `VectorImage` (SVG insertado) genera su imagen entera con `Svg::gen_image`, sin tope: con zoom extremo podría
    fallar al dibujarse. El tope no se puso ahí porque `Svg::gen_image` también se usa al exportar.

**Mano** (`feat/zoom-amplio`)
- **Qué:** es `ToolStyle::OffsetCamera` de Rnote.
- **Acciones / atajos:** botón en Dibujar y Vista; acción `win.tool-style::offsetcamera` (**Alt+7**); opción «hand»
  en la barra flotante.
- **Probado:** nube; arrastrar mueve la vista sin dibujar.

---

## 7. Matemáticas en detalle

### 7.1 Idea y flujo de punta a punta

**Qué ve el usuario:**
- Activa el **modo matemáticas**: botón en la pestaña Matemáticas o **Ctrl+Alt+M**. El lienzo muestra un borde
  naranja.
- Escribe con el pincel. Tras **0,8 s** sin escribir (`RECOGNITION_DELAY` en `mathhelper.rs`), los trazos nuevos se
  envían al programa auxiliar.
- Los símbolos reconocidos con confianza ≥ umbral (0,95 por defecto) **se sustituyen por trazos limpios**, que son
  trazos de pincel normales, así que el `.rnote` no cambia. Ctrl+Z devuelve la tinta original.
- Si la fila acaba en «=» y está activada la opción, se muestra el resultado en gris, como superposición, no como
  tinta.
- **Tocar** (sin arrastrar) un símbolo reconocido abre sus candidatas:
  - corregir ese símbolo o todo su grupo de símbolos parecidos;
  - «Unir con el de al lado»;
  - «Separar en trazos».

  Cada corrección se envía como ejemplo personal (kNN) y cuenta al instante.
- Tras unas 50 correcciones la app sugiere **Reentrenar** (pestaña Matemáticas → Tu letra).

**Rust, motor** (`rnote-engine/src/engine/math.rs`):
- `MathOptions { replace: true, threshold: 0.95, equals: false }` vive en `brush_config.math`, es decir, en los
  ajustes de la app.
- `MathState`:
  - `mode`;
  - cola `pending` de `StrokeKey`;
  - `in_flight`;
  - `symbols: Vec<MathSymbol { id, keys, latex, bounds, candidates, group, original }>`;
  - `overlays` (resultados del «=»);
  - `stroke_times: HashMap<StrokeKey,[f64;2]>`, guardado con el evento Down y con Up;
  - `group_fixes`, `corrections`, el toque…
- Funciones principales:
  - `math_take_request` construye la petición `recognize` con los trazos, sus `times` y los símbolos `known` del
    renglón;
  - `math_apply_response` sustituye los trazos por los limpios (`math_insert_clean`) y coloca el resultado tras el
    **último** «=» de la respuesta;
  - correcciones: `math_correction_requests` y `math_apply_correction`;
  - unir y separar: `math_join_with_next_requests`, `math_split_requests` y `math_apply_group_fix`;
  - `math_insert_lines` y `math_symbol_style`;
  - archivo oculto: `math_sidecar_json` y `math_load_sidecar`;
  - `math_sidecar_path(page)` devuelve `.<nombre>.onepepe.json`, en la misma carpeta;
  - `math_reset_document` se llama al cargar una instantánea.
- **El motor no sabe matemáticas:** guarda el estado y aplica las respuestas.

**Rust, interfaz** (`rnote-ui/src/mathhelper.rs`):
- **Lanzamiento** (`helper_command`):
  - la variable de entorno `ONEPEPE_MATH_CMD`, si existe (se usa en la nube);
  - dentro de Flatpak, `flatpak-spawn --host onepepe-math`. El manifiesto tiene `--talk-name=org.freedesktop.Flatpak`,
    que el usuario pidió explícitamente;
  - fuera de Flatpak, `onepepe-math` del `PATH` (`~/.local/bin`).
- El proceso es **persistente**: se lanza una vez y habla por stdin/stdout, una línea JSON por petición, con un `id`.
- `schedule_recognition`, `recognize_now` y `show_candidates` (la ventanita de candidatas, con botones de unir y
  separar).
- Archivo oculto: `save_sidecar` y `load_sidecar`, más `sidecar_follow_move`, `sidecar_follow_copy` y
  `sidecar_follow_trash` (el explorador los llama).
- **Reentrenar:**
  - `retrain` (`adw::AlertDialog` con el tiempo estimado: ejecuta `onepepe-math --estimar`);
  - `run_retraining` (`onepepe-math --reentrenar`, aviso con barra de progreso y «Cancelar», que manda **SIGTERM**);
  - `note_correction` sugiere reentrenar a las 50 correcciones nuevas.

### 7.2 Protocolo (`tools/onepepe-math/onepepe_math/server.py`)

Una petición JSON por línea; cada respuesta lleva el mismo `id`. Los errores se responden con `{"id", "error"}`; el
programa nunca se cae.

| `cmd` | Petición | Respuesta |
|---|---|---|
| `ping` | — | `{ok, personal_examples}` |
| `recognize` | `strokes` [[[x,y]…]…], `known` [{latex, box}], `notebook_dir`, `threshold`, `clean`, `equals`, `times` [[t0,t1]…] | `{symbols:[{strokes:[índices], box, candidates:[{latex,p}], confidence, group, clean:[[[x,y]…]] o null}], latex, result:{latex,text,kind} o null, confidence}` |
| `correct` | `strokes`, `latex`, `predicted`, `notebook_dir`, `add_to_glossary` | `{ok, clean}`; guarda el ejemplo y lo añade al glosario del cuaderno |
| `glyph` | `latex`, `box` | `{clean}` |
| `recognize_group` | `strokes` | igual que un símbolo: reconoce ese grupo sin cortarlo (para unir/separar) |
| `segment_fix` | `junk` [[trazos]…], `notebook_dir` | `{ok, saved}`; guarda los cortes malos como «trozo» |
| `examples` / `forget` / `relabel` | id de ejemplo | gestión del almacén personal (aún no hay pantalla en la app) |

**Lanzador `tools/onepepe-math/onepepe-math`** (bash):
- Usa `~/.local/share/onepepe/venv/bin/python` si existe; si no, `ONEPEPE_PYTHON` o `python3`.
- `--estimar` y `--reentrenar` ejecutan `reentrenar.py`.
- Sin argumentos, arranca `python -m onepepe_math.server`.

### 7.3 Módulos Python (`tools/onepepe-math/onepepe_math/`)

| Módulo | Qué hace |
|---|---|
| `features.py` | rasgos de un grupo de trazos (normalizados en tamaño) para la red |
| `classifier.py` | `SymbolClassifier`: red MLP evaluada con numpy desde `simbolos.npz` |
| `segment.py` | corte en símbolos por programación dinámica |
| `forced.py` | `forced_groups(strokes, tokens, clf)` y `junk_examples` |
| `context.py` | contexto y grupos de símbolos parecidos |
| `personal.py` | almacén personal y kNN |
| `layout.py` | estructura espacial |
| `evaluate.py` | resultado del «=» |
| `glyphs.py` | símbolos limpios con fuentes Hershey |
| `glossary.py` | glosario por cuaderno |
| `symbols.py` | vocabulario, alias, categorías, `same_shape` |
| `latex_tokens.py`, `inkml.py`, `synth.py` | utilidades y fórmulas sintéticas para medir |
| `recognizer.py` | une todo (orden abajo) |
| `server.py` | el servidor del protocolo |

**`classifier.py`:**
- La red tiene una versión: `npz["version"]` o el sha1 del archivo.
- `mask` restringe las clases al vocabulario: por defecto `CORE`; los nombres de clase se normalizan con
  `normalize_label`, así que `\longrightarrow` cuenta como `\rightarrow`.
- `candidates_w_junk` **suma la probabilidad de las clases que son el mismo símbolo**.
- La clase especial `JUNK = "__trozo__"` significa «esto no es un símbolo completo».
- Lleva incluido el modelo de contexto por bigramas (`lm_*`).

**`segment.py`:**
- Corta los trazos en símbolos por programación dinámica, con grupos de 1 a `MAX_STROKES` = 4 trazos seguidos.
- `SegmentParams` (los valores de MathWriting; se pueden sobreescribir con `corte.json`):

  | Parámetro | Valor |
  |---|---|
  | `touch_gap` | 0.25 |
  | `stack_gap` | 1.0 |
  | `symbol_penalty` | 1.2 |
  | `junk_weight` | 1.0 |
  | `pause_same` | 0.3 s |
  | `pause_weight` | 0.5 |

- Puntuación: `log p_top + junk_weight·log(1-p_junk) − penalty + puntuación de pausa`, la de pausa si hay tiempos.

**`forced.py`:** el «corte forzado». Dada una expresión etiquetada, busca el corte que da exactamente los símbolos
esperados; de ahí salen ejemplos de símbolos y de «trozos».

**`context.py`:**
- `BigramModel`, `reading_order` y `viterbi(cand_lists, lm, lm_weight=1.0, prior_weight=0.3)`.
- `apply_glossary` (refuerzo ×2).
- `shape_groups(embeddings, 0.85)`: grupos de símbolos parecidos por similitud coseno.
- `share_within_groups(…, strength=0.7)`.

**`personal.py`:**
- `PersonalModel`: almacén en `correcciones.jsonl`.
- Cada registro: `id`, `latex`, `predicted`, `strokes`, `embedding`, `model`, `date`, `notebook`, `kind` (`symbol` o
  `junk`).
- Funciones: `add_correction`, `list_examples`, `forget`, `relabel`, `count`.
- Si cambia la versión del modelo, recalcula los embeddings en segundo plano.
- kNN con `K`=5, `TAU`=0.1, `D_MAX`=0.3 y `LAMBDA_MAX`=0.6:
  - `p_final = (1-λ)·p_red + λ·p_knn`, con `λ = λ_max · cercanía · coherencia`;
  - λ = 0 si el vecino más cercano está a más de `D_MAX`;
  - los ejemplos `junk` no entran en el kNN.

**`layout.py`:**
- Estructura espacial: `Sym(label, box)` → árbol (`row`, `sup`, `sub`, `subsup`, `frac`, `sqrt`) y LaTeX
  (`to_latex`).
- `_NEVER_SCRIPT`: símbolos que nunca son exponentes, como `, . = < > ≤ ≥ ≠ ≈ → ⇒ ⇔ ; :`.

**`evaluate.py`:**
- Construye una expresión de SymPy con un analizador propio del árbol, porque `parse_latex` solo entendía 19 de las 30
  expresiones de prueba.
- `answer(tree, glossary, units)` solo actúa si la fila **acaba en «=»**. Ver 7.5.

**`glyphs.py`:**
- Símbolos limpios con fuentes Hershey (paquete `Hershey-Fonts`, MIT).
- Créditos obligatorios: A. V. Hershey (U.S. National Bureau of Standards) y James Hurt (formato de los datos).
- Dibujados a mano en `_MANUAL`: `≈`, `∼`, `⇒` y `⇔`.
- `place_glyph(latex, box)` centra y escala el glifo en la caja escrita. Los símbolos «planos» (`_FLAT`) se ajustan
  por anchura.

**`glossary.py`:**
- Archivo `.onepepe-glosario.txt` en la carpeta del cuaderno o de la sección: símbolos y constantes del cuaderno.
- Refuerza esas candidatas y da valores y unidades (con pint) al «=».

**`symbols.py`:**
- `ALIASES` (`\to`→`\rightarrow`, `\longrightarrow`→`\rightarrow`, `\implies`→`\Rightarrow`, `\iff`→`\Leftrightarrow`,
  …) y `normalize_label`.
- `CORE`: vocabulario por defecto. Cifras, latinas, griegas, operadores y relaciones, `; : → ← ⇒ ⇔`, `∫ ∑ ∏ ∂ ∇ ∞`,
  `lim`, `%`, `∘`…
- `RELATIONS`, `STEP_SEPARATORS = {";", "\rightarrow", "\Rightarrow", "\Leftrightarrow"}`, `category()`,
  `same_shape()`.

**`recognizer.py`**, en este orden:
1. `segment`;
2. kNN personal;
3. glosario;
4. grupos parecidos;
5. contexto con Viterbi (junto con los símbolos `known` del renglón);
6. confianza = `p_top × (1-p_trozo)`;
7. `clean` si confianza ≥ umbral;
8. `parse` y `answer`.

### 7.4 Scripts (`tools/onepepe-math/`)

| Script | Qué hace |
|---|---|
| `instalar.sh` | crea `~/.local/share/onepepe/venv`, instala `requirements.txt` y la orden `~/.local/bin/onepepe-math`. Con `--entrenar`, además descarga MathWriting (2,9 GB, `descargar_mathwriting.sh`) y entrena; usa `--personal` si existe `correcciones.jsonl` |
| `train_classifier.py <mathwriting> [-o salida] [--personal F] [--personal-augment 90]` | entrena la red (scikit-learn) con deformaciones aleatorias, los ejemplos personales (los `junk` en la clase `__trozo__`) y el modelo de contexto; guarda la versión. Reserva un 20 % de MathWriting por id (`is_holdout`) |
| `reentrenar.py` | `--estimar` devuelve una línea JSON `{examples, junk, new_since_last, minutes, dataset, model}`. Sin opciones: reserva el **30 % más reciente** de los ejemplos personales para comparar (con 10 o más), entrena en un archivo temporal, compara red+kNN viejo frente a nuevo en esos ejemplos, **solo sustituye si no empeora** (el anterior queda como `simbolos.anterior.npz`) y escribe `ultimo_entreno.json`. Progreso en líneas JSON `{stage, progress, message}`; SIGTERM cancela, y el hijo usa `PR_SET_PDEATHSIG`. Variables: `ONEPEPE_DATA`, `ONEPEPE_MATHWRITING` |
| `importar_mi_letra.py pagina.rnote expresiones.txt` | corte forzado de cada renglón, guarda los símbolos y los «trozos» en el almacén |
| `evaluar_mi_letra.py` | evaluación temporal (red sola, red+kNN, modelo anterior) |
| `rnote_extract.py` | lee un `.rnote` con la biblioteca estándar, extrae los trazos y los separa en renglones (`merge_close_lines` junta fracciones y exponentes) |
| `expresiones.txt` | las 30 expresiones de prueba que escribe el usuario |

Requisitos: `requirements.txt` para ejecutar (numpy, sympy, pint, Hershey-Fonts…) y `requirements-train.txt` para
entrenar (más scikit-learn).

**Tests:** `python -m unittest discover -s tests`, 34 tests con 1 saltado si no hay modelo. Archivos
`test_evaluate.py`, `test_personal.py`, `test_reentrenar.py`, `test_rnote_extract.py`, `test_segment.py` y
`test_server.py`; fixture `tests/fixtures/dos_mas_tres.rnote`.

### 7.5 Resultado del «=» (`evaluate.py`)

- Solo si la fila **acaba en «=»**.
- **Separadores:** `,`, `;`, `→`, `⇒` y `⇔` separan expresiones o pasos, y **solo cuenta el último**.
- Casos:

  | Escribes | Resultado |
  |---|---|
  | `expr =` sin incógnitas | valor exacto si es una fracción sencilla; si no, 4 cifras significativas; un irracional corto da exacto ≈ aproximado |
  | `ecuación, x =` o `ecuación → x =` | despeja x |
  | `ecuación =` con una sola incógnita | la despeja |
  | `expr =` con variables | forma simplificada, solo si es más corta |
  | derivadas `d/dx`, integrales (indefinidas + C), sumatorios, productos, límites | se calculan |
  | constantes y unidades del glosario (pint) | se calculan; si las dimensiones no casan, no hay resultado |
  | cualquier cosa ambigua | `None`: no se muestra nada |

- La coma decimal se une a las cifras **antes** de partir por comas (`decimal_comma=True`).
- **No** comprueba todavía si cada paso de un desarrollo se sigue del anterior (idea en el plan).

### 7.6 Medidas (lo que se sabe)

**Con letra de MathWriting, modelo genérico:**
- Símbolo suelto: top-1 61 %, top-5 92 % (229 clases).
- Agrupación de trazos: 90 %.
- Fórmula entera perfecta: aproximadamente 10 %.
- Unos 5 ms por fórmula.

**Con la letra del usuario** (30 expresiones, 391 trazos):

| Medida | Resultado |
|---|---|
| Modelo genérico: símbolos bien | 49 % |
| Modelo genérico: fórmulas exactas | 0/30 |
| Modelo genérico: sustituidos con seguridad y bien | 76 % |
| Modelo genérico: expresiones bien cortadas | 7/29 |
| División temporal (entrena 1-15, prueba 16-30), red sola | 44 % de símbolos, 68 % de sustituciones bien |
| Misma división, reentrenada con corte forzado + «trozos» | **56 % de símbolos, 88 % de sustituciones bien** |

- **Lo que no mejoró** y se descartó: umbrales de corte calibrados a su letra.
- **El kNN:** casi no cambia el acierto, pero se equivoca menos al sustituir.
- **Reentrenar desde la app** (682 ejemplos importados): 2 min 11 s; en sus ejemplos reservados pasó de 36/72 a 40/72.
- **Flechas y `;`** en los ejemplos reservados de MathWriting: flechas 24/24, `;` 2/4 (se confunde con «j»). El resto
  no empeoró: 559 frente a 560 de 842.

Detalle completo en `MATEMATICAS-INFORME.md`, secciones 0.4–0.7.

### 7.7 Consejos que se le dieron al usuario para escribir

- Símbolos de tamaño parecido, más o menos un renglón (7–9 mm).
- Exponentes claramente más pequeños.
- La raya de fracción más larga que lo de arriba y lo de abajo.
- Un hueco entre símbolos, sin que se toquen; una expresión por renglón.
- Terminar los trazos de un símbolo seguidos.
- El «=» con dos rayas iguales.
- Esperar 0,8 s.
- Corregir al principio y reentrenar tras unas 50 correcciones.
- **Apagar «Tinta a formas»** en matemáticas.
- (2026-10-30) Zoom moderado, del 100 % al 200 %: con mucho zoom reconoce peor (§13.1, punto 2).
- (2026-10-30) Funciones con paréntesis, `ln(x)` y `sin(x)`: `sin x` sin paréntesis no se calcula.
- (2026-10-30) Si falla un símbolo, tocarlo y corregirlo 3–4 veces: el kNN aprende al momento.

---

## 8. Formatos y archivos que escribe OnePepe

### 8.1 `.rnote`: NO TOCAR

- Formato de Rnote: JSON con `ijson`, comprimido, versionado (`RnoteFileMaj0Min15`). **No se ha añadido ningún campo
  que se escriba.**
- Campos nuevos en estructuras que se serializan, siempre con **`#[serde(skip)]`**:
  - `Ruler::snap_to_steps`;
  - `Background::onepepe_line_width`;
  - `Background::onepepe_margin`.
- `Ruler` sí se serializa, con `#[serde(default)] infinite`, pero **solo en el archivo oculto**: la regla nunca va al
  `.rnote`.
- Cambios de **valores por defecto** (no de formato): `Format::default` pone `show_borders` y
  `show_origin_indicator` a false y `BORDER_COLOR_DEFAULT` gris al 30 %. Solo afecta a documentos nuevos sin
  configuración guardada.
- La cámara guarda `zoom` en el `.rnote`: con el zoom vectorial puede llegar a 1000; Rnote lo recorta al abrir.

### 8.2 Archivo oculto por página: `.<nombre>.rnote.onepepe.json`

Va en la misma carpeta que la página (`math_sidecar_path`). JSON con `version: 2`, que sigue leyendo la versión 1.

```json
{
  "version": 2,
  "symbols":  [{"id", "keys": [StrokeKey], "keys_bounds": [x0,y0,x1,y1], "latex", "bounds", "candidates",
                "group", "original"}],
  "overlays": [{"anchor": [x,y], "height", "text", "latex"}],
  "times":    [[StrokeKey, [t0, t1]]],
  "ruler":    {"center", "angle", "length", "width", "infinite"} | null,
  "line_width": 1.5 | null,
  "margin":   {"x", "color", "page_width"} | null
}
```

- Se escribe **al guardar** (también con el autoguardado) y se borra si no hay nada que guardar. Se lee **al abrir**.
- Un símbolo se restaura solo si sus trazos existen y la caja coincide (±0,5): un archivo oculto desfasado nunca cambia
  la página.
- Sigue a la página al renombrar, mover, duplicar o borrar **desde el explorador de OnePepe**.
- «Fondo y renglones» para toda la sección modifica los archivos ocultos de las páginas cerradas:
  `rewrite_page_lines` cambia solo `line_width` y `margin`, y conserva lo demás.

### 8.3 Índice por carpeta: `.onepepe-indice.json`

```json
{ "orden": ["Álgebra", "Tema 2.rnote", "Tema 1.rnote"],
  "subpaginas": { "Ejercicios.rnote": "Tema 2.rnote" } }
```

- Lo que no está en `orden` va detrás: primero las secciones y luego en orden natural.
- Una subpágina cuyo padre ya no existe aparece arriba del todo.
- Si queda vacío, el archivo se borra.

### 8.4 Ajustes de la app

**`~/.config/onepepe/ajustes.json`** (`prefs.rs`):

| Clave | Tipo | Quién |
|---|---|---|
| `lienzo-oscuro` | bool (por defecto true) | theme.rs / `win.dark-canvas` |
| `zoom-vectorial` | bool | `win.vector-zoom` |
| `barra-herramientas` | [ids] (por defecto brush, eraser, selector) | optionsbar.rs |
| `barra-lado` | "auto" / "left" / "right" | optionsbar.rs |
| `barra-plegada` | bool | optionsbar.rs |
| `barra-y` | entero px o null | optionsbar.rs |
| `estilos-texto` | `TextStyles` | textstyles.rs |
| `presets-fondo` | [[nombre, `LinesSettings`]] | lines.rs |
| `arbol-abiertos` | [rutas] | tree.rs |
| `panel-plegado` | bool | tree.rs |

**Otros archivos de `~/.config/onepepe/`:** `boligrafos.json` (`PenPresets`) y `boligrafos.anterior.json`.

**GSettings de Rnote** (esquema `com.github.flxzt.rnote`): `engine-config` (JSON de `EngineConfig`, incluidas las
opciones de matemáticas y de tinta a formas), `color-scheme`, `righthanded`, áreas de trabajo, etc. OnePepe **no
añade claves** al esquema: si falta una clave, la app se cierra al arrancar.

### 8.5 Datos locales: `~/.local/share/onepepe/`

| Ruta | Qué |
|---|---|
| `venv/` | entorno de Python del programa auxiliar |
| `mathwriting/mathwriting-2024/` | dataset (2,9 GB; fuera del repositorio) |
| `math/simbolos.npz` | el modelo |
| `math/simbolos.anterior.npz` | el modelo anterior |
| `math/correcciones.jsonl` | ejemplos personales |
| `math/corte.json` | opcional |
| `math/ultimo_entreno.json` | datos del último entrenamiento |
| `copias/<cuaderno>/<página>/…` | copias de seguridad (`backup_dir_for`, `onepepe_backup_root()`) |

### 8.6 Variables de entorno

| Variable | Para qué |
|---|---|
| `ONEPEPE_MATH_CMD` | orden que lanza el programa auxiliar (pruebas) |
| `ONEPEPE_DATA` | sustituye a `~/.local/share/onepepe` |
| `ONEPEPE_MATHWRITING` | ruta del dataset |
| `ONEPEPE_PYTHON` | intérprete |
| `ADW_DEBUG_COLOR_SCHEME=prefer-dark` | forzar el tema oscuro al probar |
| `ADW_DEBUG_HIGH_CONTRAST=1` | forzar el alto contraste al probar |
| `GSK_RENDERER=cairo` y `GDK_BACKEND=x11` | necesarias en la nube |

---

## 9. Atajos de teclado

Comprobados con `grep set_accels_for_action`.

### 9.1 OnePepe (nuevos o cambiados)

| Atajo | Acción | Nota |
|---|---|---|
| **Ctrl+1 … Ctrl+5** | `win.pen-preset(0..4)` | bolis favoritos. **Antes, en Rnote, eran las herramientas** |
| **Alt+1 … Alt+6** | `win.pen-style::brush/shaper/typewriter/eraser/selector/tools` | las herramientas, movidas aquí |
| **Alt+7** | `win.tool-style::offsetcamera` | Mano |
| **Ctrl+Alt+1 / 2 / 0** | `win.text-quick-style::heading1 / heading2 / normal` | estilos |
| Ctrl+Alt+3 … 9 | `win.set-color-3..9` | colores de la paleta. **El 1 y el 2 perdieron el atajo** |
| Ctrl+Alt+M | `win.math-mode` | modo matemáticas |
| Ctrl+Alt+R | `win.ruler` | regla |
| Ctrl+F1 | `win.ribbon` | mostrar u ocultar la cinta |

### 9.2 De Rnote, sin cambios

- **Zoom:** Ctrl+0 (100 %), Ctrl++ / Ctrl+= (ampliar), Ctrl+− (alejar).
- **Archivo:** Ctrl+S, Ctrl+Shift+S, Ctrl+O, Ctrl+P, Ctrl+Shift+I (importar).
- **Ventana y pestañas:** Ctrl+T, Ctrl+W, Ctrl+N, Ctrl+Q, Ctrl+? (ventana de atajos), Ctrl+Shift+O, F11.
- **Edición:** Ctrl+Z / Ctrl+Shift+Z, Ctrl+C/X/V, Ctrl+B/I/U.
- **Documento:** Ctrl+L (limpiar), Ctrl+Shift+A / Ctrl+Shift+R (añadir / quitar página), Ctrl+Home, Ctrl+Shift+P.

### 9.3 Notas

- La ventana de atajos (`data/ui/shortcuts.ui`) está actualizada.
- La pestaña «Atajos» de los ajustes muestra los de OnePepe.

---

## 10. Compilar, instalar y probar

### 10.1 En el equipo del usuario

```bash
cd /home/pepe/Pepe_Flock/One_Pepe
git pull
~/instalar-onepepe.sh          # compila con Flatpak, instala y abre; notify-send si falla
# o a mano:
./setup-deps.sh --flatpak
flatpak-builder --user --install --force-clean _flatpak_app build-aux/com.github.flxzt.rnote.Devel.yaml
flatpak run com.github.flxzt.rnote.Devel
# programa de matemáticas (una vez; --entrenar descarga 2,9 GB y entrena unos minutos):
cd tools/onepepe-math && ./instalar.sh --entrenar
```

`./setup-deps.sh --check` dice si su sistema puede compilar sin Flatpak; si responde «DEMASIADO VIEJA», usa Flatpak.

### 10.2 Tests (en cualquier sitio)

```bash
cargo test -p rnote-compose -p rnote-engine        # 19 (+1 ign.) y 64 + 4 + 2 (+1 ign.)
cd tools/onepepe-math
python3 -m venv /tmp/venv && /tmp/venv/bin/pip install -r requirements-train.txt
/tmp/venv/bin/python -m unittest discover -s tests   # 34 (1 saltado sin modelo)
msgfmt -c -o /dev/null crates/rnote-ui/po/es.po      # traducción
```

### 10.3 La interfaz en una nube sin GTK 4.22 (cómo se ha trabajado)

**a) Comprobar tipos de la interfaz** (no enlaza ni ejecuta). Hace falta un `crates/rnote-ui/src/config.rs` generado
desde `config.rs.in`; los comandos están en `ESTADO.md` y no se sube. Después:

```bash
for n in GTK4 LIBADWAITA_1 GRAPHENE_GOBJECT_1_0; do
  export SYSTEM_DEPS_${n}_NO_PKG_CONFIG=1 SYSTEM_DEPS_${n}_LIB=x SYSTEM_DEPS_${n}_SEARCH_NATIVE=/usr/lib
done
cargo check -p rnote
```

**b) Ejecutar de verdad con GTK 4.14** (sistema Ubuntu de la nube con `libgtk-4-dev libadwaita-1-dev xvfb xdotool
dbus-x11 imagemagick`):

- Un **`git worktree`** aparte (`smoke`) que **no se sube**:
  - en su `Cargo.toml`, `v4_22` → `v4_14` y `v1_9` → `v1_5`;
  - se borran de `crates/rnote-ui/data/ui/*.ui` las líneas `<property name="limit-events">`, que GTK 4.14 no conoce
    y hacen que la plantilla no cargue.
- Un `sync.sh`:
  - borra y copia `crates/` del árbol de trabajo (en la nube no hay `rsync`; se usa `cp -a`) y `misc/`;
  - quita `limit-events`;
  - compila `es.po` a `/usr/share/locale/es/LC_MESSAGES/rnote.mo`;
  - ejecuta `cargo build -p rnote` con un `CARGO_TARGET_DIR` aparte.
- Un `runapp.sh`:
  - `GSK_RENDERER=cairo GDK_BACKEND=x11 DISPLAY=:99 LANG=es_ES.UTF-8`;
  - `GSETTINGS_SCHEMA_DIR=<esquema compilado>` y `GSETTINGS_BACKEND=memory`;
  - `HOME`/`XDG_*` privados;
  - `ONEPEPE_MATH_CMD=<lanzador del programa auxiliar con el venv de la nube>`;
  - `dbus-launch rnote`;
  - luego busca la ventana con `xdotool search --onlyvisible --class rnote` y la pone a 1480×920.
- `Xvfb :99 -screen 0 1500x950x24` (si se pierde tras reiniciarse el contenedor, se vuelve a lanzar).
- Se maneja con `xdotool mousemove X Y click 1`, `key ctrl+2`, `type "…"`, `keydown ctrl click 4 keyup ctrl` (rueda
  con Ctrl). Las capturas se hacen con `import -window root` y `convert -crop/-resize`.
- Para los temas: `ADW_DEBUG_COLOR_SCHEME=prefer-dark`, `ADW_DEBUG_HIGH_CONTRAST=1`.

**c) Limitaciones conocidas de esa app de prueba:**

- **Los trazos y el texto salen como cuadrados rosas.** Es un fallo de texturas de GTK 4.14 con el renderizador
  cairo, no de OnePepe: no se puede juzgar el aspecto de la tinta.
- **Arrastrar y soltar con xdotool no funciona:** el arrastre no llega a empezar. Por eso no está probado arrastrar en
  el árbol ni en los favoritos.
- Sin portal de capturas, sin lápiz real, sin bus de accesibilidad (avisos inofensivos).
- Las ventanas emergentes salen con un rectángulo negro alrededor (no hay compositor).
- Las variables CSS `var(--…)` y `@media (prefers-color-scheme)` **no existen en GTK 4.14**: por eso el tema usa
  `@define-color` y una clase `dark` (sección 12).
- Los tests de `theme.rs` se ejecutan en el árbol de prueba: `cargo test -p rnote theme`.

**d) Probar el modo matemáticas con letra real en la app de la nube** (así se comprobó el «=» el 2026-10-30).

Los scripts de la nube se pierden en cada sesión; este es el esencial.

1. Necesitas:
   - el venv con `requirements-train.txt`;
   - MathWriting descargado (`descargar_mathwriting.sh`);
   - un modelo entrenado (`train_classifier.py`);
   - un lanzador con `ONEPEPE_DATA`, `ONEPEPE_MATHWRITING` y `ONEPEPE_PYTHON`, que haga `exec
     tools/onepepe-math/onepepe-math`. Ese lanzador se pasa en `ONEPEPE_MATH_CMD`.
2. Este script dibuja con xdotool símbolos de verdad de MathWriting:

   ```python
   # xdraw.py <dir mathwriting-2024> <k> "2,+,3,=" <x0> <y0>  > draw.txt ; luego: xdotool - < draw.txt
   import sys
   from onepepe_math.synth import load_holdout_symbols, _fit
   d = load_holdout_symbols(sys.argv[1]); k = int(sys.argv[2]); x, y0 = int(sys.argv[4]), int(sys.argv[5])
   for lab in sys.argv[3].split(","):
       box = (x, y0, x + 45, y0 + 60) if lab not in "=-+" else (x, y0 + 18, x + 45, y0 + 45)
       for s in _fit(d[lab][k % len(d[lab])], box):
           pts = s[::max(1, len(s) // 25)] + [s[-1]]
           print("mousemove %d %d mousedown 1" % pts[0])
           for p in pts[1:]: print("mousemove %d %d sleep 0.01" % p)
           print("mouseup 1 sleep 0.15")
       x += 60
   ```

   Se ejecuta con `PYTHONPATH=tools/onepepe-math`.
3. Pasos en la app:
   1. activa el modo con Ctrl+Alt+M;
   2. activa la opción del «=»: el ⚙ de la barra flotante → «Modo matemáticas» → «Mostrar el resultado tras el
      "="»;
   3. dibuja y espera unos 5 s.
4. **Prueba sin la app:** se le pasan al programa auxiliar los símbolos ya reconocidos:

   ```bash
   echo '{"id":1,"cmd":"recognize","strokes":[],"known":[{"latex":"2","box":[0,0,30,40]},{"latex":"+","box":[40,12,70,30]},{"latex":"3","box":[80,0,110,40]},{"latex":"=","box":[120,15,150,28]}],"equals":true}' | onepepe-math
   # → "result": {"latex": "5", ...}
   ```

   Es la misma orden que se le dio al usuario para comprobar su instalación.

---

## 11. Estado: ramas, pruebas y lo que falta probar

**Ramas fusionadas en `onepepe`**, de la más reciente a la más antigua:

- **Solo documentación (2026-10-30):** `feat/contexto-ia-2`, `feat/pendiente-myscript`, `feat/pendiente-funciones`,
  `feat/pendiente-resultado`, `feat/pendiente-matematicas-2`, `feat/pendiente-reconocer-seleccion`,
  `feat/pendiente-matematicas`.
- **Sin Rnote a la vista (2026-10-29):** `feat/sin-rnote`, y la rama `rnote-original` creada.
- **Antes:** `feat/contexto-ia`, `feat/matematicas-pasos`, `feat/zoom-amplio`, `feat/regla-angulo`.
- **Diseño:** `feat/diseno-ajustes`, `feat/diseno-regla-renglones`, `feat/diseno-arbol`, `feat/diseno-favoritos`,
  `feat/diseno-barra`, `feat/diseno-cinta`, `feat/diseno-tokens`.
- **Corte y reentrenamiento:** `feat/corte-app`, `feat/corte-pausas`, `feat/corte-tiempos`,
  `feat/reentrenar-boton`, `feat/corte-trozo`, `feat/knn-correcciones`.
- **Tercera tanda:** `feat/diseno-fluent`, `feat/diseno-onenote`, `feat/notas-rapidas-y-seccion-pdf`,
  `feat/traduccion-es`, `feat/cinta`, `feat/copias-seguridad`, `feat/boligrafos-favoritos`, `feat/insertar`,
  `feat/estilos-listas`.
- **Marca:** `feat/logos-pepe-flock`, `feat/nombre-onepepe`, `feat/readme`.
- **Matemáticas iniciales:** `feat/flatpak-matematicas`, `feat/matematicas-archivo-oculto`,
  `feat/matematicas-limpias`, `feat/matematicas-prototipo`.
- **Primeras fases:** `feat/listas`, `feat/fecha-hora`, `feat/catalogo-simbolos`, `feat/subindices`,
  `feat/botones-lapiz`, `feat/cuadernos`, `feat/formas`, `feat/regla`, `feat/capas`, `feat/colores`,
  `feat/hojas-infinitas`.

La tabla detallada de qué se probó en cada una está en `ESTADO.md` → «Hecho».

**Todavía sin probar en ningún sitio:**
- Nada con el **lápiz real**.
- Nada con **GTK 4.22** (por ejemplo, el bloque de variables CSS de libadwaita 1.6+).
- **Arrastrar** en el árbol y en los favoritos.
- El **recorte de pantalla** con portal.
- Que los trazos sigan nítidos con el zoom vectorial.
- El reconocimiento matemático **dentro de la app con la letra del usuario**. El programa auxiliar sí se probó de
  punta a punta en la nube, y unir/separar se probó con un «=».

**Cosas que el usuario tiene que hacer:**
- Probar con el lápiz siguiendo `CAMBIOS-ONEPEPE.md`.
- Comprobar que la rama por defecto en GitHub es `main` (Settings → General → Default branch).
- Ejecutar la prueba del «=» de §10.3d en su equipo: en la app no le sale el resultado (§13.1, punto 6).
- Decir sus asignaturas, para preparar la lista de unas 80 expresiones de entrenamiento (§13.1, punto 8).
- Enviar `mis_simbolos.rnote`.
- Contestar las preguntas cortas de `MATEMATICAS-INFORME.md` §5: signo de multiplicar, coma o punto decimal, unidades,
  asignaturas.
- Decir el modelo de su Wacom y la salida de `evtest`.

---

## 12. Decisiones clave y por qué

Resumen de `DECISIONES.md`; allí está todo.

- **Formato:** nunca se cambia el `.rnote`. Lo nuevo va al archivo oculto por página, al índice por carpeta, a
  `~/.config/onepepe/` o a campos `#[serde(skip)]`.
- **Interfaz nueva en código Rust y no en `.ui`:** el compilador detecta los errores; un `.ui` roto solo se ve al
  ejecutar.
- **Tema:**
  - Una tabla de tokens en `theme.rs` y `@define-color`, que funciona en GTK 4.14 y 4.22. `var(--…)` solo se escribe
    con GTK ≥ 4.16.
  - El cambio claro/oscuro lo avisa `adw::StyleManager` (no `@media`).
  - `style.css` sin colores sueltos, comprobado por un test.
- **Lienzo oscuro:** una matriz de color solo en pantalla, que conserva el tono; los archivos y lo exportado no
  cambian.
- **Atajos:** favoritos en Ctrl+1…5 (lo pedía la guía), herramientas en Alt+1…6, mano en Alt+7, estilos en
  Ctrl+Alt+1/2/0.
- **Favoritos:** 5 huecos en un JSON propio, no en GSettings (una clave nueva rompe el arranque si falta en el
  esquema).
- **Estilos de texto:** la identidad es el tamaño y el grosor exactos, sin guardar nada aparte. Es compatible con las
  páginas de Rnote, aunque puede dar algún falso positivo.
- **Árbol:** un `TreeListModel` propio; subpáginas y orden en el índice oculto; reutiliza `RnFileRow` y su menú.
- **Regla:**
  - Solo actúa sobre el pincel.
  - El trazo se pega si empieza cerca del borde.
  - Ángulo libre (Mayús = 15°).
  - Infinita con el asa en la vista.
  - Estado en el archivo oculto.
- **Renglones:** grosor y margen fuera del `.rnote`. «Toda la sección» reescribe las páginas cerradas tras hacer una
  copia, de forma atómica.
- **Ajustes:** se mantiene el comportamiento inmediato de Rnote y se añade Revertir/Aplicar; cerrar sin aplicar
  conserva los cambios.
- **Zoom:** 5 %–1600 %; zoom vectorial hasta 1000×, ajuste de la app. Se recortan las imágenes a la vista y se limitan
  a 4096 px.
- **Matemáticas:**
  - Reconocimiento por símbolos (no imagen→LaTeX), para corregir por símbolo y aprender la letra.
  - Programa persistente en Python.
  - SymPy desde el árbol (no `parse_latex`).
  - MathWriting (CC BY-NC-SA, uso personal) descargado aparte.
  - Clase «trozo».
  - kNN personal y reentrenamiento que solo sustituye si no empeora.
  - Las flechas y `;` separan pasos.
- **Tinta a formas:** algoritmo geométrico propio, sin modelos; desactivada por defecto.
- **Copias de seguridad:** fuera de los cuadernos, una cada 20 minutos como mucho, 10 por página.
- **Nombre:** solo el nombre visible es OnePepe; el id `com.github.flxzt.rnote` no cambia (ajustes, Flatpak).
- **Sin Rnote a la vista (2026-10-29):**
  - `main` = OnePepe y Rnote original en `rnote-original`.
  - Se borraron los documentos, el CI y las plantillas de GitHub de Rnote; siguen en `rnote-original`.
  - Se quedan los nombres internos (`rnote`, `com.github.flxzt.rnote`, `.rnote`) y los créditos que pide la GPL
    (`LICENSE`, `AUTHORS`, «Acerca de → Rnote» y la nota del README).
  - `build-aux/inno_build.py` y `rnote_inno.iss.in` se quedan porque `meson.build` los exige.
  - **No hay CI:** si se quiere, se recupera `ci.yml` de `rnote-original`.

---

## 13. Pendiente y prioridades

Todo está en `PLAN-PENDIENTE.md` (manda ese archivo: si cambia, actualiza esto). **Nada de esta sección está
implementado.** El usuario pidió apuntarlo; no lo hagas sin que lo pida.

**Orden de prioridad del usuario:**

0. Los fallos del modo matemáticas (§13.1).
1. Pruebas con el lápiz real.
2. Matemáticas con su letra; el entrenamiento, en paralelo.
3. El plan tipo MyScript (§13.2), en su orden.
4. Nombre interno OnePepe.
5. Mejoras opcionales.
6. Ideas.

### 13.1 Fallos del modo matemáticas (encontrados por el usuario el 2026-10-30)

**1. No deja escribir cerca de un símbolo reconocido, ni donde acabas de borrar uno.** Lo que más le pasa.

- **Causa, cerca de un símbolo:** `math_filter_pen_event` (`crates/rnote-engine/src/engine/math.rs`, hacia la
  línea 336).
  - Si el lápiz baja dentro de la caja de un símbolo (ampliada 4 px / zoom), guarda `tap_start` y devuelve `None`
    para **todos** los eventos del trazo.
  - Aunque el lápiz se mueva, el trazo se pierde; solo se abren las candidatas si no se movió.
- **Causa, donde has borrado:**
  - el símbolo borrado sigue en `math.symbols`, y `math_symbol_index_at` no comprueba si sus trazos viven;
  - `math_take_request` comprueba `get_stroke_ref(...).is_some()`, pero los trazos borrados siguen en el almacén,
    marcados en la papelera (`store.trashed(key)`, `store/trash_comp.rs`); así manda como contexto símbolos
    borrados.
- **Arreglo propuesto:**
  - Guardar los eventos en un búfer y decidir al levantar:
    - si se mueve más de 6 px de pantalla, se reenvían al pincel desde el primer punto;
    - si no se mueve, es un toque;
    - `math_filter_pen_event` devolvería `Vec<PenEvent>`, y `Engine::handle_pen_event` (`engine/mod.rs`, hacia la
      línea 519) enviaría cada uno.
  - La zona del toque, la tinta (`hitboxes()` de sus trazos) y no la caja.
  - Ignorar los símbolos con todos sus trazos en la papelera, tanto en el toque como en `known` y en el archivo
    oculto.
- **Tests:**
  - un trazo que empieza en la caja y se mueve se dibuja entero;
  - un toque abre las candidatas (el test actual `tap_on_symbol_and_group_correction`);
  - un punto junto a la tinta no cuenta;
  - borrar y dibujar encima funciona.

**2. Con mucho zoom (escribiendo pequeño) reconoce peor.** Causa no confirmada.

- **Medido:** Python no depende del tamaño. Con 150 fórmulas de MathWriting reducidas hasta 1/200, los símbolos bien
  sustituidos pasan de 462 a 460.
- **Sospechas:**
  1. `MODELER_PARAMS` del pincel modelado (`crates/rnote-compose/src/builders/penpathmodeledbuilder.rs`) trabaja en
     unidades del documento: el mismo gesto es más lento en esas unidades con zoom.
  2. La banda de renglón de `math_take_request` (±0,5 × altura).
  3. El grosor fijo del pincel, que deja los símbolos pequeños muy gordos.
- **Cómo investigarlo:**
  - en la app de prueba, el mismo símbolo con xdraw (§10.3d) al 100 % y al 800–1600 %;
  - comparar el JSON enviado al programa auxiliar y su respuesta.

**3. Reconocer lo ya escrito fuera del modo.**

- **Hoy:** solo entra en la cola lo escrito con el modo activo (`math_after_pen_event`).
- **Propuesta:**
  - un botón «Reconocer selección» (pestaña Matemáticas y menú de la selección);
  - una función `math_queue_strokes(keys)` que agrupa por renglones (bandas de y) y manda una petición por renglón,
    con `times: null`;
  - la acción `win.math-recognize-selection` en `crates/rnote-ui/src/mathhelper.rs` llama a `recognize_now` por
    renglón.

**4. La i y la j.**

| Letra | Primera opción | Sustituidas |
|---|---|---|
| i | 626 de 640 | 579 |
| j | 100 de 244 | 81 (se confunde con i, «;» y t) |

Medido con MathWriting, solo los casos en que el corte acertó. Propuestas:
- medir los fallos de corte del punto;
- regla «punto encima de un palo corto»: es j si el palo baja de la línea base (`layout.py`) e i si no;
- kNN y reentrenar.

**5. Escribir tú el símbolo cuando no está entre las candidatas.**

- **Hoy:** `show_candidates` (`mathhelper.rs`) solo ofrece 8 candidatas, «Unir» y «Separar».
- **Propuesta:** un campo «Otro…»:
  - búsqueda por nombre en español o en LaTeX (`delta`, `alfa`, `sumatorio`, `menor o igual`);
  - el carácter o el LaTeX directamente (`δ`, `\delta`);
  - atajos (`\d` → δ, `\D` → Δ, `\a` → α, `->` → →, `=>` → ⇒, `<=` → ≤, `!=` → ≠, `inf` → ∞);
  - una rejilla plegable con griego, operadores y flechas.
- **Por dentro:**
  - va por `math_correction_requests` / `correct`;
  - el kNN acepta etiquetas que el modelo no tiene;
  - el glifo, si está en `glyphs.py`;
  - los nombres y atajos, en un `nombres.py` nuevo con la orden `names`.
- Encaja con la fase 2 de MyScript.

**6. El resultado del «=».**

- **Funciona en la nube** (2026-10-30, app de prueba con xdraw): «2+3=» → 5, escrito de una vez y con el «=»
  después.
- **No sale:**
  - en lo escrito antes de activar la opción;
  - tras corregir un símbolo: `math_apply_correction` no recalcula;
  - si un símbolo está mal leído: devuelve `result: null` y no avisa.
- **Propuesta:**
  - recalcular tras corregir y al activar la opción (una orden `evaluate` con los `known` del renglón);
  - un «?» gris cuando no se puede calcular, que al tocarlo enseñe el LaTeX leído.
- **Al usuario no le sale en su equipo:** falta que ejecute la prueba de §10.3d y diga qué sale.
- La opción está apagada por defecto. Se activa con el ⚙ de la barra flotante → «Modo matemáticas» → «Mostrar el
  resultado tras el "="».

**7. Funciones** (`FUNCTIONS` en `layout.py`: sin, cos, tan, cot, sec, csc, sinh, cosh, tanh, arcsin, arccos,
arctan, ln, log, exp, lim, max, min, det).

| Escrito | Sale | ¿Bien? |
|---|---|---|
| `ln(1)=`, `log(100)=`, `sin(0)=`, `exp(0)=` | 0, 2, 0, 1 | sí |
| `ln 2=` | «log(2) ≈ 0.6931» | el valor sí, pero el nombre no: SymPy imprime log |
| `sin x=` | nada | no: `\sin_{x}`, la x tomada como subíndice |
| `log₂(8)=` | 0,9031 | **no**: ignora la base y calcula log₁₀ 8 |
| `ln` con la «l» leída como «1» | n | **no**, y sin avisar |

**Arreglos propuestos:**
- tras una función, el argumento no es subíndice;
- `log_b(x)` → `log(x, b)`;
- mostrar `ln`;
- unión tolerante («1» o «|» + `n` / `og` → `ln` / `log`, si «l» está entre las candidatas);
- añadir `sen`, `tg`, `arcsen` y `arctg`;
- tests con cada fila.

**8. Cuánto escribir para entrenar.**

- **Recomendación:**
  - unas **80 expresiones variadas**, cerca de 1 hora, de una lista que prepara la IA cuando el usuario diga sus
    asignaturas;
  - más una **página de símbolos sueltos**, unos 20 de cada símbolo difícil.
- **Por qué:** cuentan los ejemplos por símbolo (10–20), no el número de expresiones.
- **Medido:** de 0 a 15 expresiones, los símbolos bien pasan del 44 % al 56 % y las sustituciones correctas del
  68 % al 88 % (`MATEMATICAS-INFORME.md` §0.6). La curva más allá no se conoce.
- **Después:** importar (`importar_mi_letra.py`), reentrenar y medir con una parte reservada.
- **Sus `.rnote`, por el chat o por Drive, nunca por git.**

### 13.2 Plan tipo MyScript Math (documento del usuario, `PLAN-MYSCRIPT.md`)

Todo en local, sin IA en la nube. Cada fase se usa sola, se prueba antes de la siguiente y se activa desde ajustes;
lo matemático va en un módulo propio.

| Fase | Prioridad | Ya existe | Falta |
|---|---|---|---|
| 1. Tachar con zigzag para borrar (deshacer en un paso; ajustes de sensibilidad y herramienta) | máxima | nada (`rnote-compose/src/shaperecognition.rs` sirve de modelo para el gesto) | todo; el riesgo es confundirlo con w o z |
| 2. Diccionario y catálogo (nombre, LaTeX, categoría, alias; buscador; constantes editables; química y unidades raras opcionales) | alta | `CORE` (`symbols.py`), `glyphs.py`, `FUNCTIONS`, pint, `glossary.py`, catálogo de símbolos para insertar | un solo fichero de datos y el buscador (junto con §13.1, punto 5) |
| 3. Variables (`x = 3`, `m = 70 kg`; se recalcula al cambiar; resultado al lado, en color suave; alcance la página) | muy alta | árbol `layout.py`, `evaluate.py` con SymPy, glosario con valores | definir, recalcular y mostrar; guardarlo en el archivo oculto; x variable frente a ×, por contexto |
| 4. Revisión de álgebra paso a paso (equivalencia con valores aleatorios, marcar en rojo, explicar errores típicos) | — | `STEP_SEPARATORS` (`→ ⇒ ⇔ ;`) | todo; empezar por una variable |
| 5. Exportar a LaTeX («Copiar como LaTeX» sobre una selección) | — | `to_latex(tree)`, que ya devuelve `recognize` | el botón y ampliar el árbol si falta algo |
| 6. Fracciones (decimal ↔ fracción ↔ mixta) y gráficos de y = f(x) | — | gráficas ya planeadas en `PLAN-PENDIENTE.md` §2 | todo |

Aplazado por el usuario: resolver desde una foto o un PDF.

### 13.3 Lo que ya estaba pendiente

1. **Pruebas del usuario** con el lápiz real y GTK 4.22. Arreglar lo que salga.
2. **Matemáticas con su letra:**
   - páginas nuevas con tiempos, para ajustar `pause_weight`;
   - `mis_simbolos.rnote`;
   - una pantalla para ver y borrar sus ejemplos (`examples`, `forget`, `relabel` ya existen);
   - el modo matemáticas mientras se mantiene un botón del lápiz;
   - **que el modo matemáticas ignore la tinta a formas**;
   - **búsqueda en haz** (`beam.py`).
3. **Nombre interno OnePepe:** id, migración de ajustes, instalable junto a Rnote.
4. **Diseño:** color del cuaderno en el árbol, curvas de animación, ajustes en una ventana aparte.
5. **Ideas:**
   - gráficas de funciones;
   - etiquetas y búsqueda;
   - índice de la página;
   - menú radial;
   - minimapa;
   - modo repaso;
   - plantillas propias;
   - tabla de texto de verdad;
   - tinta a texto.

---

## 14. Errores ya resueltos y trampas conocidas

**Shell y nube:**
- `pkill -f train_classifier` **mata la propia shell**, porque el patrón aparece en su línea de órdenes. Usa
  `pkill -f 'train_classifie[r].py'`; para la app, `pkill -x rnote`.
- En la nube no hay `rsync`: usa `cp -a`.
- Si un reinicio del contenedor mata Xvfb, relánzalo.

**GTK y libadwaita:**
- `<property name="limit-events">` en `.ui` no existe en GTK 4.14: la plantilla no carga.
- `var(--x)` y `:root` en GTK 4.14 dan errores de parser; por eso el bloque solo se escribe con GTK ≥ 4.16.
- **Los hijos de plantilla sin `TemplateChild` no tienen id en tiempo de ejecución** (`buildable_id()` devuelve
  None). Para moverlos, `settingslayout.rs` los busca por **título traducido**.
- `AdwViewStack` es homogéneo y la página de ajustes ensanchaba el panel plegado: `set_hhomogeneous(false)`.
- `StackSwitcher` hace los botones de ancho homogéneo: por eso las pestañas de ajustes son ToggleButton propios.
- **Un `ToggleButton` con acción** no cambia la acción con `set_active`: hay que llamar a `emit_clicked()`.
- Una regla de libadwaita (`toolbarview > .top-bar headerbar`) es más específica que la de OnePepe: el CSS de la
  cabecera incluye ese selector.
- `ShortcutLabel` está obsoleto desde 4.18 (y aquí se compila con `v4_22`): usa `accelerator_parse` +
  `accelerator_get_label` en un `Label`.
- **Imágenes gigantes:** cairo no crea superficies mayores de 32767 px (o falla por memoria) y luego da
  `SurfaceFinished`. Solución: recortar a la vista y limitar a 4096 px.
- `set_halign` de la barra flotante dentro de un arrastre no se aplica hasta el siguiente ciclo de diseño: se hace en
  `idle_add_local_once`.

**Rust y Rnote:**
- `replace_attr_for_range` de Rnote **descarta los demás atributos del rango**: los estilos meten
  `RangedTextAttribute` directamente.
- `rustfmt` puede cambiar el texto que una edición busca: después de formatear, vuelve a leer antes de editar. Así se
  perdió una vez la llamada a `favorites::refresh`.
- **La semilla del pincel** (`seed` en `textured_options`) cambia en cada trazo: no la uses para detectar cambios de
  ajustes.
- El color de un favorito pasa por el selector de color de GTK (f32): compara con tolerancia.
- glam `Vector2`: no tiene `.coords`; usa `max_element()` y `/` por componentes.

**Botones de la cinta y foco:**
- Los botones quitaban el foco al texto: `set_focus_on_click(false)` y `give_focus_back`.
- Una ventanita anclada dentro del botón que la abre recibe mal los clics: se ancla al contenedor.

**Traducción:** las entradas obsoletas `#~` de `es.po` pueden duplicar un `msgid` nuevo y hacer fallar `msgfmt`;
bórralas.

**Matemáticas:**
- El extractor partía fracciones y exponentes en renglones aparte: lo arregla `merge_close_lines`.
- Un motor temporal que se suelta pronto daba «Failed to send engine task»: mantén vivo el receptor
  (`take_engine_tasks_rx`).
- Cancelar el reentrenamiento dejaba un proceso huérfano: SIGTERM + manejador + `PR_SET_PDEATHSIG`.

---

## 15. Glosario español ↔ código

| Español (usuario / documentos) | Código |
|---|---|
| cinta | `ribbon.rs`, clases CSS `onepepe-ribbon*` |
| boli / bolígrafo favorito | `PenPreset`, `PenPresets`, `favorites.rs` |
| estilos (Título 1, Título 2, Normal) | `QuickStyle`, `TextStyles`, `textstyles.rs` |
| barra de opciones / barra flotante | `optionsbar.rs` (es la `penssidebar` de Rnote en `sidebar_box`) |
| barra de estado | `statusbar.rs` |
| cuaderno | área de trabajo (*workspace*), `RnWorkspacesBar` |
| sección | carpeta |
| página | archivo `.rnote` (un `Document`) |
| subpágina | `FolderIndex.parents` |
| árbol | `workspacebrowser/tree.rs` |
| archivo oculto | *sidecar*, `math_sidecar_path`, `.<página>.rnote.onepepe.json` |
| índice oculto | `.onepepe-indice.json`, `FolderIndex` |
| guías | `Format::show_borders` (bordes de página) |
| cruz del origen | `show_origin_indicator` |
| lienzo oscuro | `theme::canvas_dark`, `win.dark-canvas` |
| renglones / fondo | `Background`, `PatternStyle`, `LinesSettings` |
| línea de margen | `MarginLine` |
| regla infinita | `Ruler::infinite` |
| mano | `ToolStyle::OffsetCamera` |
| zoom vectorial | `Camera::set_vector_zoom` |
| tinta a formas | `shaperecognition.rs`, `brush_config.shape_recognition` |
| modo matemáticas | `MathState.mode`, `win.math-mode` |
| programa auxiliar / de matemáticas | `tools/onepepe-math`, `mathhelper.rs` |
| símbolo limpio | `clean`, glifos Hershey |
| trozo | clase `__trozo__`, `kind: "junk"` |
| corte (en símbolos) | `segment.py` |
| corte forzado | `forced.py` |
| reentrenar | `reentrenar.py`, `win.math-retrain` |
| tu letra / ejemplos personales | `correcciones.jsonl`, `PersonalModel` |
| copias de seguridad | `backup.rs`, `copias/` |
| app de prueba (nube) | árbol `smoke` con GTK 4.14 en Xvfb |

---

## 16. Cómo empezar una sesión nueva (lista de comprobación)

1. **Prepara el repositorio:**
   ```bash
   git clone https://github.com/merinodpepe/OnePepe.git && cd OnePepe
   git checkout onepepe && git pull
   git log --oneline -15
   git rev-parse origin/rnote-original   # debe seguir siendo 6bbc9b2… (el Rnote original, no se toca)
   git rev-parse origin/main origin/onepepe   # deben ser iguales
   ```
2. **Lee, en este orden:** este documento → `ESTADO.md` → `PLAN-PENDIENTE.md` → lo específico (`DISENO.md`,
   `MATEMATICAS-INFORME.md`, `DECISIONES.md`).
3. **Comprueba que todo pasa:** `cargo test -p rnote-compose -p rnote-engine` y los tests de Python (sección 10.2).
4. **Para cada petición:**
   - si es grande, un plan breve en español;
   - una rama `feat/<nombre>`;
   - cambios pequeños con tests;
   - `cargo check` de la interfaz (sección 10.3a);
   - si toca la interfaz, verlo en la app de prueba;
   - actualizar `es.po`/`POTFILES` y los documentos (`ESTADO`, `CAMBIOS`, `DECISIONES`, `PLAN`);
   - `git merge --no-ff` en `onepepe`, `git push origin onepepe` y `git push origin onepepe:main` (avance rápido).
   - Si el usuario dice «apúntalo» o «no lo hagas», solo se documenta en `PLAN-PENDIENTE.md`.
5. **Antes de subir:** `git ls-files | grep -E "npz|jsonl|mis_|dataset"` debe salir vacío. Nada del usuario ni del
   modelo en git.
6. **Al terminar**, dile al usuario en español, en pocas líneas:
   - qué se hizo;
   - cómo está probado (compilado / tests / nube / lápiz);
   - qué tiene que hacer él, con los comandos exactos (normalmente `git checkout main && git pull` y
     `~/instalar-onepepe.sh`).
