# Estado del proyecto OnePepe

Actualizado: 2026-10-30. **Versión 1.3** (set de tus asignaturas y símbolos de estadística; antes, 1.2: modo entreno; qué trae cada versión: [`VERSIONES.md`](VERSIONES.md)). Antes, en el mismo día: arreglos del modo matemáticas y plan tipo MyScript; antes, el diseño nuevo según tu guía, con capturas en [`DISENO.md`](DISENO.md)). Rama de integración: `onepepe`; **`main` = OnePepe** desde el 2026-10-29 y el Rnote original está en `rnote-original`.
Lo que falta por hacer, en orden de prioridad: [`PLAN-PENDIENTE.md`](PLAN-PENDIENTE.md) (plan de mejora).

Leyenda de cómo se probó cada cosa:
- **compilado**: el motor compila de verdad; la interfaz GTK solo con `cargo check` (comprobación de tipos, sin
  enlazar ni ejecutar), porque la nube no tiene GTK 4.22 / libadwaita 1.9 ni pantalla.
- **tests**: tests automáticos que pasan en la nube (Rust y Python).
- **ejecutada (nube)**: la app se abrió y se usó con clics simulados en la nube, en una copia de prueba con GTK 4.14
  (tu equipo usará GTK 4.22). En la tercera tanda la app arrancó con **todos** los cambios de OnePepe, incluidos los
  anteriores.
- **lápiz real**: probado por ti en tu equipo. **Todavía nada está probado con lápiz real.**

## Hecho

| Fase | Qué | Rama | compilado | tests · ejecutada (nube) | lápiz real |
|---|---|---|---|---|---|
| 0 | Auditoría (`AUDITORIA.md`), `setup-deps.sh`, compilación en la nube | `onepepe` | sí (motor); UI solo `cargo check` | — | — |
| 0 | Hojas infinitas: ya existían; test de desplazamiento y zoom | `feat/hojas-infinitas` | sí | sí (4) | pendiente |
| 1 | Paleta de 48 colores | `feat/colores` | sí | sí (3) | pendiente |
| 1 | Reordenar capas; imágenes y PDF por encima de la tinta | `feat/capas` | sí | sí (5) | pendiente |
| 1 | Regla (mover, girar, enderezar, ángulo) | `feat/regla` | sí | sí (10) + dibujo renderizado | pendiente |
| 2 | Tinta a formas (interruptor, sensibilidad, deshacer) | `feat/formas` | sí | sí (9) | pendiente |
| 1b | Botones del lápiz: borrador de trazo entero y lazo mientras pulsas, reasignables; tamaños del borrador | `feat/botones-lapiz` | sí | sí (4) | pendiente |
| 3 | Informe, extractor de `.rnote` y lista de 30 expresiones | `feat/matematicas-prototipo` | — | sí (3, Python) | no aplica |
| 3 | Modo matemáticas: reconocedor, símbolos limpios, correcciones, glosario, resultado del "=" | `feat/matematicas-limpias` | sí | sí (Rust 6 + Python 15) y prueba de punta a punta del programa auxiliar con el modelo | pendiente |
| 3 | Archivo oculto junto a cada página para corregir símbolos tras reabrir | `feat/matematicas-archivo-oculto` | sí | sí (4) | pendiente |
| — | Permiso de Flatpak para lanzar el programa de matemáticas (`--talk-name=org.freedesktop.Flatpak`) | `feat/flatpak-matematicas` | — | — | pendiente |
| — | README propio de OnePepe con logo | `feat/readme` | — | — | no aplica |
| 1c | Subíndice / superíndice del texto | `feat/subindices` | sí | sí (2) | pendiente |
| 1c | Catálogo de símbolos | `feat/catalogo-simbolos` | sí | — (solo interfaz) | pendiente |
| 1c | Fecha y hora | `feat/fecha-hora` | sí | sí (2) | pendiente |
| 1c | Viñetas, listas numeradas y casillas | `feat/listas` | sí | sí (4) | pendiente |
| 4 | Cuadernos: nueva página y mover a… (lo demás ya existía) | `feat/cuadernos` | sí | sí (1) | pendiente |
| 3ª | **Cinta tipo OneNote** (Archivo, Inicio, Insertar, Dibujar, Vista; Ctrl+F1 la oculta) | `feat/cinta` | sí | ejecutada ✔ (todos los botones, sin cierres) | pendiente |
| 3ª | **Diseño cercano a OneNote** (morado #7719AA; la barra lateral solo muestra cuadernos) | `feat/diseno-onenote` | sí | ejecutada ✔ | pendiente |
| 3ª | **Bolígrafos favoritos** (usar, guardar el actual, quitar) | `feat/boligrafos-favoritos` | sí | sí (3) + ejecutada ✔ | pendiente |
| 3ª | Insertar tabla, plantillas (Cornell, clase, problemas), título y fecha | `feat/insertar` | sí | sí (4) + ejecutada ✔ | pendiente |
| 3ª | Recorte de pantalla (portal del escritorio) | `feat/cinta` | sí | ejecutada hasta el aviso de cancelado (sin portal en la nube) | pendiente |
| 3ª | Tamaño de hoja A5–A0 y Carta; exportar PDF ajustado a una hoja o al ancho | `feat/insertar` | sí | sí + ejecutada ✔ | pendiente |
| 3ª | Nota rápida y exportar una sección entera a un PDF | `feat/notas-rapidas-y-seccion-pdf` | sí | sí (1) + ejecutada ✔ | pendiente |
| 3ª | Estilos rápidos (Título 1/2, cita, código) y listas que siguen con Intro | `feat/estilos-listas` | sí | sí (5) + ejecutada ✔ | pendiente |
| 3ª | Copias de seguridad automáticas con versiones | `feat/copias-seguridad` | sí | sí (2) + ejecutada ✔ | pendiente |
| 3ª | Interfaz en español (138 textos nuevos) | `feat/traduccion-es` | — | `msgfmt -c` ✔ + ejecutada en español ✔ | no aplica |
| Diseño | Tokens de color, tema claro/oscuro/alto contraste, lienzo oscuro solo en pantalla | `feat/diseno-tokens` | sí | sí (contraste y colores sueltos) + ejecutada en los 3 temas ✔ | pendiente |
| Diseño | Cabecera con logo, cinta según la guía, barra de estado, guías y cruz del origen discretas, selección en color de marca | `feat/diseno-cinta` | sí | sí + ejecutada ✔ | pendiente |
| Diseño | Barra de opciones: herramientas elegidas, plegar, arrastrar, anclar a izquierda o derecha | `feat/diseno-barra` | sí | ejecutada ✔ (arrastre arriba/abajo y de lado ✔) | pendiente |
| Diseño | Bolis favoritos en 5 huecos (Ctrl+1…5) y estilos de texto editables (Ctrl+Alt+1/2/0) | `feat/diseno-favoritos` | sí | sí (6) + ejecutada ✔ (arrastrar para reordenar sin probar) | pendiente |
| Diseño | Árbol del cuaderno con subpáginas, búsqueda, «+ Página», panel plegable | `feat/diseno-arbol` | sí | sí (4) + ejecutada ✔ (arrastrar sin probar: no funciona el arrastre simulado en la nube) | pendiente |
| Diseño | Regla infinita y «Fondo y renglones» (grosor, margen, valores rápidos, toda la sección) | `feat/diseno-regla-renglones` | sí | sí (8) + ejecutada ✔ | pendiente |
| Diseño | Ajustes en pestañas con logo y barra Revertir/Aplicar | `feat/diseno-ajustes` | sí | ejecutada ✔ (Revertir probado con interruptores y tema) | pendiente |
| — | Regla: ángulo libre (también infinita) y ángulo exacto | `feat/regla-angulo` | sí | sí (2) + ejecutada ✔ (girada a mano a 152°, puesta a 30°) | pendiente |
| — | Zoom del 5 % al 1600 %, «Zoom vectorial» hasta 100 000 %, herramienta Mano (Alt+7) | `feat/zoom-amplio` | sí | sí (1) + ejecutada ✔ (5 % y 91 710 % sin errores; la mano mueve la vista) | pendiente |
| Mat. | **Escribir junto a un símbolo y donde has borrado** (fallo 1): el trazo que empieza sobre un símbolo se dibuja entero si te mueves; el toque cuenta sobre la tinta; lo borrado no cuenta | `feat/matematicas-arreglos` | sí | sí (3) | pendiente |
| Mat. | **Zoom** (fallo 2): el final del trazo ya no se recorta con zoom ≥ 5000 %; lo escrito pequeño con mucho zoom no se mezcla con la letra grande de al lado (era «2+3=3») | `feat/matematicas-arreglos` | sí | sí (2) + ejecutada: al 1600 % «2+3=» bien 3 de 4 veces (al 100 %, 2 de 2); ver PLAN-PENDIENTE §2, punto 2 | pendiente |
| Mat. | **Reconocer selección** (fallo 3): lo escrito fuera del modo, un renglón por petición | `feat/matematicas-arreglos` | sí | sí (1) + ejecutada ✔ | pendiente |
| Mat. | **i y j** (fallo 4): se une el punto cortado aparte. Medido: i bien 141 → 147 de 358, lo demás igual | `feat/matematicas-arreglos` | — | sí (2) + medido con MathWriting | pendiente |
| Mat. | **«Otro…»** (fallo 5): buscar el símbolo por nombre, carácter, LaTeX o atajo; rejilla con todos | `feat/matematicas-arreglos` | sí | sí (4) + ejecutada ✔ («menor o», «x», «y») | pendiente |
| Mat. | **Resultado del «=»** (fallo 6): se recalcula al corregir, borrar, deshacer y al activar la opción; «?» que explica lo leído | `feat/matematicas-arreglos` | sí | sí + ejecutada ✔ | pendiente |
| Mat. | **Funciones** (fallo 7): `sin x`, `ln 2` (ahora «ln»), `log₂ 8 = 3`, «1n» → ln, sen/tg/arcsen/arctg, ∛ | `feat/matematicas-arreglos` | — | sí (9, Python) | pendiente |
| MyScript 1 | **Tachar con zigzag** (solo en modo matemáticas por defecto; un deshacer) y la tinta a formas espera en modo matemáticas | `feat/matematicas-arreglos` | sí | sí (3 + 3 + 1) + ejecutada ✔ | pendiente |
| MyScript 2 | **Diccionario de símbolos** (`nombres.py`, orden `names`) | `feat/matematicas-arreglos` | — | sí (4) | no aplica |
| MyScript 3 | **Variables** (`x = 3`, `m = 70 kg`; lo que las usa muestra su valor en morado) | `feat/matematicas-arreglos` | sí | sí (4) + ejecutada ✔ («x+1» → «= 4») | pendiente |
| MyScript 4 | **Revisar pasos** (rojo con explicación de errores típicos) | `feat/matematicas-arreglos` | sí | sí (9) + ejecutada (sin falsos avisos) | pendiente |
| MyScript 5 | **Copiar como LaTeX** | `feat/matematicas-arreglos` | sí | sí + ejecutada ✔ (aviso «LaTeX copiado») | pendiente |
| MyScript 6 | **Fracciones** (tocar: fracción → decimal → mixta) y **gráficas** de y = f(x) | `feat/matematicas-arreglos` | sí | sí (2 + 6) + gráfica ejecutada (en la nube los trazos salen como cajas rosas) | pendiente |
| 1.1 | **Versión visible** (barra de estado y «Acerca de», con el commit) y `VERSIONES.md` | `feat/version` | sí | ejecutada ✔ | no aplica |
| 1.2 | **Modo entreno**: ventana con la expresión y un recuadro; archivo `*.entreno.json`; set de 200; «Usar para entrenar» | `feat/modo-entreno` | sí | sí (7, Python) + medido (200 de 200 encajan; 93,7 % de símbolos exactos con letra ajena) + ejecutada ✔ (crear, escribir, siguiente, saltar, anterior, importar, cálculo de 3 renglones) | pendiente |
| 1.3 | **Set «Tus asignaturas»** (genética de poblaciones, biofísica, aprendizaje estadístico; 200) y **elegir el set** | `feat/entreno-asignaturas` | sí | sí (se dibujan las 200, todo en el vocabulario) + medido (199 de 200 encajan; 88 % de símbolos exactos con letra ajena) + ejecutada ✔ (elegir set, Nernst dibujado) | pendiente |
| 1.3 | **Símbolos de estadística** (ŷ, x̄, x̃, 𝔼, ℙ, ℝ, ⊤, ⟨ ⟩, ∩, ∪, ⇌) y acentos en la estructura | `feat/entreno-asignaturas` | — | sí (4) + medido (el resto igual: 1725 de 2641) | pendiente |
| 1.3 | **Glosario de biofísica** (k_B, R, F, N_A, h, c, q_e) | `feat/entreno-asignaturas` | — | comprobado: RT/F a 310 K = 26,71 mV | pendiente |
| 1.3.1 | **Programa de matemáticas por su ruta en Flatpak** y aviso si no responde («Nuevo entreno» no hacía nada en tu equipo) | `fix/programa-matematicas` | sí | ejecutada ✔ (con un programa que no responde sale el aviso; con el bueno, el selector de sets) | pendiente |
| 1.3.2 | **Informe de fallos** (`fallos.log` si la app se cierra por un error; aviso al abrir) | `fix/informe-fallos` | sí | funcionó en tu equipo: guardó el informe del cierre del «?» | sí |
| 1.3.3 | **Cierre al tocar el «?»**: arreglado (causa vista en tu informe: el motor seguía en uso al abrir la ventanita) | `fix/cierre-interrogacion` | sí | ejecutada ✔ (las dos ventanitas abren; el cierre no se reproduce en la nube porque allí no cambia el foco) | pendiente |
| 1.3.4 | **Sets del entreno en el repositorio** (faltaban por el `.gitignore`: «file not found») | `fix/sets-entreno-en-git` | — | sí, en una copia limpia del repositorio (83 tests de Python) | pendiente |
| 1.3.5 | **Expresión del entreno en texto legible** si no hay matplotlib, y aviso de qué instalar | `fix/entreno-expresion-visible` | sí | sí (2) + ejecutada ✔ con y sin matplotlib | pendiente |

Ya existían en Rnote y lo comprobé en el código (sin cambios): resaltador que no tapa la tinta, ejes 2D/3D, negrita,
cursiva, subrayado, tachado y color de texto, reasignación de botones del lápiz, extremo borrador, borrar con
confirmación. Rutas en `AUDITORIA.md`.

Todas las ramas están fusionadas en `onepepe` y subidas a GitHub. Cómo probar cada cosa con el lápiz:
[`CAMBIOS-ONEPEPE.md`](CAMBIOS-ONEPEPE.md). Decisiones tomadas: [`DECISIONES.md`](DECISIONES.md). Matemáticas en
detalle (diseño y medidas): [`MATEMATICAS-INFORME.md`](MATEMATICAS-INFORME.md).

## Pendiente

Resumen (detalle en [`PLAN-PENDIENTE.md`](PLAN-PENDIENTE.md)):

1. **Tus pruebas con lápiz real** de todo lo anterior (primero: hojas infinitas, tinta a formas, matemáticas, botones).
2. **Matemáticas con tu letra:** medir y ajustar con tus 30 expresiones; ahora mismo, con el modelo genérico, la
   sustitución acierta ~84 % de lo que sustituye y una fórmula entera sale perfecta solo ~10 % de las veces (medido con
   letra de otras personas). Tus correcciones mejoran el resultado al momento. Falta también el modo matemáticas
   "solo mientras mantengo pulsado un botón del lápiz".
3. **Que la aplicación se llame OnePepe por dentro** (título, "Acerca de", icono e identificador), con migración de
   ajustes y posibilidad de instalarla junto a Rnote.
4. Opcionales: árbol desplegable de tres niveles para los cuadernos, tinta a texto (Fase 5).
5. Ideas para más adelante (sección 6 de `PLAN-PENDIENTE.md`, sin implementar):
   - Reentrenar el modelo en tu equipo cuando tú quieras, con tiempo estimado.
   - Gráficas de funciones a partir de lo que escribes en modo matemáticas.
   - Etiquetas y búsqueda.
   - Menú radial bajo el lápiz y minimapa.
   - Modo repaso, entre otras.

Ya hecho en esta tanda: el permiso de Flatpak para el programa de matemáticas está **en el manifiesto** a petición tuya
(`feat/flatpak-matematicas`), y el README es de OnePepe (`feat/readme`).

## Qué necesito de ti

**De la 1.3:**

- Para Biofísica, copia el glosario a la carpeta de esa asignatura (cambia la ruta por la tuya):
  `cp tools/onepepe-math/glosarios/biofisica.txt "/ruta/a/Biofisica/.onepepe-glosario.txt"`
- Al crear un entreno, elige **Tus asignaturas**.

**Del modo entreno (1.2):**

- Una vez, para que se vean dibujadas las expresiones: `cd tools/onepepe-math && ./instalar.sh` (instala matplotlib;
  tu modelo no se toca).
- Escribe las 200 en varias sesiones (Matemáticas → Nuevo entreno; para seguir, Abrir entreno), pulsa **Usar para
  entrenar** y luego **Reentrenar**. Si quieres que yo mida tu letra, mándame el `.entreno.json` por el chat o Drive.
- Cuando me digas tus asignaturas preparo otro set solo con lo tuyo.

**De los arreglos de matemáticas y el plan tipo MyScript (30/10/2026):** no me hace falta ningún permiso nuevo. Cuando
lo instales:

- Prueba con el lápiz lo de `CAMBIOS-ONEPEPE.md` → «Matemáticas: arreglos y plan tipo MyScript (30/10/2026)».
- El programa de matemáticas se actualiza solo con `git pull` (la orden `onepepe-math` es un enlace a la carpeta
  del repositorio y no hay librerías nuevas). No hace falta reentrenar: tu modelo y tus correcciones se quedan.
- Dime si el zigzag para tachar te borra algo que no querías o no te borra cuando sí (ajustes: pestaña Matemáticas →
  Tachar).
- Sigue pendiente tu lista de asignaturas para preparar las ~80 expresiones de entrenamiento.

**De `main` = OnePepe (29/10/2026):**

- Si la portada de GitHub no muestra OnePepe: **Settings → General → Default branch** → `main`.
- El Rnote original sigue en la rama `rnote-original`. Para actualizar tu copia:
  `git fetch origin && git checkout main && git pull`.

**Del diseño nuevo (29/10/2026), cuando lo instales:**

- Mira si te gustan los colores en claro y en oscuro (`DISENO.md` tiene las capturas).
- Prueba a arrastrar páginas en el árbol y bolis en la cinta: en la nube no se pudo probar el arrastre.
- Los atajos cambiaron: las herramientas pasan a **Alt+1…Alt+6**, y Ctrl+1…Ctrl+5 son tus bolis favoritos. Si
  prefieres otra cosa, dímelo.
- Si tenías más de 5 bolis favoritos, la lista entera sigue en `~/.config/onepepe/boligrafos.anterior.json`.

1. **Cómo compilas:** `./setup-deps.sh --check`. Si dice "DEMASIADO VIEJA", usa Flatpak.
2. **Probar con el lápiz** siguiendo `CAMBIOS-ONEPEPE.md` y contarme qué falla (y cuánto acierta la tinta a formas).
3. **Instalar el programa de matemáticas:** `cd tools/onepepe-math && ./instalar.sh --entrenar` (descarga 2,9 GB y
   entrena unos minutos).
4. **Datos de tu letra:** `mis_expresiones_1.rnote` con las 30 expresiones de `tools/onepepe-math/expresiones.txt`, y
   `mis_simbolos.rnote` (instrucciones en `tools/onepepe-math/README.md`). Adjuntos en el chat o en tu Google Drive;
   **no en el repositorio**.
5. **Respuestas cortas** (`MATEMATICAS-INFORME.md`, sección 5): signo de multiplicar, coma o punto decimal, unidades en
   el mismo renglón, asignaturas y símbolos para el glosario.
6. **Botones del lápiz:** modelo de tu Wacom y qué muestra `evtest` al pulsar cada botón (ver `CAMBIOS-ONEPEPE.md`).

## Comandos para retomar el trabajo

En una sesión nueva (en la nube o en tu equipo):

```bash
git clone https://github.com/merinodpepe/OnePepe.git
cd OnePepe
git checkout onepepe
git submodule update --init --recursive
```

Tests del motor (no necesitan GTK):

```bash
cargo test -p rnote-compose -p rnote-engine
```

Tests del programa de matemáticas (con su entorno de Python; el modelo hace falta solo para un test, que se salta si no
está):

```bash
cd tools/onepepe-math
python3 -m venv /tmp/venv-onepepe && /tmp/venv-onepepe/bin/pip install -r requirements-train.txt
/tmp/venv-onepepe/bin/python -m unittest discover -s tests
# entrenar el modelo (descarga MathWriting antes con ./descargar_mathwriting.sh):
/tmp/venv-onepepe/bin/python train_classifier.py ~/.local/share/onepepe/mathwriting/mathwriting-2024
```

Ejecutar la interfaz en la nube (como en la tercera tanda): en una copia aparte (`git worktree`), cambiar en
`Cargo.toml` las características `v4_22` → `v4_14` y `v1_9` → `v1_5`, instalar `libgtk-4-dev libadwaita-1-dev xvfb
xdotool dbus-x11`, quitar de `crates/rnote-ui/data/ui` las líneas `<property name="limit-events">` y arrancar con
`Xvfb :99`, `GSK_RENDERER=cairo GDK_BACKEND=x11` y un esquema de ajustes compilado (`glib-compile-schemas`). Esa copia
**no se sube**.

Comprobar la interfaz sin GTK 4.22 (como se hizo en la nube; necesita las dependencias del motor y `libasound2-dev`):

```bash
# 1) generar un config.rs local (meson lo crea normalmente; no se sube al repositorio)
sed -e 's/@APP_NAME@/"rnote"/;s/@APP_NAME_CAPITALIZED@/"Rnote"/;s/@APP_ID@/"com.github.flxzt.rnote"/' \
    -e 's/@APP_IDPATH@/"\/com\/github\/flxzt\/rnote\/"/;s/@APP_VERSION@/"1.1.0"/;s/@APP_VERSION_SUFFIX@/""/' \
    -e 's/@APP_AUTHOR_NAME@/"x"/;s/@APP_AUTHORS@/"x"/;s/@APP_WEBSITE@/"x"/;s/@APP_REPO_URL@/"x"/' \
    -e 's/@APP_ISSUES_URL@/"x"/;s/@APP_SUPPORT_URL@/"x"/;s/@APP_DONATE_URL@/"x"/;s/@PROFILE@/"devel"/' \
    -e 's/@GETTEXT_PACKAGE@/"rnote"/;s/@LOCALEDIR@/"\/usr\/share\/locale"/;s/@DATADIR@/"\/usr\/share"/;s/@LIBDIR@/"\/usr\/lib"/' \
    crates/rnote-ui/src/config.rs.in > crates/rnote-ui/src/config.rs
echo "crates/rnote-ui/src/config.rs" >> .git/info/exclude
# 2) cargo check sin pkg-config para GTK/libadwaita/graphene
for n in GTK4 LIBADWAITA_1 GRAPHENE_GOBJECT_1_0; do
  export SYSTEM_DEPS_${n}_NO_PKG_CONFIG=1 SYSTEM_DEPS_${n}_LIB=x SYSTEM_DEPS_${n}_SEARCH_NATIVE=/usr/lib
done
cargo check -p rnote
```

Compilar y ejecutar la app en tu equipo (Flatpak, recomendado):

```bash
./setup-deps.sh --flatpak
flatpak-builder --user --install --force-clean _flatpak_app build-aux/com.github.flxzt.rnote.Devel.yaml
flatpak run com.github.flxzt.rnote.Devel
```

Para que otra IA retome: que lea primero [`CONTEXTO-IA.md`](CONTEXTO-IA.md) (contexto completo) y luego `ESTADO.md`, `PLAN-PENDIENTE.md`, `MATEMATICAS-INFORME.md` y `DECISIONES.md`,
y que siga el orden de prioridad de `PLAN-PENDIENTE.md` (una rama `feat/<nombre>` por función, fusionada en `onepepe`
cuando compile y pase sus tests).
