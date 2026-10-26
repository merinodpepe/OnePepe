# Auditoría de Rnote para OnePepe

Base: rama `main` del fork (Rnote 0.15.0, commit `6bbc9b2`). Fecha: 2026-10-26.
Leyenda: **existe** / **parcial** / **no existe**. Las rutas son las que he leído; lo que no he comprobado lo indico.

## Compilación en este entorno (nube, Ubuntu 24.04, sin pantalla)

| Parte | Resultado |
|---|---|
| `rnote-compose` y `rnote-engine` (motor, sin GTK) | **compila** (`cargo check` y `cargo test`) |
| `rnote` (interfaz GTK, `crates/rnote-ui`) | **solo comprobación de tipos** (`cargo check`) saltándome la verificación de versión de pkg-config. **No se puede enlazar ni ejecutar** aquí. |
| App completa con meson | **no**: `meson.build` exige glib ≥ 2.88, GTK ≥ 4.22 y libadwaita ≥ 1.9 (GNOME 50). Ubuntu 24.04 trae glib 2.80, GTK 4.14 y libadwaita 1.5. |

Consecuencia para ti: casi ninguna distribución estable trae aún GNOME 50. **Lo más fiable en tu equipo es compilar con Flatpak** (`./setup-deps.sh --flatpak`). La compilación nativa solo funcionará en distribuciones muy recientes (probablemente Fedora 44 o Arch actualizado; no lo he comprobado).

## Funciones de tu lista

| Función | Estado | Archivos relevantes | Cómo se haría el cambio |
|---|---|---|---|
| **Hojas infinitas** | **existe** (y es el modo por defecto) | `crates/rnote-engine/src/document/layout.rs` (`Layout::Infinite`, `Default`), `document/mod.rs` (`resize_doc_infinite_layout`), `camera.rs` (`surface_mins_maxs`, `set_offset`) | Nada que cambiar. He añadido un test (`crates/rnote-engine/tests/infinite_layout.rs`) que simula 60 000 unidades de desplazamiento en 6 direcciones y zoom de 0,15× a 6×: la zona visible siempre queda dentro del documento y la cámara nunca se bloquea. **Probado con tests, no con lápiz.** |
| Tinta a formas | **no existe** | Hay una herramienta de formas aparte (`crates/rnote-engine/src/pens/shaper.rs`, formas en `crates/rnote-compose/src/shapes/`), pero no reconoce trazos a mano. El trazo se termina en `crates/rnote-engine/src/pens/brush.rs` (estado `BrushState::Drawing` → `Idle`, línea ~173). | Módulo de reconocimiento sin GTK en `rnote-compose`; al terminar el trazo en `brush.rs`, si encaja, se sustituye el `BrushStroke` por un `ShapeStroke` antes de `store.record()`, de modo que deshacer recupere el trazo original. |
| PDFs e imágenes como objetos movibles | **existe** (mover, girar, redimensionar) / **parcial** en el orden de apilado | `strokes/bitmapimage.rs:88` y `strokes/vectorimage.rs:126` implementan `Transformable` (translate, rotate, scale). PDF: `engine/import.rs:225` crea una imagen por página. El selector (`pens/selector/`) las trata como cualquier trazo. | Ya se pueden seleccionar y transformar. Limitación: las imágenes van en la capa fija `Image` y las páginas PDF en la capa `Document` (`strokes/stroke.rs:217`, `import.rs:261`), siempre **debajo** de la tinta. Se resuelve junto con "reordenar capas". |
| Selección con lazo | **existe** | `pens/pensconfig/selectorconfig.rs` (`SelectorStyle::Polygon` = lazo, también rectángulo, uno a uno y por trazo que cruza) | Nada que cambiar. |
| Reordenar capas | **parcial** | `store/chrono_comp.rs`: orden por capa (`StrokeLayer`) y luego por tiempo `t`. Ojo: `store/selection_comp.rs:56` sube el trazo al frente **cada vez que se selecciona o deselecciona**. | Añadir al `StrokeStore` funciones de traer al frente / enviar al fondo / subir / bajar (cambiando `t` y, para imágenes, pasando su capa a `UserLayer(0)`, valor ya válido en el formato), dejar de reordenar al (de)seleccionar, y añadir botones/atajos en el menú de selección de la UI. Sin cambio de formato. |
| Regla | **no existe** | Solo hay restricciones de ángulo para la herramienta de formas (`crates/rnote-compose/src/constraints.rs`) y ajuste a rejilla (`crates/rnote-engine/src/snap.rs`). | Matemática de la regla (proyección del punto sobre el borde) en un módulo sin GTK con tests; estado de la regla en el motor; el lápiz proyecta los puntos cercanos al borde; dibujo de la regla y botón en la UI. Es la mejora de Fase 1 con más riesgo porque no puedo probar la interacción aquí. |
| Más colores | **parcial** | `crates/rnote-ui/src/colorpicker/mod.rs` (9 casillas, `default_color`), `data/ui/colorpicker.ui`, guardado en `appwindow/appsettings.rs:191`. | Las 9 casillas ya son **colores favoritos guardados** (se cambian con el selector personalizado y se recuerdan). Falta una paleta amplia: añadir un botón con un desplegable de ≥ 24 colores. |
| Reconocimiento y resolución de matemáticas | **no existe** | — | Fase 3: programa auxiliar externo + extractor de trazos de `.rnote`. |
| Cuadernos y secciones | **parcial** | `crates/rnote-ui/src/workspacebrowser/`: espacios de trabajo = carpetas, crear carpeta (`workspaceactions/createfolder.rs`), y por archivo abrir, renombrar, duplicar, mover a la papelera (`filerow/actions/`). | Apoyarse en él: el espacio de trabajo es el "cuaderno", las subcarpetas las "secciones" y los `.rnote` las "páginas". Falta sobre todo: crear página nueva desde la barra, mover entre secciones y la vista en tres niveles. |
| Tinta a texto | **no existe** | — | Fase 5, si sobra crédito. |

## Otras cosas comprobadas

- Botones del lápiz reconfigurables: **existe**, `crates/rnote-engine/src/pens/shortcuts.rs` (`StylusPrimaryButton`, `StylusSecondaryButton`, botones de tableta `DrawingPadButton0..3`). Útil para el "modo matemáticas mientras mantengo pulsado".
- Formato `.rnote`: JSON comprimido con gzip (`crates/rnote-engine/src/fileformats/rnoteformat/`, no lo he leído a fondo aún). Nada de lo planeado para las Fases 1–2 necesita cambiarlo.

## Qué toqué primero (Fase 0)

1. Fase 1 en este orden: colores (barato) → reordenar capas → PDFs/imágenes por encima de la tinta (sale con lo anterior) → regla.
2. Fase 2: tinta a formas.

Todo lo de la tabla de arriba se hizo después; el estado actual está en `ESTADO.md`.

## Parte B: funciones extra de OneNote (auditoría del 2026-10-26, rama `onepepe`)

Estado **antes** de tocar nada, con la ruta donde lo comprobé, y qué hice.

### Fase 1b: botones del lápiz

| Función | Estado | Dónde | Qué se hizo |
|---|---|---|---|
| Reasignar botones del lápiz a cualquier herramienta | **existe** | `crates/rnote-engine/src/pens/shortcuts.rs` (`ShortcutAction::ChangePenStyle`), ajustes en `crates/rnote-ui/src/settingspanel/penshortcutrow.rs` | nada |
| Modos "mientras pulso" y "alterna al pulsar" | **existe** | `ShortcutMode::Temporary` y `ShortcutMode::Toggle` en `shortcuts.rs`; lógica en `crates/rnote-engine/src/pens/penholder.rs` (`handle_pressed_shortcut_key`) | nada |
| Botón 1 = borrador **de trazo entero** mientras pulso | **parcial** | por defecto el botón 1 ya era "borrador mientras pulso", pero usaba el estilo de borrador de la barra | variante "Eraser (Whole Strokes)" independiente de la barra, y por defecto (`feat/botones-lapiz`) |
| Botón 2 = **lazo** mientras pulso | **parcial** | por defecto "selección mientras pulso", con el estilo de la barra (rectángulo por defecto, `selectorconfig.rs`) | variante "Selector (Lasso)", por defecto (`feat/botones-lapiz`) |
| Extremo borrador del lápiz | **existe** | `crates/rnote-ui/src/canvas/input.rs:480` (`DeviceToolType::Eraser` → `PenMode::Eraser`), herramienta elegible en ajustes ("Tool for the eraser action") | documentado; el problema del README (botones que envían "Eraser Tool") no se puede distinguir desde la app (ver `CAMBIOS-ONEPEPE.md`) |
| Borrador de trazo entero y por área en la barra | **existe** | `crates/rnote-ui/data/ui/penssidebar/eraserpage.ui` ("Trash Strokes" / "Split Strokes") | tamaños rápidos del borrador 8/24/64 (`feat/botones-lapiz`) |

### Fase 1c: texto, símbolos y utilidades

| Función | Estado | Dónde | Qué se hizo |
|---|---|---|---|
| 1. Subíndice y superíndice en el texto | **no existe** | `TextAttribute` (`crates/rnote-engine/src/strokes/textstroke.rs:85`) no tiene atributo de elevar | botones x² / x₂ con caracteres Unicode (`feat/subindices`) |
| 2. Catálogo de símbolos | **no existe** (solo el selector de emojis) | `crates/rnote-ui/data/ui/penssidebar/typewriterpage.ui` (`emojichooser`) | botón "Ω" con griego, operadores, análisis, conjuntos y física (`feat/catalogo-simbolos`) |
| 3. Resaltador semitransparente que no tapa la tinta, con paleta | **existe** | estilo "Marker" del pincel (`crates/rnote-engine/src/pens/pensconfig/brushconfig.rs`), capa `Highlighter` dibujada en modo multiplicar (`crates/rnote-engine/src/engine/rendering.rs:189`); color con el selector común (paleta de 48 y favoritos) | nada |
| 4. Ejes cartesianos 2D y 3D | **existe** | `ShapeBuilderType::CoordSystem2D`, `CoordSystem3D`, `QuadrantCoordSystem2D` (`crates/rnote-compose/src/builders/mod.rs`), en la herramienta de formas | nada |
| 5. Estampar fecha, hora o ambas | **no existe** | — | botón de calendario en la barra de texto (`feat/fecha-hora`) |
| 6. Negrita, cursiva, subrayado, tachado, color de fuente | **existe** | botones en `typewriterpage.ui`; atributos en `textstroke.rs` (`FontWeight`, `Style`, `Underline`, `Strikethrough`, `TextColor`) | nada |
| 7. Casillas, viñetas y listas numeradas | **no existe** | — | tres botones en la barra de texto, por párrafo (`feat/listas`) |
| Estilos rápidos (Título 1/2, cita, código) | **no existe** | — | no hecho (opcional, "solo si sobra") |
| Recorte de pantalla | **no existe** | — | no hecho (opcional) |
