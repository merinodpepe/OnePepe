# Estado del proyecto OnePepe

Actualizado: 2026-10-26. Rama de integración: `onepepe` (creada desde `main`, que no se ha tocado).
Plan pendiente con los últimos cambios: [`PLAN-PENDIENTE.md`](PLAN-PENDIENTE.md).

Leyenda de cómo se probó cada cosa:
- **compilado**: el motor compila de verdad; la interfaz GTK solo con `cargo check` (comprobación de tipos, sin
  enlazar ni ejecutar), porque la nube no tiene GTK 4.22 / libadwaita 1.9 ni pantalla.
- **tests**: tests automáticos que pasan en la nube.
- **lápiz real**: probado por ti en tu equipo. **Todavía nada está probado con lápiz real.**

## Fases hechas

| Fase | Qué | Rama | compilado | tests | lápiz real |
|---|---|---|---|---|---|
| 0 | Auditoría (`AUDITORIA.md`), `setup-deps.sh`, compilación en la nube | `onepepe` | sí (motor); UI solo `cargo check` | — | — |
| 0 | Hojas infinitas: ya existían; test de desplazamiento y zoom | `feat/hojas-infinitas` | sí | sí (4) | pendiente |
| 1 | Paleta de 48 colores | `feat/colores` | sí (UI con `cargo check`) | sí (3) | pendiente |
| 1 | Reordenar capas; imágenes y PDF por encima de la tinta | `feat/capas` | sí | sí (5) | pendiente |
| 1 | Regla (mover, girar, enderezar, ángulo) | `feat/regla` | sí | sí (10) + dibujo renderizado a imagen | pendiente |
| 2 | Tinta a formas (interruptor, sensibilidad, deshacer) | `feat/formas` | sí | sí (6 del reconocedor + 3 de extremo a extremo) | pendiente |
| 3 | Primer entregable: `MATEMATICAS-INFORME.md` + extractor de trazos | `feat/matematicas-prototipo` | — | sí (3 del extractor, Python) | no aplica |
| 4 | Cuadernos: nueva página y mover a… (lo demás ya existía) | `feat/cuadernos` | sí (UI con `cargo check`) | sí (1: página vacía se reabre) | pendiente |

Todas las ramas están fusionadas en `onepepe` y subidas a GitHub. Detalle de cada función y cómo probarla con el
lápiz: [`CAMBIOS-ONEPEPE.md`](CAMBIOS-ONEPEPE.md). Decisiones tomadas: [`DECISIONES.md`](DECISIONES.md).

## Pendiente

En el orden de prioridad de `PLAN-PENDIENTE.md`:

1. **Tus pruebas con lápiz real** de todo lo de la tabla (lo primero: hojas infinitas y tinta a formas).
2. **Matemáticas (Fase 3), con el cambio de plan:** sustituir tu letra por símbolos limpios (fuente Hershey de trazo
   único), corrección por símbolo o grupo, y el resultado del "=" como función aparte. Diseño en
   `MATEMATICAS-INFORME.md`, sección 0. **Bloqueado hasta tener tus datos y respuestas** (ver abajo).
3. **Fase 1b: botones del lápiz Wacom** (borrador de trazo entero y lazo mientras se pulsa, reasignables, extremo
   borrador, borrador de trazo y por área en la barra, verificación con `evtest`). No empezada.
4. Resto de la Fase 1: nada pendiente salvo tus pruebas.
5. **Fase 1c:** subíndice/superíndice en texto, catálogo de símbolos, resaltador con paleta, ejes 2D/3D, fecha y hora,
   formato de texto, listas y casillas. No empezada. Antes hay que auditar en el código qué existe (parte B) y
   apuntarlo en `AUDITORIA.md`.
6. Cuadernos: posible árbol desplegable de tres niveles si lo quieres (ahora se navega entrando en carpetas).
7. Fase 5 (tinta a texto): no empezada; última prioridad.

## Qué necesito de ti

1. **Saber cómo compilas:** ejecuta `./setup-deps.sh --check` y dime si dice "DEMASIADO VIEJA" (entonces Flatpak).
2. **Probar con el lápiz** siguiendo `CAMBIOS-ONEPEPE.md` y contarme qué falla.
3. **Datos para matemáticas** (instrucciones en `tools/onepepe-math/README.md`): `mis_expresiones_1.rnote` con las 30
   expresiones de `tools/onepepe-math/expresiones.txt`, y `mis_simbolos.rnote`. Pásamelos adjuntos en el chat o en una
   carpeta de tu Google Drive. **No los subas al repositorio.**
4. **Respuestas cortas** (`MATEMATICAS-INFORME.md`, sección 5): signo de multiplicar, coma o punto decimal, unidades en
   el mismo renglón, asignaturas y símbolos para el glosario.
5. **Decisión de formato** (`MATEMATICAS-INFORME.md`, 0.1): ¿guardo la información para corregir símbolos limpios en un
   archivo oculto junto a cada página, o las correcciones solo valen durante la sesión?
6. **Para la Fase 1b:** modelo de tu lápiz Wacom y salida de `evtest` al pulsar cada botón y al usar el extremo
   borrador (cómo obtenerla: se explicará en `CAMBIOS-ONEPEPE.md` al hacer esa fase; adelanto abajo).

Adelanto para la salida de `evtest` (en tu equipo, con la tableta conectada):

```bash
sudo apt install evtest        # Debian/Ubuntu;  Fedora: sudo dnf install evtest;  Arch: sudo pacman -S evtest
sudo evtest                    # elige el número del dispositivo que diga "Pen" o "Stylus"
# pulsa el botón 1, el botón 2 y acerca el extremo borrador; copia las líneas "EV_KEY" que salgan (BTN_STYLUS,
# BTN_STYLUS2, BTN_TOOL_RUBBER...). Ctrl+C para salir.
```

## Comandos para retomar el trabajo

En una sesión nueva (en la nube o en tu equipo):

```bash
git clone https://github.com/merinodpepe/OnePepe.git
cd OnePepe
git checkout onepepe
git submodule update --init --recursive
```

Tests del motor y de las herramientas (no necesitan GTK):

```bash
cargo test -p rnote-compose -p rnote-engine
(cd tools/onepepe-math && python3 -m unittest discover -s tests)
```

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

Para que Claude retome: pídele que lea `ESTADO.md`, `PLAN-PENDIENTE.md`, `MATEMATICAS-INFORME.md` y
`DECISIONES.md`, y que siga el orden de prioridad de `PLAN-PENDIENTE.md` (una rama `feat/<nombre>` por función,
fusionada en `onepepe` cuando compile y pase sus tests).
