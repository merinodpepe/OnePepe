# Estado del proyecto OnePepe

Actualizado: 2026-10-28 (tercera tanda). Rama de integración: `onepepe` (creada desde `main`, que no se ha tocado).
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
    -e 's/@APP_IDPATH@/"\/com\/github\/flxzt\/rnote\/"/;s/@APP_VERSION@/"0.15.0"/;s/@APP_VERSION_SUFFIX@/""/' \
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

Para que Claude retome: pídele que lea `ESTADO.md`, `PLAN-PENDIENTE.md`, `MATEMATICAS-INFORME.md` y `DECISIONES.md`,
y que siga el orden de prioridad de `PLAN-PENDIENTE.md` (una rama `feat/<nombre>` por función, fusionada en `onepepe`
cuando compile y pase sus tests).
