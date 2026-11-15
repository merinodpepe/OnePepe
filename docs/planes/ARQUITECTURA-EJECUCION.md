# Arquitectura de ejecución del cierre (2026-11-15)

Manual para ejecutar de una vez las tareas T0–T13 de [`PLAN-CIERRE.md`](PLAN-CIERRE.md). El plan dice QUÉ; la
auditoría ([`AUDITORIA-MATEMATICAS.md`](../auditorias/AUDITORIA-MATEMATICAS.md)) dice POR QUÉ; este documento dice CÓMO. No repite
el plan: usa sus ids, ramas, metas y criterios de aceptación.

Cómo se lee:

- **Usuario:** solo §1 (bloque A, al arrancar) y §9 (bloque B, al terminar).
- **Orquestador (Opus 5.5):** §0–§5 y §8. No programa.
- **Subagente:** §3 (reglas), su tarea en §7 y las secciones de §6 que esa tarea cite. Nada más.

Cómo arrancar (sesión nueva de Claude Code con Opus 5.5, en la carpeta del repositorio):

```text
Ejecuta ARQUITECTURA-EJECUCION.md como orquestador. El bloque A ya está respondido en §1: no preguntes nada.
```

## 0. Estado de partida (comprobado el 2026-11-15)

| Qué | Valor |
|-----|-------|
| Rama y versión | En el remoto, `onepepe` = `main`, **versión 1.4.0** (el punto multiplica y la coma es el decimal). Ninguna tarea T0–T13 hecha |
| Código leído | La auditoría y este manual se escribieron leyendo la 1.3.5 que había en el equipo. La 1.4 solo cambia, en matemáticas, `evaluate._tokens`, `layout._NEVER_SCRIPT` y `pagina._mark` (resultados con coma). Revisado contra la 1.4: los números de línea citados son los de la 1.4 y las 8 sondas con resultado erróneo siguen igual |
| Rama local del equipo | Alineada con el remoto el 2026-11-15 (decisión A8). El commit `146e1526` «Punto de guardado antes de migrar», con archivos personales, está en la rama local `guardado/antes-de-migrar`, que **no se sube ni se fusiona nunca**. Sus 12 archivos siguen en la carpeta, sin seguimiento (`.git/info/exclude`), con copia en `~/.local/share/onepepe/guardado-146e1526/` |
| Documentos | `AUDITORIA-MATEMATICAS.md`, `PLAN-CIERRE.md`, este manual y las notas de `ESTADO.md`, `DECISIONES.md` y `CLAUDE.md`: con commit en `feat/auditoria-plan-cierre`, fusionados en `onepepe` y subidos el 2026-11-15 |
| Tests | Python: 88 pasan en la 1.4 (15 archivos en `tools/onepepe-math/tests/`) |
| Entorno Python | `~/.local/share/onepepe/venv`: numpy 2.5.4, scikit-learn 1.9.1, sympy 1.14.0, scipy 1.18.1. **Sin PyTorch** |
| Modelo | `~/.local/share/onepepe/math/simbolos.npz`: 227 clases, versión `20261111-122039-2191`. Sin `corte.json` (valen los valores por defecto del código) |
| Almacén | `correcciones.jsonl`: 3145 ejemplos (1293 símbolos, 1852 trozos). 2962 vienen de `entreno:*`; 183, de correcciones a mano |
| Sesiones | `~/Documentos/OnePepe entrenos/`: general 85 de 200 escritas, asignaturas 30 de 200. Todas con tiempos por trazo. 157 renglones |
| MathWriting | `~/.local/share/onepepe/mathwriting/mathwriting-2024`: `train` 229 864, `valid` 15 674, `test` 7 644, `symbols` 6 423. Sin CROHME |
| Reparto con la regla de §6 M0.4 | Examen: 22 expresiones, 31 renglones, 217 símbolos, **6 rayas de fracción**. Validación: 10, 14, 99. Entrenamiento: 83, 112, 842 |

**Aviso que condiciona todo: el examen es pequeño.** Con 217 símbolos, una cifra del 90 % tiene un error típico de
unos ±2–3 puntos; con 31 renglones, una del 60 % tiene ±9; con 6 rayas, cada raya vale 17 puntos. Por eso:

- Las elecciones de parámetros se hacen con la **medida cruzada** (§6 M0.5: 941 símbolos no vistos), nunca con el examen.
- Las metas con menos de 50 casos en el examen (rayas, números de 2 o más cifras) se leen en la medida cruzada.
- Cada expresión nueva que escribas agranda el examen sola: los 400 ids ya están repartidos.

## 1. Bloque A: decisiones iniciales (única pregunta al usuario)

**Respondido por el usuario el 2026-11-15** (respuestas debajo de la tabla). El orquestador no vuelve a preguntar:
usa las respuestas y no pregunta nada más hasta §9.

| Id | Decisión | Valor por defecto | Si es «no» |
|----|----------|-------------------|------------|
| A0 | Commits locales en ramas `feat/` y `fix/`, fusiones `--no-ff` en `onepepe`, y `push` | **Sí** a los commits; `push` solo si el usuario lo permite | No arranca nada: todo el flujo depende de esto |
| A1 | Reimportar los entrenos (T1) y, al final, instalar en tus datos reales el almacén y el modelo nuevos, con copia previa | **Sí** | Todo se hace en la copia de trabajo y la instalación pasa a §9 como una orden |
| A2 | Metas de «terminado» (tabla de la «Lista priorizada» del plan) | **Las del plan** | Se usan las que indiques |
| A3 | Descargar CROHME para T7 | **No** (MathWriting tiene 230 000 fórmulas sin aprovechar) | — |
| A4 | Encender «=» y «Variables» por defecto en T13 | **Sí**, solo si las sondas dan 0 resultados erróneos. «Revisar pasos» sigue apagado | Se dejan apagadas |
| A5 | Rutas: sesiones `.entreno.json`; dataset nuevo; páginas reales de apuntes | Sesiones: `~/Documentos/OnePepe entrenos`. Dataset nuevo: **ninguno**. Páginas reales: **ninguna** | Sin páginas, T9 se salta y queda en §9 |
| A6 | Instalar PyTorch (CPU, unos 200 MB) para comparar una red convolucional en T7 | **No** | — |
| A7 | Alcance de T12 (diseño fluido) | **Lote seguro:** propuestas 1, 2, 4, 9 y 12 | «Todo» añade 3, 5, 6, 7, 8, 10 y 11 |
| A8 | Qué hacer con el commit local `146e1526` (punto de guardado con archivos personales, sin subir) | Guardarlo en la rama `guardado/antes-de-migrar` y alinear la rama local `onepepe` con el remoto; sus archivos se quedan en la carpeta, sin seguimiento de git | La rama local no se toca y el cierre trabaja en un árbol aparte (§3.4, punto 8) |

**Respuestas (2026-11-15):**

| Id | Respuesta |
|----|-----------|
| A0 | Sí. **Además se permite `push`**: lo hace el integrador al cerrar cada oleada (§3.4, punto 5) |
| A1 a A6 | El valor por defecto de la tabla |
| A7 | **El diseño (T12) queda en espera y es el último paso**: oleada O10, después del cierre T13. Alcance: lote seguro |
| A8 | **Sí. Hecho el 2026-11-15:** el commit está en `guardado/antes-de-migrar` y la rama local `onepepe` es la del remoto |

## 2. Agentes y contrato

### 2.1 Reparto

| Agente | Modelo | Hace | No hace |
|--------|--------|------|---------|
| Orquestador | Opus 5.5 | Lanza oleadas, lee resúmenes (máx. 15 líneas), decide en los fallos, valida el estado final | Programar, leer código salvo al desbloquear un fallo |
| Programador (S-A, S-B, S-C) | Sonnet | Una tarea de §7: test que falla, código, medida | Tocar documentos compartidos o versión; preguntar al usuario |
| Integrador (H-I) | Haiku | Ejecuta comprobaciones, fusiona, hace `push`, sube versión, escribe `CAMBIOS`, `ESTADO`, `VERSIONES`, `DECISIONES`, `es.po`, informe final | Cambiar lógica |
| Tests (H-T) | Haiku | Tests repetitivos a partir de una tabla de §7 (sondas, casos de borde) | Decidir valores esperados: van en la tabla |

Donde el plan pedía a Fable diseñar T7 y T9, el diseño ya está en §6 M7 y §7 T9. Opus solo revisa la fusión.

### 2.2 Lo que recibe un subagente (plantilla exacta)

```text
Eres el agente {S-A|S-B|S-C|H-I|H-T} de la tarea {Tn} del cierre de OnePepe.
Repositorio: /home/pepe/Pepe_Flock/One_Pepe. Carpeta de trabajo: {ruta}. Rama: {rama}.
1. Lee SOLO: ARQUITECTURA-EJECUCION.md §3, §7 {Tn} y las secciones de §6 que esa tarea cite.
2. Decisiones del usuario: A0={..} A1={..} A2={..} A3={..} A4={..} A5={..} A6={..} A7={..}.
3. Estado: onepepe en {hash}. Fusionadas: {lista}. Saltadas o bloqueadas: {lista}.
4. {Solo en un reintento} Corrección del orquestador: {texto}.
Devuelve EXACTAMENTE el formato de §2.3, con 15 líneas como máximo. No preguntes nada a nadie.
```

### 2.3 Lo que devuelve (formato exacto, máx. 15 líneas)

```text
TAREA: Tn   ESTADO: ok | fallo | saltada
RAMA: <rama>   COMMITS: <n> (<hash corto del último>)
TESTS: py <pasan>/<fallan>/<saltados> · cargo <ok|fallo|no toca> · msgfmt <ok|fallo|no toca>
MEDIDA: <métrica>: <antes> -> <después> (n=<n>) [una línea por métrica de la aceptación]
ACEPTACION: cumplida | no cumplida (<qué cifra falta>)
ARCHIVOS: <lista corta>
SUPUESTOS: <lista o «ninguno»>
NOTAS-DOC: <1 a 3 líneas para DECISIONES y VERSIONES>
FALLO: <test> · <error, máx. 3 líneas> · <archivos> · <qué intentó, máx. 2 líneas>   [solo si ESTADO=fallo]
```

## 3. Reglas

### 3.1 Bloque común (se pega en cada prompt de subagente)

```text
REGLAS (no se rompen):
- Responde en español. Rama feat/<nombre> o fix/<nombre> creada desde onepepe. Commits pequeños en español.
- git add SOLO con rutas explícitas (nunca -A ni .). Nunca instalacion.log. No hagas push (lo hace el integrador). Nunca --force. Nunca borrar
  ramas. Solo el repositorio merinodpepe/OnePepe.
- No cambies el formato .rnote. Nada de modelos, datasets ni letra del usuario en git: antes de cada commit,
  mira `git status --short` y `git diff --cached --stat`.
- Datos: ONEPEPE_DATA=~/.local/share/onepepe/cierre-trabajo (copia de trabajo). Los datos reales
  (~/.local/share/onepepe/math, ~/Documentos/OnePepe entrenos) son de SOLO LECTURA. Los tests unitarios usan una
  carpeta temporal propia.
- Orden de trabajo: lista TODOS los archivos que necesitas, léelos todos de una vez en paralelo, escribe primero el
  test que falla, haz todas las ediciones, ejecuta los tests. Prohibido leer archivo por archivo o releer trozos
  contiguos en bucle.
- Si un test falla tras tu arreglo: NO iteres ni releas código. Devuelve ESTADO=fallo con la línea FALLO.
- «Si no mejora, no se queda»: si la medida de aceptación no mejora, deja el código detrás de un parámetro apagado
  o retíralo, y dilo en NOTAS-DOC.
- No toques meson.build, app.metainfo.xml.in.in, VERSIONES.md, ESTADO.md, DECISIONES.md, CAMBIOS-ONEPEPE.md:
  los escribe el integrador. Archivos nuevos: cabecera SPDX GPL-3.0-or-later.
- Ante una duda: aplica el valor por defecto de este documento, apúntalo en SUPUESTOS y sigue.
- Di qué está compilado, qué tiene tests y qué se ha ejecutado. Nada está probado con lápiz.
```

### 3.2 Órdenes comunes

```bash
REPO=/home/pepe/Pepe_Flock/One_Pepe; M=$REPO/tools/onepepe-math
PY=~/.local/share/onepepe/venv/bin/python
export ONEPEPE_DATA=~/.local/share/onepepe/cierre-trabajo
export ONEPEPE_MATHWRITING=~/.local/share/onepepe/mathwriting/mathwriting-2024
ENTRENOS="$HOME/Documentos/OnePepe entrenos"
cd $M && $PY -m unittest discover -s tests            # tests de Python
cd $REPO && cargo test -p rnote-engine math            # tests de Rust del motor
cd $REPO && cargo check -p rnote                       # interfaz (preparación: CONTEXTO-IA.md §10.3a)
msgfmt -c -o /dev/null crates/rnote-ui/po/es.po        # traducción
```

### 3.3 Regla anti-bucle de lectura (ya copiada en `CLAUDE.md`)

Antes de cambiar nada: (1) lista todos los archivos necesarios; (2) léelos todos de una vez, en paralelo; (3) haz
todas las ediciones; (4) ejecuta los tests; (5) si fallan y eres un subagente, escala al orquestador (§4). Prohibido
leer archivo por archivo o releer trozos contiguos en bucle.

### 3.4 Seguridad sin usuario

1. Sin «ok» al bloque A no se ejecuta nada. Sin A1, ningún agente escribe en `~/.local/share/onepepe/math`.
2. **Copia de trabajo.** La oleada O0 crea `~/.local/share/onepepe/cierre-trabajo/math/` copiando
   `correcciones.jsonl` y `simbolos.npz`, y guarda sus `sha256sum` en `cierre-trabajo/origen.sha256`. Todo el
   cierre lee y escribe ahí. El integrador comprueba al final que los originales tienen la misma suma.
3. **Promoción** (única escritura en datos reales; solo con A1 = sí; la hace H-I en T13 con `promover_cierre.py`):
   copia cada archivo real a `<nombre>.<AAAAMMDD-HHMMSS>.bak` y después instala los de la copia de trabajo.
4. Las sesiones `.entreno.json` nunca se modifican (ni la marca `imported`).
5. Git: nunca `--force`, nunca borrar ramas, nunca `git add -A`. `push` solo el integrador, al cerrar cada oleada:
   `git push -u origin <ramas de la oleada> onepepe` y `git push origin onepepe:main` (siempre avance rápido; si el
   remoto lo rechaza, para y escala). Antes de cada `push`: `git diff --stat origin/onepepe..onepepe` no puede
   contener `*.rnote`, `*.npz`, `*.jsonl`, `*.entreno.json` ni `instalacion.log`.
6. Red: solo lo aprobado en A3 y A6. Ningún dato del usuario sale del equipo.
7. Trabajo en paralelo: cada programador en su `git worktree` (Agent con `isolation: "worktree"`). Las tareas con
   Rust van siempre en el árbol principal y nunca dos a la vez (comparten `target/`).
8. **Punto de guardado local.** El commit `146e1526` y la rama `guardado/antes-de-migrar` no se suben ni se
   fusionan nunca. Antes de cada `push`: `git merge-base --is-ancestor 146e1526 onepepe` debe fallar. Los archivos
   personales que quedan en la carpeta sin seguimiento (`mis_expresiones_1.rnote`, `instalacion.log` y otros) no
   se añaden a git.

## 4. Protocolo de fallo

1. Un test o una medida falla → el subagente para y devuelve `ESTADO: fallo` con la línea `FALLO`. No itera.
2. El orquestador responde con UNA de estas dos cosas, y relanza al mismo tipo de agente con la plantilla de §2.2:
   - **Corrección concreta:** archivo, función y cambio exacto.
   - **Reapertura de la especificación:** corrige el paso de §7 en este documento (commit `docs:`) y cita el cambio.
3. Segundo fallo de la misma tarea tras escalar → **bloqueada**: el integrador apunta la causa en `DECISIONES.md`
   como `BLOQUEADA: Tn — causa`, la rama se queda sin fusionar (no se borra) y la tarea pasa a §9.
4. Las tareas que dependen de una bloqueada se saltan (`SALTADA: Tn — depende de Tm`). Las demás siguen.
   Dependencias: T1←T0; T2←T1; T3←T0; T4←T3; T5←T0; T6←T3,T5; T7←T2; T8←T2; T9←T5; T10←T6; T11←T0.
5. «Aceptación no cumplida» con tests en verde **no es un fallo**: se fusiona si la medida no empeora (regla de
   §6 M0.6), se apunta la cifra que falta y se sigue. Si empeora, el cambio queda apagado (parámetro) y se fusiona así.
6. Conflicto al fusionar → el integrador aborta (`git merge --abort`) y devuelve fallo; el programador de esa rama
   hace `git merge onepepe` en su rama, resuelve, pasa tests y se reintenta.

## 5. Oleadas

| Oleada | Tareas (agente, dónde) | En paralelo | Archivos que toca cada rama | Versión al fusionar |
|--------|------------------------|-------------|------------------------------|---------------------|
| O0 | Preparación (H-I, árbol principal) | — | Solo documentos | — |
| O1 | T0 (S-A; sondas: H-T) | — | `alinear.py`, `medir.py`, tests nuevos | 1.5.0 |
| O2 | T1→T2 (S-A, worktree) ∥ T3→T4 (S-B, worktree) ∥ T11 (S-C, árbol principal) | 3 | A: `entreno.py`, `reimportar_entrenos.py`, `train_classifier.py`, `reentrenar.py`, `classifier.py`, `entrenar_pliegues.py`. B: `layout.py`, `evaluate.py`, `pagina.py`, `entorno.py`, `test_sondas.py`. C: `server.py`, `math.rs`, `mathhelper.rs`, `es.po` | 1.6.0 |
| O3 | T5 (S-A) | — | `segment.py`, `reparar.py`, `recognizer.py`, `layout.py` (`bar_role`) | 1.7.0 |
| O4 | T6 (S-A, árbol principal) ∥ T7a extracción (S-B, worktree) | 2 | A: `lexico.py`, `layout.py`, `evaluate.py`, `recognizer.py`. B: `extraer_simbolos.py` (nuevo) | 1.8.0 |
| O5 | T8 (S-A, árbol principal: toca Rust) | — | `recognizer.py`, `pagina.py`, `calibrar_umbral.py`, `math.rs` | 1.9.0 |
| O6 | T7b (S-A) | — | `features.py`, `classifier.py`, `segment.py`, `personal.py`, `train_classifier.py` | 1.10.0 |
| O7 | T10 (S-A) | — | `haz.py`, `recognizer.py`, `context.py`, `layout.py` | 1.11.0 |
| O8 | T9 (S-A, árbol principal) — solo si A5 da páginas | — | `reproducir_pagina.py`, `math.rs`, `mathhelper.rs`, `server.py`, `recognizer.py` | 1.12.0 |
| O9 | T13 (S-A: valores por defecto; H-I: lo demás) | — | `math.rs`, documentos, `promover_cierre.py` | 1.13.0 |
| O10 | T12 (S-A, árbol principal) — **en espera: último paso** | — | `theme.rs`, `style.css`, `ribbon.rs`, `canvas/mod.rs`, `math.rs` | 1.14.0 |

Reglas de las oleadas:

- **O0** (H-I): los documentos ya tienen commit y están subidos. Comprueba el punto 8 de §3.4 y que el árbol de
  trabajo está al día con `origin/onepepe`. Crea la copia de trabajo (§3.4). Ejecuta los tests y apunta «88/0/0».
- **Fusión** (H-I, tras cada tarea con `ESTADO: ok`): `git checkout onepepe && git merge --no-ff --no-commit <rama>`;
  ejecuta las comprobaciones de §3.2 que toquen; si pasan, `git commit`; si no, `git merge --abort` y fallo.
- **Cierre de oleada** (H-I): sube la versión una vez (`meson.build`, `app.metainfo.xml.in.in`, `VERSIONES.md`),
  vuelca las `NOTAS-DOC` y `SUPUESTOS` en `DECISIONES.md`, actualiza `ESTADO.md`, hace commit en `onepepe`. Una oleada
  saltada no consume número de versión. Después hace los `push` del punto 5 de §3.4.
- **Recalibrado:** tras fusionar T2 y de nuevo tras T7b, H-I ejecuta en este orden `elegir_corte.py` (T5, si ya
  existe) y `calibrar_umbral.py` (T8, si ya existe). Cambiar de modelo cambia la escala de las puntuaciones.
- T12 está en espera por decisión del usuario y es el último paso: va después del cierre (T13). Al fusionarla, H-I
  añade su fila y su versión a `INFORME-CIERRE.md`.

## 6. Matemática resuelta

Notación: los trazos de un renglón son `s_0 … s_{n-1}` en orden de escritura. Un símbolo es un grupo de 1 a `K = 4`
trazos seguidos (`MAX_STROKES`). `p_[a,b)(t)` es la probabilidad que da la red a la etiqueta `t` para los trazos
`a … b-1`; si varias clases son el mismo símbolo (`classifier.labels`), se suman. `log` es logaritmo natural.

### M0. Alineado ordenado, reparto y medida (T0)

**M0.1 Problema.** Dado el LaTeX esperado, con fichas `t_0 … t_{m-1}` (`entreno.expected_symbols`), hallar qué trazos
forman cada símbolo. `forced_groups` falla porque puntúa cada grupo con «la mejor ficha de cualquier posición»
(`forced.py:31`) y reparte las etiquetas después: el corte no sabe qué ficha toca en cada sitio. En «2x+1=7» puede
dar una raya al «=» y tres trazos al «7».

**M0.2 Programación dinámica.** Para un orden de escritura fijo `v` (una permutación de las fichas):

- Estado: `B[i][j]` = mejor puntuación de repartir los primeros `i` trazos entre las primeras `j` fichas de `v`.
- Inicio: `B[0][0] = 0`; el resto, `−∞`.
- Coste de un grupo: `g(a, b, t) = log max(p_[a,b)(t), 1e-9) − LEJOS·[b−a > 1 y los trazos no están encadenados
  por cercanía]`, con `LEJOS = 3.0` y «encadenados» según `segment._close`. Si `t` no es una clase del modelo,
  `g = log UNK` con `UNK = 0.05` y el símbolo se marca `desconocido`.
- Transición: `B[i][j] = max_{k=1..K, k≤i} B[i−k][j−1] + g(i−k, i, v_j)`. Se guarda el `k` ganador en `R[i][j]`.
- Solución: `B[n][m]`. Existe solo si `m ≤ n ≤ K·m`; si no, `None`.
- Reconstrucción: desde `(n, m)`, repetir `k = R[i][j]`, grupo `[i−k, i)` con ficha `v_j`, `i ← i−k`, `j ← j−1`.
- Coste: `O(n·m·K)` sumas más `≤ n·K` pasadas por la red, guardadas en una caché por `(a, b)` que comparten todas
  las variantes.

*Por qué es correcta.* Si los símbolos son independientes, la probabilidad de un corte es el producto de las de sus
grupos; maximizar el producto es maximizar la suma de logaritmos. La suma se descompone grupo a grupo, así que el
mejor reparto de `i` trazos en `j` fichas termina en un último grupo de `k` trazos precedido del mejor reparto de
`i−k` trazos en `j−1` fichas (si ese prefijo no fuera óptimo, cambiarlo por uno mejor mejoraría el total:
contradicción). La recurrencia prueba los `K` últimos grupos posibles, luego encuentra el óptimo. La diferencia con
`forced_groups` es el índice `j`: fija qué ficha corresponde a cada grupo.

**M0.3 Variantes de orden.** El LaTeX no dice en qué orden se escribió. `arbol_latex(latex)` analiza la cadena
(tras la misma limpieza que `expected_symbols`) con esta gramática y numera las fichas en el orden de
`expected_symbols`:

```text
sec   := item*
item  := base guiones?
base  := \frac grupo grupo | \sqrt ([ sec ])? grupo | ACENTO grupo | { sec } | FICHA
guiones := (_ grupo)? (^ grupo)?        (en cualquier orden)
grupo := { sec } | FICHA
ACENTO := \hat | \bar | \overline | \tilde | \widehat | \widetilde
```

Una función (`\sin`) da una ficha por letra. Si las fichas del árbol no coinciden una a una con `tokens`, se usa una
sola variante, la identidad.

| Estructura | Variantes (orden de escritura), la primera es la de partida |
|------------|--------------------------------------------------------------|
| `\frac{N}{D}` | N, raya, D · raya, N, D · N, D, raya |
| `\sqrt{R}` | raíz, R · R, raíz |
| `\sqrt[I]{R}` | I, raíz, R · raíz, I, R |
| acento sobre B | B, acento · acento, B |
| base con subíndice S y exponente E (también límites de ∑, ∏, ∫) | base, S, E · base, E, S |

Cada estructura lleva un índice de variante. Si el producto de variantes es `≤ 64` (`MAX_VARIANTES`), se prueban
todas. Si es mayor: descenso por coordenadas (se parte de la primera de cada una; para cada estructura, en orden, se
prueban sus variantes con las demás fijas y se queda la mejor; dos pasadas). Gana la variante con mayor `B[n][m]`;
en caso de empate, la que se probó primero.

**Trazo tardío** (`tardios=True`, solo con letra del usuario): si el mínimo `p` de algún símbolo es `< 0,2`, se
prueba a mover cada trazo `r` a cada posición `q ∈ [r−6, r−1]`, se repite M0.2 y se acepta el movimiento con mayor
total si mejora en más de `1,0`. Cubre la raya de la «t» o el punto de la «i» puestos después.

**Resultado:** `{"grupos": [(índices, etiqueta)], "p": [...], "total", "media", "min", "orden": v, "movido"}`, con
los grupos en orden de escritura.

**Control de calidad** (`aceptados(res, …)`, receta 3 de la auditoría). Para el símbolo `j`:

- `A(j)` = su etiqueta está entre las 5 primeras de la red, o el vecino kNN más cercano (distancia `≤ D_MAX`) tiene
  esa etiqueta.
- `aceptado(j) = A(j)` **o** (`A(j−1)` y `A(j+1)`), contando el borde del renglón como verdadero.

*Por qué la segunda parte.* Los grupos son trazos seguidos: el grupo `j` empieza donde acaba el `j−1` y acaba donde
empieza el `j+1`. Si los dos vecinos están bien cortados, el corte de `j` es exacto, y su etiqueta la da el LaTeX.
Así entran los ejemplos difíciles (los que la red aún lee mal), que son los que más enseñan; con solo `A(j)` se
quedarían fuera justo esos.

**M0.4 Reparto congelado por hash.**

```python
def u(sal, clave):   # número fijo en [0, 1) para cada expresión
    return int(hashlib.sha1(f"{sal}:{clave}".encode()).hexdigest()[:8], 16) / 2**32
```

- `clave = f"{set}#{id}"`, por ejemplo `general-200#g001`. Valores de control para los tests:
  `u("onepepe-examen-v1", "general-200#g001") = 0.562747`,
  `u("onepepe-examen-v1", "asignaturas-200#a001") = 0.581660`,
  `u("onepepe-validacion-v1", "general-200#g001") = 0.866588`.
- **Examen:** en cada estrato `(set, kind)` de `entrenos/<set>.json` (los 200 ids, escritos o no), se ordenan los
  ids por `(u("onepepe-examen-v1", clave), id)` y entran los primeros `int(0.20·N + 0.5)`.
- **Validación:** con los ids restantes de cada estrato, igual con la sal `onepepe-validacion-v1` y `0.15`.
- **Pliegues** (para la medida cruzada): `pliegue = int(sha1("onepepe-pliegues-v1:" + clave)[:8], 16) % 4`, sobre lo
  que no es examen.
- **Otros orígenes** (dataset nuevo, `fallos.jsonl`): examen si `u("onepepe-examen-v1", f"{origen}#{id}") < 0.20`.
- `examen.json` (`$ONEPEPE_DATA/math/`) guarda los ids. Una vez escrito no se reescribe; solo se añaden ids de
  orígenes nuevos.

*Por qué hash y no azar.* El reparto no depende de una semilla, ni del orden de los archivos, ni de cuántas
expresiones hay escritas: una expresión cae siempre en el mismo sitio. Como se reparte sobre los 400 ids, lo que
escribas mañana ya tiene sitio. *Por qué por expresión.* Los símbolos de una misma expresión se parecen entre sí
(misma sesión, mismo tamaño); si se repartieran por símbolo, el examen tendría «hermanos» de lo entrenado y mediría
de más. *Por qué por estratos.* Para que examen y entrenamiento tengan la misma mezcla de tipos (`expr`, `steps`,
`symbols`).

**M0.5 Referencia y medida cruzada.**

- `referencia.json`: el alineado ordenado de todos los renglones escritos, calculado **una vez** en T0 con el modelo
  actual y congelado (clave `set#id#renglón`, con `n_trazos` para detectar que se reescribió). Los renglones sin
  alineado, o con algún símbolo no aceptado, llevan `dudoso: true`: no cuentan en las medidas por símbolo y sí en
  «LaTeX exacto», que no necesita alineado.
- **Medida cruzada** (`medir.py --cruzada`): cuatro modelos; el modelo `k` se entrena con la receta fija (peso,
  épocas y temperatura elegidos en la validación) sin el pliegue `k` y sin el examen, y lee el pliegue `k` con un
  kNN que tampoco lo contiene. Reúne 941 símbolos, 126 renglones y 34 rayas, todos no vistos. Los cuatro modelos se
  guardan en `$ONEPEPE_DATA/pliegues/<receta>/`: medir un cambio que no reentrena solo cuesta cuatro lecturas.

**M0.6 «Si no mejora, no se queda», con números.** Para comparar dos medidas A y B sobre las mismas expresiones:
`Δ_e` = aciertos de B menos aciertos de A en la expresión `e`. Se remuestrean las expresiones con reposición 2000
veces (semilla 7) y se calcula `P = fracción de remuestreos con ΣΔ > 0`.

- `P ≥ 0,80`: **mejora**. `P ≤ 0,20`: **empeora**. En medio: **empate** (se queda si es más simple o si otra
  métrica de la aceptación mejora).

*Por qué.* Con tan pocas expresiones, dos puntos de diferencia pueden ser azar. Remuestrear expresiones enteras
respeta que los símbolos de una expresión aciertan o fallan juntos, y comparar por parejas elimina la dificultad
propia de cada expresión.

### M2. Entrenar sin fugas, parar de verdad y calibrar (T2)

**M2.1 La fuga actual.** `train_classifier.py:118` deforma cada muestra 30 veces (90 las propias) y después
`MLPClassifier(early_stopping=True)` aparta un 10 % al azar. Las copias de una misma muestra caen a los dos lados:
la «validación» mide si la red recuerda la muestra, no si generaliza. La parada temprana no para nunca a tiempo.

**M2.2 Reparto antes de deformar.**

| Origen | Entrenamiento | Validación (sin deformar) | Reservado |
|--------|---------------|---------------------------|-----------|
| MathWriting `symbols/` | resto | `int(id[-4:-2], 16) % 8 == 0` | `is_holdout(id)` (el 20 % actual; no cambia) |
| Trozos de fórmulas sintéticas | resto | fórmula nº `% 8 == 0` | — |
| Letra propia con expresión (`notebook = entreno:<set>#<id>`) | ni examen ni validación | validación de M0.4 | examen: nunca |
| Correcciones a mano (sin expresión) | todas | — | — |

La media y la desviación para normalizar se calculan solo con el entrenamiento.

**M2.3 Mezcla equilibrada.** `W` = copias deformadas por ejemplo propio.

- MathWriting: como mucho 80 muestras base por clase, cada una con 30 copias.
- Propio: copias por ejemplo de la clase `c`: `a_c = max(1, min(W, int(40·W / n_c)))`, con `n_c` ejemplos base. Una
  clase con hasta 40 ejemplos recibe `W` copias de cada uno; una con más recibe menos copias por ejemplo, y su total
  no pasa de `40·W`. Así «=» (112 ejemplos) no pesa 20 veces más que «π» (2).
- Trozos: misma función `augment` que los símbolos; copias por trozo `a_t = clip(round(0.15·N_s / n_t), 2, 30)`,
  con `N_s` el total de muestras de símbolos ya deformadas y `n_t` los trozos base. Los trozos quedan en torno al
  15 % del total.
- `W` se elige entre 10, 30 y 90: se entrena una vez con cada uno y gana el de menor `J` (M2.4); en caso de empate,
  el menor. Se guarda en `ultimo_entreno.json` (`personal_weight`); el botón «Reentrenar» de la app usa el
  guardado (30 si no hay) y no repite la elección.

**M2.4 Parada real.** Bucle propio con `MLPClassifier(hidden_layer_sizes=(384, 192), alpha=1e-4, batch_size=256,
learning_rate_init=1e-3, random_state=seed)` y una llamada a `partial_fit(X, y, classes=todas)` por época.

- Tras cada época: `NLL_p` y `NLL_m`, la log-verosimilitud negativa media en la validación propia y en la de
  MathWriting; `J = ½·NLL_p + ½·NLL_m`.
- Se guardan los pesos de la época con menor `J`. Se para tras 5 épocas sin mejorar, o a las 60.
- SUPUESTO a comprobar con un test de tres líneas antes de nada: cada `partial_fit` da una sola pasada a los datos
  (`n_iter_` sube de uno en uno) y conserva el estado del optimizador. Si no fuera así, usar `warm_start=True` con
  `max_iter=1`.

*Por qué `J` y no el acierto.* La log-verosimilitud usa toda la probabilidad (cuánto de seguro se equivoca), es más
estable con pocas muestras y es justo lo que consume el corte (`log p`). *Por qué mitad y mitad.* Hay muchas más
muestras de MathWriting que propias; con media simple, la letra del usuario no pintaría nada. Con mitades, ninguno
de los dos objetivos (tu letra, letra general) domina al otro.

**M2.5 Temperatura.** La red da valores `z_i` (antes del softmax) para la muestra `i`, de clase `y_i`. Con
`β = 1/T`, la probabilidad calibrada es `softmax(β·z_i)` y la pérdida:

```text
L(β) = (1/N) Σ_i [ log Σ_k exp(β·z_ik) − β·z_i,yi ]
L'(β) = (1/N) Σ_i [ E_i(z) − z_i,yi ]      con E_i(z) = Σ_k p_ik·z_ik  y  p_i = softmax(β·z_i)
L''(β) = (1/N) Σ_i Var_i(z) ≥ 0            con Var_i(z) = Σ_k p_ik·z_ik² − E_i(z)²
```

- Método de Newton: `β ← clip(β − L'(β)/max(L''(β), 1e-6), 0.125, 4)`, desde `β = 1`, 20 pasos. `T = 1/β`.
- Las muestras son la validación sin deformar, con pesos tales que la propia y la de MathWriting sumen ½ cada una.
  Las `z` son las de las clases permitidas (`mask`).
- `T` se guarda en el `.npz` como `temperature`. `SymbolClassifier` divide por `T` antes de enmascarar (1.0 si
  falta la clave).

*Demostración de la derivada.* `d/dβ log Σ_k e^{β z_k} = Σ_k z_k e^{β z_k} / Σ_k e^{β z_k} = E(z)`. Derivando otra
vez: `d/dβ E(z) = E(z²) − E(z)² = Var(z)`. Como la varianza no es negativa, `L` es convexa en `β`: hay un único
mínimo y Newton lo encuentra. *Por qué temperatura.* Dividir por `T > 0` no cambia el orden de las clases, luego el
acierto es el mismo; solo cambia cuánta seguridad declara la red. `T > 1` la baja (red demasiado segura, que es lo
que se midió: 18 de 111 fallos con confianza `≥ 0,95`). *Por qué esta pérdida.* La log-verosimilitud es una regla de
puntuación propia: en promedio se minimiza diciendo la probabilidad verdadera.

**M2.6 Aceptar un modelo nuevo** (receta 4.3 punto 7; `reentrenar.py`). Se cambia el modelo solo si se cumplen las
tres:

1. En el examen, «símbolos con corte y etiqueta» no empeora (M0.6: `P > 0,20`).
2. En MathWriting reservado, el acierto top-1 no baja más de 2 puntos.
3. Las sustituciones erróneas del examen no suben.

Si falta `examen_trazos.json`, se usa la validación para 1 y 3. El examen nunca entra en el entrenamiento.

### M5. Corte: penalización y rayas (T5)

**M5.1 Qué es la penalización.** El corte maximiza `Σ_símbolos [log p_top + log(1 − p_trozo) − λ] + pausas`
(`segment.py:131`). Equivale a un máximo a posteriori con una probabilidad previa `e^{−λ}` por símbolo: `λ` es
cuánto se cree, antes de mirar, que hay pocos símbolos. Cada símbolo ya resta por sí mismo (`log p ≤ 0`), así que
partir en dos siempre paga dos términos negativos: el sesgo natural es a juntar, y `λ = 1,2` lo aumenta. Medido:
juntados 8,9 % frente a partidos 2,5 %. Por eso `λ` debe bajar a entre 0 y 0,3.

**M5.2 Elección** (`elegir_corte.py`). Rejilla `λ ∈ {0; 0,1; 0,2; 0,3}` × variante de reglas `∈ {V0, V1, V2, V3}`
(16 casos):

| Variante | Reglas de `_close` |
|----------|--------------------|
| V0 | Las actuales |
| V1 | V0 + «rayas primero» (M5.3) |
| V2 | V1 + solape estricto: se quita el margen `−0,35·típico` de `segment.py:71` |
| V3 | V2 + `stack_gap = 0,6` |

- Error de corte de un caso: `E = ½·E_propio + ½·E_sint`, con `E = 100 − % de corte exacto`. `E_propio` sale de la
  medida cruzada (M0.5); `E_sint`, de 300 fórmulas de `synth.formulas(holdout=True, seed=11)`.
- Restricción: `partidos ≤ partidos(V0, λ = 1,2) + 0,5` puntos, en las dos partes.
- Gana el menor `E` que cumple la restricción; en caso de empate (diferencia `< 0,3`), la variante de número más
  bajo y el `λ` más alto, por ser lo más parecido a lo que ya hay.
- Los valores elegidos pasan a ser los valores por defecto de `SegmentParams` en el código.

*Por qué con letra no vista.* Con letra ya vista la red da `p` cercanas a 1 y cualquier `λ` parece bueno; lo que
importa es el `λ` cuando la red duda. *Por qué mitad sintético.* Da miles de símbolos con el corte verdadero
conocido y de otras manos: evita ajustar la regla a 126 renglones.

**M5.3 Rayas primero.** Para el trazo `i`, con caja de ancho `w_i` y alto `h_i`, y `típico = typical_size(trazos)`:

- `plano(i)`: `h_i < 0,25·w_i` y `w_i ≥ 0,3·típico`.
- `raya(i)`: `plano(i)` y `w_i > 1,5·típico`.
- `punto(i)`: `max(w_i, h_i) < 0,3·típico`.

Si `a` o `b` es una raya, `_close(a, b)` solo es verdadero en estos tres casos:

1. El otro es plano, `0,6 ≤ w_a/w_b ≤ 1,67`, se solapan en horizontal más de la mitad del más estrecho y el hueco
   vertical es `< 0,6·típico` (`=`, `≡`, la raya de `≤`).
2. El otro es un punto, su centro cae dentro del ancho de la raya y el hueco vertical es `< 0,6·típico` (`÷`).
3. El otro la **cruza**: `y0 < cy_raya < y1` y `min(cy_raya − y0, y1 − cy_raya) > 0,15·h_otro` (`+`, `t`, `7`, `H`).

*Por qué.* La regla de apilados actual da por cercanos a dos trazos que se solapan en horizontal y distan menos de
`1,0·típico` en vertical. Una raya de fracción se solapa en horizontal con todo su numerador y su denominador, que
están a menos de esa distancia: siempre «pueden» unirse, y por eso se traga al vecino en el 44,7 % de los casos. Un
trazo largo y plano solo forma símbolo con otro plano parecido, con puntos, o con un trazo que lo atraviesa.
Rozarla por arriba (el rabo de una «y» del numerador) no basta: de ahí el 15 %.

**M5.4 Reparación general** (`reparar.py`, tras la DP). Puntuación de un símbolo: `S(g) = log p_top + log(1 −
p_trozo) − λ`.

- Candidato a sobrante `d`: 1 o 2 trazos y (etiqueta en `{. \cdot ' , - | 1 l / ) (}` o `p_trozo ≥ 0,5` o
  `punto`).
- Pareja `s`: cualquier otro símbolo (no hace falta que sea vecino en el tiempo) cuya caja toca la de `d` (huecos
  `< 0,25·típico` en los dos ejes) o está apilada con ella (solape horizontal `> 0` y hueco vertical
  `< 0,6·típico`), con `≤ 5` trazos entre los dos.
- Ganancia: `G = S(d ∪ s) − S(d) − S(s)`. Se une la pareja de mayor `G` si `G > 0,5` y la etiqueta del conjunto
  está en `{= \leq \geq \neq \div \pm \equiv \approx i j ! : ; t 7 H x + k f A F E T \pi z Z y 4 5 \theta \times *
  \in}`. Se repite hasta que no haya ninguna (máximo 10).
- La regla actual de i/j (`p(i) + p(j) ≥ 0,1`) se conserva dentro del mismo módulo.

*Por qué `G > 0,5`.* `G` es el logaritmo de cuántas veces más probable es la lectura unida que las dos separadas;
`e^{0,5} ≈ 1,65`. Exigir margen evita unir por empates.

**M5.5 Papel de la raya** (`layout.bar_role(caja, otras, típico)`, usada por `layout` y por `recognizer`). Con los
mismos umbrales que `_parse_region`: hay algo encima y algo debajo dentro de su ancho → `frac`; solo una letra
debajo, y la raya mide `≤ 1,6` veces su ancho → `overline`; si no → `minus`. `recognizer` mueve la probabilidad de
`-`, `\frac` y `\overline` a la etiqueta de su papel antes del contexto. Así «−» y «raya de fracción» dejan de ser
un error de etiqueta (13 de los 111 medidos).

### M7. Clasificador v2: rasgos y vecinos (T7)

**M7.1 Qué falta hoy.** `features._normalize` divide por el lado mayor: una «x» y una «X» quedan idénticas. Y los
32 puntos recorren los trazos en el orden y sentido en que se escribieron: la misma forma con otro orden es otro
vector.

**M7.2 Vector v2** (556 valores; `features.VERSION = 2`):

| Bloque | Tamaño | Qué es |
|--------|-------:|--------|
| Secuencia | 160 | El actual: 32 puntos × (x, y, dx, dy, salto) |
| Direcciones | 128 | Histograma de 8 orientaciones en rejilla 4×4 (M7.3) |
| Rejilla | 256 | 16×16 suavizada (M7.4) |
| Contexto | 6 | Tamaño y posición respecto al renglón (M7.5) |
| Extras | 6 | Los actuales: proporción y número de trazos |

`symbol_features_v1` (el actual) se conserva. `symbol_features(strokes, linea=None, version=VERSION)` elige. El
`.npz` guarda `feature_version`; `SymbolClassifier` usa el extractor de la versión del modelo cargado, así que un
modelo viejo sigue funcionando con el código nuevo.

**M7.3 Histograma de direcciones.** Con el símbolo normalizado a `[−1, 1]²` (como ahora) y cada trazo remuestreado a
paso `0,05`:

1. Para cada tramo entre dos puntos seguidos del mismo trazo: longitud `ℓ`, punto medio `(mx, my)`, orientación
   `θ = atan2(dy, dx) mod π`.
2. Orientación: `b = θ/(π/8)`; reparte entre los cubos `⌊b⌋ mod 8` y `(⌊b⌋+1) mod 8` con pesos `1 − frac(b)` y
   `frac(b)`.
3. Posición: `cx = (mx+1)·2 − 0,5`, `cy = (my+1)·2 − 0,5`; reparte entre las cuatro celdas vecinas con pesos
   bilineales (recortando a `[0, 3]`).
4. `H[celda_y, celda_x, cubo] += ℓ · peso_orientación · peso_x · peso_y`.
5. Normaliza: `H ← sqrt(H / Σℓ)`.

*Demostración de que no depende del orden ni del sentido.* `H` es una suma, sobre los tramos, de una función de
(punto medio, orientación módulo π, longitud). Cambiar el orden de los trazos solo reordena los sumandos. Recorrer un
trazo al revés cambia `θ` por `θ + π`, que es la misma orientación módulo π, y no cambia ni el punto medio ni la
longitud. ∎ *Por qué repartir entre cubos vecinos.* Sin reparto, un tramo a 22,4° y otro a 22,6° caerían en cubos
distintos: un temblor mínimo cambiaría el vector a saltos. Con reparto lineal, `H` cambia de forma continua con el
dibujo. *Por qué la raíz.* Iguala la varianza de cubos llenos y vacíos (un trazo largo no tapa a los cortos).

**M7.4 Rejilla 16×16.** Cada punto del remuestreo suma 1 repartido bilinealmente entre sus cuatro celdas; después
un suavizado `[1 2 1]/4` en cada eje, y `min(valor, 1)`. Sustituye a la rejilla binaria de 10×10.

**M7.5 Contexto del renglón.** `linea = (típico, pendiente, corte)`, o `None`.

- `típico`: `typical_size` de los trazos nuevos más las cajas de los símbolos conocidos del renglón.
- Línea base: con los puntos `(cx_i, fondo_i)` de los trazos de alto `≥ 0,5·típico`. Si hay 4 o más y abarcan más
  de `4·típico` en horizontal, `pendiente` = mediana de las pendientes de todas las parejas (Theil–Sen), limitada a
  `±0,3`; si no, 0. `corte = mediana(fondo_i − pendiente·cx_i)`. Con menos de 2 trazos altos: `linea = None`.
- Para un grupo de caja `(x0, y0, x1, y1)`, con `w = max(x1−x0, 0,02·típico)`, `h` igual, y
  `base = corte + pendiente·(x0+x1)/2`:

```text
c0 = 1 (hay contexto)                 c3 = clip((y0 − base)/típico, −3, 3)/3
c1 = clip(ln(w/típico), −3, 2)/2      c4 = clip((y1 − base)/típico, −3, 3)/3
c2 = clip(ln(h/típico), −3, 2)/2      c5 = clip(ln(max(w, h)/típico), −3, 2)/2
```

Sin contexto, los seis valen 0.

- En el entrenamiento: los símbolos sueltos de MathWriting van sin contexto; los propios y los extraídos de fórmulas,
  con el de su renglón, borrado al azar el 30 % de las veces. Al deformar, el contexto se recalcula sobre el símbolo
  deformado, con `típico·exp(N(0; 0,1))` y `base + N(0; 0,08·típico)`.

*Por qué logaritmo.* El tamaño es una razón: «el doble» y «la mitad» deben pesar igual y con signo contrario, y el
resultado no debe depender del zoom. *Por qué Theil–Sen.* La mediana de pendientes ignora los fondos raros (una
«g» que baja, un exponente), que con mínimos cuadrados torcerían la recta. *Por qué borrar el contexto a veces.* La
red debe funcionar también cuando no lo hay (un solo trazo en la página, símbolo corregido a mano).

**M7.6 kNN.** Distancia entre dos símbolos: `d = 1 − cos(e_a, e_b)`, con `e` la última capa oculta. Para vectores de
norma 1, `‖e_a − e_b‖² = 2 − 2·cos = 2d`: ordenar por coseno es ordenar por distancia euclídea sobre la esfera. Se
usa el coseno porque la norma de `e` crece con la seguridad de la red y no con la forma.

```text
p_knn(c) = Σ_{i ∈ vecinos, y_i = c} e^{−d_i/τ} / Σ_{i ∈ vecinos} e^{−d_i/τ}        (K = 5 vecinos con d ≤ D_MAX)
λ        = λ_max · e^{−d_1/τ} · max_c p_knn(c)
p        = (1 − λ)·p_red + λ·p_knn
```

- `λ` es grande solo si el vecino más cercano está muy cerca **y** los vecinos están de acuerdo. Lejos de tus
  ejemplos `λ → 0` y la respuesta es la de la red: el kNN no puede estropear lo que no conoce.
- Al cambiar de red cambian las `e`: se reajusta la rejilla `τ ∈ {0,05; 0,1; 0,2}` × `D_MAX ∈ {0,2; 0,3; 0,4}` ×
  `λ_max ∈ {0,4; 0,6; 0,8}` (27 casos) con la medida cruzada; gana el mayor top-1 con el corte bueno; en caso de
  empate, los valores actuales (`0,1; 0,3; 0,6`).

**M7.7 Datos genéricos.** `extraer_simbolos.py` saca símbolos de las fórmulas `train/` de MathWriting con M0.2 (sin
trazos tardíos):

- Orden de las fórmulas: por `u("onepepe-extraccion-v1", id)`. Se descartan las que tienen fichas fuera de `CORE`,
  más de 25 fichas, `\begin`, o `n` fuera de `[m, 4m]`.
- Una fórmula vale si el 80 % o más de sus símbolos cumple `A(j)` (solo red). De ella entran los símbolos
  `aceptado(j)`.
- Se para cuando todas las clases de `CORE` tienen 400 símbolos, o a las 30 000 fórmulas.
- Segunda ronda: se entrena un modelo intermedio con lo extraído y se repite la extracción con él.
- Salida: `$ONEPEPE_DATA/mathwriting-extraido/simbolos.jsonl` (etiqueta, trazos, id de fórmula, `linea`). Nunca
  en git. Validación: lo mismo sobre 3 000 fórmulas de `valid/`.
- Tope en el entrenamiento: 400 por clase.

*Por qué más datos genéricos antes que más propios.* Hoy hay unas 22 muestras por clase de otras manos: la red no
puede aprender qué es una «b» en general, solo memorizar las tuyas. Con 400, la forma general viene de MathWriting y
tu letra solo tiene que corregir el estilo.

### M8. Confianza (T8)

**M8.1 Por qué no basta la temperatura.** La confianza final es `c = p_top·(1 − p_trozo)` **después** del kNN, el
glosario y el contexto (`recognizer.py:61`), que reordenan y renormalizan. La temperatura calibra la red, no ese
número. El umbral se elige con la confianza final y con letra no vista.

**M8.2 Regla de sustitución.** Un símbolo se sustituye por el limpio solo si se cumplen las tres:

1. `c ≥ τ*`.
2. La red está de acuerdo: `same_shape(y_red, ŷ)`, con `y_red` la primera candidata antes del kNN y `ŷ` la final.
3. El kNN no contradice: no hay vecinos a distancia `≤ D_MAX`, o `same_shape(y_knn, ŷ)`.

*Por qué el acuerdo.* Si dos lectores fallaran de forma independiente con frecuencias `e1` y `e2`, y al fallar
eligieran al azar entre las otras clases, coincidirían en la misma lectura errónea con probabilidad
`≈ e1·e2/(nº de clases − 1)`, mucho menor que `e1` o `e2`. El kNN usa la capa interna de la red, así que no son
independientes y la ganancia real es menor, pero el acuerdo sigue filtrando justo los casos en que tu forma de
escribir se aparta de la general. Se usa `same_shape` porque «x» y «X» dibujan el mismo símbolo limpio.

**M8.3 Umbral calibrado** (`calibrar_umbral.py`). Datos: todos los símbolos predichos en la medida cruzada.

- Para cada `τ ∈ {0,90; 0,95; 0,97; 0,98; 0,99; 0,995; 0,999}`: `n(τ)` sustituidos (con las tres reglas), `k(τ)`
  erróneos entre ellos, cobertura `n(τ)/N`.
- Cota superior de Wilson del error, unilateral al 90 % (`z = 1,2816`), con `q = k/n`:

```text
CS(k, n) = [ q + z²/(2n) + z·sqrt( q(1−q)/n + z²/(4n²) ) ] / (1 + z²/n)
```

- `τ*` = el menor `τ` con `CS ≤ 0,02` y cobertura `≥ 0,60`. Si ninguno cumple las dos: el de mayor cobertura entre
  los que cumplen `CS ≤ 0,02`, y se informa de la cobertura que falta. Si ninguno cumple `CS ≤ 0,02`: `τ* = 0,999`
  y la aceptación queda «no cumplida».
- Umbral del renglón: `τ_r` = el menor de `{0,5; 0,6; 0,7; 0,8; 0,9}` tal que los renglones con
  `mín_símbolos c ≥ τ_r` están enteros bien el 90 % de las veces; 0,7 si ninguno.
- Se guardan en `$ONEPEPE_DATA/math/confianza.json`: `{"umbral", "umbral_renglon", "n", "k", "cobertura", "fecha"}`.
  El auxiliar usa `max(umbral pedido por la app, umbral)`.

*De dónde sale la fórmula.* Si el error verdadero es `p`, el observado `q` se desvía de `p` unas `sqrt(p(1−p)/n)`.
La cota es el mayor `p` compatible con lo observado: `q = p − z·sqrt(p(1−p)/n)`. Elevando al cuadrado queda una
ecuación de segundo grado en `p`, cuya raíz mayor es `CS`. *Por qué no usar `k/n` sin más.* Con 0 errores en 100
sustituciones, `k/n = 0`, pero un error real del 2 % daría 0 errores el 13 % de las veces. La cota da 1,6 % para
`n = 100` y 0,5 % para `n = 300`: obliga a tener datos antes de fiarse.

### M10. Las N mejores lecturas y la gramática (T10)

**M10.1 Qué se busca.** Cada símbolo `j` tiene candidatas con probabilidad `p_j(·)`. Hoy se toma la primera de cada
uno. Una lectura completa `y = (y_1 … y_n)` puntúa:

```text
S(y) = Σ_j log p_j(y_j) + G(y)
G(y) = 3,0·[se puede analizar] − 2,0·(paréntesis sin pareja) − 2,0·(operadores mal puestos)
```

- «Se puede analizar»: cada lado de cada relación pasa por `_Parser(...).full()` sin `ParseError`.
- «Operador mal puesto»: un operador binario al principio (salvo signo) o al final (salvo el «=» final), o dos
  seguidos (salvo signo tras operador).

*Por qué.* Por Bayes, `P(lectura | tinta) ∝ P(tinta | lectura)·P(lectura)`. El contexto de pares es una
probabilidad previa débil. `G` es otra: `e^{3} ≈ 20` a 1 a favor de que lo escrito sea una expresión bien formada.
Una lectura 5 veces menos probable para la red (`log 5 = 1,6`) pero analizable gana a una que no lo es.

**M10.2 Búsqueda.**

1. Si la lectura de primeras candidatas tiene `G = 3,0` (analizable y sin faltas), se devuelve tal cual: no se
   busca. La mayoría de los renglones acaba aquí.
2. Margen de cada símbolo: `m_j = log p_j(1ª) − log p_j(2ª)`. Se abren los 5 de menor margen con `m_j < 3,0`. Para
   cada uno, hasta 3 candidatas con `log p ≥ log p_j(1ª) − 3,0`. Los símbolos fijos (`known`) no se abren.
3. Coste de elegir la candidata de rango `r` en `j`: `Δ_j(r) = log p_j(1ª) − log p_j(r) ≥ 0`, creciente en `r`.
4. Montículo ordenado por coste total. Se mete la lectura de todo primeras (coste 0, último índice 0). Al sacar
   una lectura con último índice `a`, se meten sus sucesoras: subir un rango en la coordenada `b`, para cada
   `b ≥ a`, con último índice `b`.
5. Se sacan como mucho 32 lecturas. Para cada una: `layout.parse`, `G`, `S = −coste + G`.
6. Parada anticipada: si `coste_actual − coste_de_la_mejor > premio − G_de_la_mejor` (premio = 3,0), ninguna
   posterior puede ganar: su coste es mayor o igual y su `G` no pasa del premio.
7. Se cambia de lectura solo si `S(mejor) − S(primeras) > 0,5`.

*Por qué salen de mejor a peor.* Cada sucesora cuesta igual o más que su madre (`Δ` crece con el rango). El
montículo saca siempre la de menor coste entre las pendientes, y cualquier lectura no generada aún desciende de una
pendiente con coste menor o igual: no puede adelantarse (es el argumento de Dijkstra). *Por qué `b ≥ a`.* Cada
lectura se genera por un solo camino (subiendo coordenadas en orden no decreciente): no hay repetidas.

**M10.3 Contexto en el orden del árbol.** `layout.parse_with_order(syms)` devuelve el árbol y el orden de los
símbolos como en el LaTeX: raya, numerador, denominador; raíz, índice, radicando; acento, base; base, subíndice,
exponente. `reading_order` usa ese orden. *Por qué.* El modelo de pares se aprendió de secuencias LaTeX de
MathWriting; leer de izquierda a derecha por el centro de las cajas mezcla numerador y denominador y le pregunta
por pares que nunca vio.

## 7. Especificación por tarea

Plantilla fija: **Rama · Agente · Bloque A · Leer · Lógica · Ediciones · Tests · Aceptación · Devuelve.** Las rutas
sin carpeta son de `tools/onepepe-math/`; `om/` abrevia `onepepe_math/`. «Devuelve» es siempre el formato de §2.3;
solo se indica qué líneas `MEDIDA` lleva.

### T0. Banco de medida

- **Rama:** `feat/banco-de-medida`. **Agente:** S-A; las sondas, H-T. **Bloque A:** A0, A5 (ruta de sesiones).
- **Leer (todo de una vez):** `om/{forced,segment,entreno,latex_tokens,symbols,classifier,personal,recognizer,
  layout,pagina}.py`; `tests/{test_segment,test_pagina,test_funciones}.py`;
  `~/.local/share/onepepe/auditoria-2026-11-15/medidas/{common,ev,e1b,e2e5,e4}.py` y `sondas/{build,s01,s02,s09,
  s10,s12}.py`; §6 M0.
- **Lógica:**
  1. `om/alinear.py` según M0.1–M0.4. Firmas:

     ```python
     def arbol_latex(latex): ...                       # -> (nodo, n_fichas)
     def variantes_de_orden(latex, tokens): ...        # -> generador de permutaciones (M0.3)
     def ordered_groups(strokes, tokens, classifier, latex=None, variantes=True, tardios=False): ...  # dict o None
     def aceptados(res, strokes, classifier, personal=None): ...   # -> [bool]
     def u(sal, clave): ...
     def reparto(sets): ...     # {set: [(id, kind)]} -> {"examen": {set: [ids]}, "validacion": {set: [ids]}}
     def pliegue(clave): ...    # 0..3
     ```

     `classifier.labels` puede no existir en un clasificador de pruebas: usar `getattr(clf, "labels", clf.classes)`.
  2. `medir.py`:
     - `cargar_renglones(dir)`: como `common.load_lines` (reparte por renglones guía con `_split_by_guides`).
     - `--crear-examen`: escribe `examen.json` (M0.4), `referencia.json` (M0.5) y `examen_trazos.json` (trazos,
       tiempos, LaTeX y referencia de los renglones del examen, para que `reentrenar.py` no dependa de dónde
       estén las sesiones). Si `examen.json` existe, no lo reescribe.
     - Estado de cada símbolo de referencia (`ev.classify`): **exacto**, **juntado**, **partido**, **mixto**.
     - Modos: `--conjunto {examen,validacion,entreno,todo}` (por defecto `examen`), `--cruzada` (M0.5; necesita los
       modelos de pliegue, que crea T2), `--incremental 0.8` (lotes por pausa, como `e4.run_inc`), `--sin-knn`,
       `--modelo RUTA`, `--ref {O,F}`, `--sin-variantes`, `--solo-sin-frac`, `--salida f.json`,
       `--comparar a.json b.json` (M0.6), `--sondas`, `--fallos` (añade `fallos.jsonl`, si existe, como otro
       origen con el reparto de M0.4). `--cruzada` y `--fallos` se escriben ya aquí para que T2 y T11, que van
       en paralelo, no tengan que tocar `medir.py`.
     - Métricas (clave del JSON → meta del plan):

       | Clave | Definición | Meta |
       |-------|------------|------|
       | `corte_y_etiqueta` | % de símbolos exactos y con la etiqueta bien | ≥ 90 |
       | `renglon_perfecto` | % de renglones con todos sus símbolos bien | ≥ 60 |
       | `numeros_2mas` | % de números de 2 o más cifras con todas bien (vecindad: `ev._adj`) | ≥ 90 |
       | `raya_papel` | % de rayas de fracción exactas, leídas `-` o `\frac`, en un renglón cuyo LaTeX leído tiene tantos `\frac` como el esperado | ≥ 90 |
       | `sust_erroneas` | % de símbolos con `clean` no nulo que no son exactos con etiqueta igual | ≤ 2 |
       | `sondas_erroneas` | nº de sondas con un resultado mostrado distinto del esperado | 0 |

       Además: `corte_exacto`, `juntado`, `partido`, `mixto`, `latex_exacto`, `raya_corte`, `raya_corte_etiqueta`,
       `sust_cobertura`, `sust_erroneas_visibles` (sin contar parejas `same_shape`), `red_top1`, `red_top5`,
       `ms_por_renglon`, `n` de cada una, intervalo de Wilson al 90 % y `por_expresion` (para M0.6).
  3. `tests/sondas_util.py`: el compositor de `sondas/build.py` sin rutas fijas. `tests/test_sondas.py` (H-T): una
     prueba por fila de la tabla de T3 y T4 de este documento, más las que hoy pasan (lista «Qué funciona» de la
     auditoría, con la salida actual como valor esperado). Las que hoy fallan llevan `@unittest.expectedFailure`.
- **Ediciones:** solo archivos nuevos: `om/alinear.py`, `medir.py`, `tests/test_alinear.py`,
  `tests/sondas_util.py`, `tests/test_sondas.py`.
- **Tests** (`tests/test_alinear.py`). Clasificador falso «por plantilla»: la prueba da la lista de símbolos en orden
  de escritura como `(etiqueta, nº de trazos)`; cada trazo lleva en su coordenada `y` a qué símbolo pertenece. A un
  grupo que coincide con un símbolo entero le da 0,9 de su etiqueta; a un trazo plano suelto, 0,45 de «-» y 0,05
  de «=»; a cualquier otro grupo, 0,9 de «trozo»:

  | Test | Escrito (en este orden) | Esperado |
  |------|-------------------------|----------|
  | `test_orden_respeta_fichas` | «2x+1=7»: `(2,1) (x,2) (+,2) (1,1) (=,2) (7,2)` | Tamaños de grupo `1, 2, 2, 1, 2, 2`: «1» 1 trazo, «=» 2, «7» 2 |
  | `test_frac_raya_en_medio` | `\frac{1}{2}`: `(1,1) (\frac,1) (2,1)` | Etiquetas en orden de escritura: `1, \frac, 2` |
  | `test_frac_raya_antes` y `_despues` | `(\frac,1) (1,1) (2,1)` y `(1,1) (2,1) (\frac,1)` | La raya es el grupo 0 y el 2 |
  | `test_acento_despues` | `\hat{y}`: `(y,2) (\hat,1)` | El sombrero es el último grupo |
  | `test_limites_al_reves` | `\sum_{i=1}^{3}`: `(\sum,1) (3,1) (i,2) (=,2) (1,1)` | Alineado no nulo; «3» antes que «i» |
  | `test_tardio` | «it»: `(i,1) (t,1)` y después el punto de la «i» y la raya de la «t» | Con `tardios=True`: «i» 2 trazos, «t» 2 trazos |
  | `test_imposible` | 2 trazos, 3 fichas | `None` |
  | `test_ficha_desconocida` | Ficha que no es clase del modelo | Alinea y la marca `desconocido` |
  | `test_muchas_variantes` | 5 fracciones seguidas (243 variantes) | Termina y da un alineado no nulo |
  | `test_arbol_no_cuadra` | LaTeX con una orden que el árbol no conoce | Una sola variante (la identidad) y alineado no nulo |
  | `test_hash_fijo` | Los tres valores de control de M0.4 | Iguales con 6 decimales |
  | `test_reparto` | Los dos sets del repositorio | 40 ids de examen por set; examen y validación disjuntos; no cambia al añadir otro set |
  | `test_estados` | Referencia `[[0],[1,2]]` frente a lo leído `[[0,1],[2]]` | `juntado` y `mixto` |
- **Aceptación:**
  - `$PY medir.py --sesiones "$ENTRENOS" --conjunto todo --ref O --sin-variantes --solo-sin-frac` da corte exacto
    `90,0 ± 1` y `corte_y_etiqueta` `79,0 ± 1` (cifras «visto, O» de la auditoría).
  - `$PY medir.py --sesiones "$ENTRENOS" --crear-examen` deja `examen.json` con 80 ids, 22 de ellos ya escritos.
  - Tests en verde; `--sondas` informa de 8 resultados erróneos.
- **Devuelve:** `MEDIDA` con corte exacto, `corte_y_etiqueta`, `sust_erroneas` (todo, visto) y tamaño del examen.

### T1. Reimportar los entrenos

- **Rama:** `fix/reimportar-entrenos`. **Agente:** S-A. **Bloque A:** A1 solo decide la promoción final; la tarea
  se hace siempre en la copia de trabajo.
- **Leer:** `om/{entreno,alinear,forced,personal,server}.py`, `tests/test_entreno.py`, `importar_mi_letra.py`,
  `crates/rnote-ui/src/entreno.rs` líneas 840–880 (cómo usa `imported`); §6 M0.2–M0.4.
- **Lógica:**
  1. `import_session(session, classifier, store, source=None, forzar=False, examen=None)`:
     - Salta los ids que están en `examen` (por defecto, los de `examen.json`). Los devuelve también en
       `"imported"` (para que la app los marque) y en una clave nueva `"examen"`.
     - Con `forzar=True` ignora la marca `imported`.
     - Por renglón: `ordered_groups(..., latex=latex, tardios=True)`. Si es `None`, la expresión va a `failed`.
     - Con `ok = aceptados(...)`: los símbolos con `ok[j]` se guardan con
       `notebook = f"entreno:{set}#{item_id}"`. Los demás se escriben en `$ONEPEPE_DATA/math/revision.jsonl`
       (mismos campos más `motivo`) y no entran ni en el kNN ni en el entrenamiento.
     - Los trozos (`junk_examples`) solo se sacan de los renglones con todos sus símbolos aceptados.
     - Devuelve además `"revision": n`.
  2. `reimportar_entrenos.py [--sesiones DIR] [--aplicar]`: sin `--aplicar` trabaja sobre una copia temporal e
     informa; con `--aplicar`: copia `correcciones.jsonl` a `correcciones.<AAAAMMDD-HHMMSS>.bak`, quita los
     ejemplos cuyo `notebook` empieza por `entreno:`, importa todas las sesiones con `forzar=True` y escribe el
     informe (entran, a revisión, fallidas, del examen). Nunca escribe en las sesiones.
- **Ediciones:** `om/entreno.py`; nuevo `reimportar_entrenos.py`; `tests/test_entreno.py`.
- **Tests** (primero el que falla):

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_import_usa_orden` | Sesión con «2x+1=7» y el clasificador falso de T0 | «=» se guarda con 2 trazos |
  | `test_import_salta_examen` | Sesión con un id del examen | No se guarda; el id está en `imported` y en `examen` |
  | `test_import_revision` | Un símbolo que no cumple `aceptado` | Va a `revision.jsonl`; no está en el almacén |
  | `test_notebook_lleva_id` | Cualquier importación | `notebook == "entreno:general-200#g001"` |
  | `test_reimportar_copia` | Almacén temporal con `entreno:*` y correcciones a mano | Existe el `.bak`; las correcciones a mano siguen; no queda ningún `entreno:` antiguo |
- **Aceptación:** tests en verde; `$PY reimportar_entrenos.py --sesiones "$ENTRENOS" --aplicar` (sobre la copia de
  trabajo) termina y su informe cuadra: símbolos que entran + revisión + fallidos + examen ≈ 1158.
- **Devuelve:** `MEDIDA` con «entran / revisión / fallidas / examen».

### T2. Entrenamiento sin fugas

- **Rama:** `fix/entrenamiento-sin-fugas`. **Agente:** S-A. **Bloque A:** ninguna.
- **Leer:** `train_classifier.py`, `reentrenar.py`, `evaluar_mi_letra.py`, `om/{classifier,features,synth,
  personal,alinear}.py`, `medir.py`, `tests/test_reentrenar.py`; §6 M2.
- **Lógica:**
  1. **Modelo de referencia «antes»:** con el `train_classifier.py` sin tocar y el almacén de T1 (sin examen),
     entrenar `$ONEPEPE_DATA/base/simbolos.npz` y medir el examen (`medir.py --modelo …`). Es el «antes».
  2. Reescribir `train_classifier.py` con M2.2–M2.5. Opciones nuevas: `--personal-weight {10,30,90,auto}`
     (`auto` prueba los tres), `--epocas-fijas N` (sin parada: para los pliegues), `--excluir-pliegue K`,
     `--max-epocas 60`, `--paciencia 5`. El progreso sigue saliendo por `stderr` con los mismos textos que busca
     `reentrenar.py` (`símbolos cargados`, `ejemplos de 'trozo'`, `entrenando con`, `entrenado en`,
     `modelo de contexto`).
  3. El `.npz` gana `temperature`, `feature_version` (1), `epocas`, `personal_weight`.
  4. `om/classifier.py`: leer `temperature` y dividir por ella en `probabilities` y
     `probabilities_and_embedding`, antes de la máscara.
  5. `reentrenar.py`: el reparto de 70/30 por fecha desaparece. Entrena con todo lo que no es examen ni validación;
     acepta según M2.6; guarda `personal_weight` y `epocas` en `ultimo_entreno.json`.
  6. Pliegues: nuevo `entrenar_pliegues.py --receta v1` entrena los cuatro modelos con `--epocas-fijas` y el peso
     elegidos, en `$ONEPEPE_DATA/pliegues/v1/`. No se toca `medir.py` (T11 va en paralelo).
- **Ediciones:** `train_classifier.py`, `reentrenar.py`, `om/classifier.py`, nuevo `entrenar_pliegues.py`,
  `tests/test_reentrenar.py`, nuevo `tests/test_entrenamiento.py`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_partial_fit_una_epoca` | 3 llamadas a `partial_fit` | `n_iter_ == 3` |
  | `test_reparto_antes_de_deformar` | 20 muestras con id | Ningún id está a la vez en entrenamiento y validación, tampoco sus copias |
  | `test_tope_por_clase` | Clase con 100 ejemplos propios, `W = 30` | Total de copias `≤ 40·30` |
  | `test_temperatura_newton` | `z` sintéticas = `2·z0`, con etiquetas muestreadas de `softmax(z0)` (N = 5000, semilla fija) | `T` entre 1,8 y 2,2 |
  | `test_temperatura_no_cambia_orden` | Un modelo con `T = 3` | Misma primera candidata que con `T = 1` |
  | `test_examen_fuera` | Almacén con un ejemplo `entreno:…` del examen | No aparece en el entrenamiento |
  | `test_estimate…` (el que hay) | — | Sigue pasando |
- **Aceptación** (plan): en el examen, frente al modelo «antes», `corte_y_etiqueta` mejora (M0.6) y
  `sust_erroneas` no sube; MathWriting reservado top-1 no baja más de 2 puntos.
  `$PY medir.py --comparar base.json nuevo.json`.
- **Devuelve:** `MEDIDA` con `corte_y_etiqueta`, `sust_erroneas`, top-1 de MathWriting, `T`, `W` y épocas.

### T3. Evaluación correcta

- **Rama:** `fix/evaluacion-correcta`. **Agente:** S-B. **Bloque A:** ninguna.
- **Leer:** `om/{layout,evaluate,pagina,symbols,glossary}.py`, `tests/{test_sondas,sondas_util,test_evaluate,
  test_pagina,test_funciones,test_acentos}.py`, tabla «Qué falla» de la auditoría.
- **Lógica:**
  1. **3a, operador tras una base** (`layout._parse_row`, bucle de guiones, `layout.py:260–288`). Para cada
     símbolo `s` que sigue a `base`, en este orden:
     - `OPS(s)` = `category(s.label) ∈ {binop, relation}` o `s.label ∈ {\cdot, \sim, \equiv}`.
     - Si `OPS(s)` y ya hay exponente: sigue en él solo si `s.cy < base_mid − 0,25·típico`. Si ya hay subíndice:
       solo si `s.cy > base_mid + 0,12·típico`. Si no cumple: fin.
     - Si `OPS(s)` y no hay nada abierto: solo puede abrir **exponente**, y solo si `s.label ∈ {-, +}`,
       `s.cy < base_top + 0,35·base.h` y el siguiente símbolo `n` no es operador, mide
       `< 0,85·max(base.h, típico)`, tiene `_middle(n) < base_mid − 0,25·típico` y `n.box[3] < base_bot −
       0,05·típico`. Si no: fin. Un operador nunca abre subíndice.
     - Resto de símbolos: regla actual con `smaller = s.h < 0,85·max(base.h, típico)` (se quita
       `or s.label in _FLAT`).
  2. **3b, límites de ∑, ∏ y ∫** (`_parse_region`, paso nuevo antes de las fracciones). Para cada `B` con etiqueta
     `\sum`, `\prod` o `\int`:
     - `dentro(s)`: más de la mitad del ancho de `s` cae en `[B.x0 − 0,6·típico, B.x1 + 0,6·típico]`, y
       `s.h < 0,8·B.h`.
     - `debajo` = `dentro(s)` con `s.box[1] ≥ B.box[3] − 0,10·B.h` y `s.box[1] − B.box[3] < 1,0·típico`.
     - `encima` = `dentro(s)` con `s.box[3] ≤ B.box[1] + 0,10·B.h` y `B.box[1] − s.box[3] < 1,0·típico`.
     - Si hay alguno: nodo `Sym(label=B.label, box=unión)` con `node = ("subsup" | "sub" | "sup", ("sym",
       B.label), _parse_region(debajo), _parse_region(encima))` y `limits_done = True`; `_parse_row` no le busca más
       guiones.
     - Límites escritos a la derecha: en `_parse_row`, si `base` es `\sum`, `\prod` o `\lim`, los símbolos «=» y
       `\rightarrow` no cortan el subíndice cuando ya está abierto y `s.cy > base_mid + 0,12·típico`.
  3. **3c, cifras y prima** (`_parse_row`):
     - Cifra tras cifra: `mismo = |s.box[3] − base.box[3]| ≤ 0,2·max(base.h, típico)`;
       `sube = s.box[3] < base.box[3] − 0,35·base.h` y `s.h < 0,85·max(base.h, típico)`. Si `mismo` o no `sube`: la
       cifra sigue en la línea (mismo número). Solo si `sube` es exponente. Nunca subíndice.
     - Hueco: si entre dos cifras seguidas de la línea hay más de `0,8·típico`, se inserta entre ellas el nodo
       `("sym", r"\,")`. `evaluate._tokens` no une cifras a través de él y `_Parser` lanza
       `ParseError("hueco entre números")`: sale «?», no un número fundido.
     - Prima: `'` tras una letra, un acento o `)`, con `s.box[3] < base_mid`: nodo `("prime", nodo_base, n)` con
       `n` = primas seguidas. `to_latex` escribe `f'`. `_tokens` da la ficha `["prime", (nombre, n), None, None]`;
       en esta tarea `_Parser.atom` lanza `ParseError("derivada de una función sin definir")`.
  4. **3d, fracción** (`_parse_region`, `layout.py:97–98`): el límite vertical pasa de `1,5·típico` a `1,5·H`, con
     `H = max(típico, percentil 90 de los altos de los símbolos no planos de la región)`.
  5. **Derivadas de Leibniz** (`evaluate._leibniz(tokens, definidos=())`, llamada al final de `_tokens`). Con
     `D ∈ {d, \partial}` y `n` = exponente entero o nada (1):

     | Patrón | Ficha resultante |
     |--------|------------------|
     | fracción con numerador `[D^n]` y denominador `[D, v^n]` | `["deriv", (None, [(v, n)]), None, None]` |
     | fracción con numerador `[D^n, f]` y denominador `[D, v^n]` | `["deriv", (f, [(v, n)]), None, None]` |
     | en línea: `[D] [f] [/] [D] [v]` | `["deriv", (f, [(v, 1)]), None, None]` |
     | fracción con numerador `[\partial^2, f?]` y denominador `[\partial, v1, \partial, v2]` | `["deriv", (f, [(v1, 1), (v2, 1)]), None, None]` |

     No se aplica si `"d" ∈ definidos`. En `_Parser.atom`: con `f = None`, `sp.Derivative(self.unary(), *vars)`
     (sustituye al caso actual de `evaluate.py:223–235`); con `f`, en esta tarea,
     `ParseError("derivada de una función sin definir")`.
  6. **Llamada sin definir:** en `_Parser.atom`, una letra `f, g, h, F, G` que no está en `glossary` y va seguida
     de `(` lanza `ParseError("función sin definir")`.
  7. **3e:** en `_symbol`, `e` es `sp.E` si no está en `glossary`. En `_with_units`: `q = q.to_base_units()`;
     si su dimensión coincide con la de `N, J, W, Pa, C, V, ohm, Hz` (en ese orden), `q = q.to(unidad)`;
     `to_compact()` solo si queda una única unidad. (Hipótesis fuerte de la causa de «686 g·km/s²»: `to_compact`
     sobre `kg·m/s²`; el test lo confirma o lo desmiente.)
- **Ediciones:** `om/layout.py`, `om/evaluate.py`, `tests/test_sondas.py` (quitar `expectedFailure` a las que
  pasan), `tests/test_evaluate.py`.
- **Tests** (sondas; `v` = con «Variables»):

  | Sonda | Renglones | Hoy | Tras T3 |
  |-------|-----------|-----|---------|
  | `paren_mas` | `(x+1)+3=` | `3*x + 3` | `x + 4` |
  | `segunda_derivada` | `\frac{d^2}{dx^2}(x^3)=` | `3*x**2` | `6*x` |
  | `dxdt_frac` (v) | `x=t^2` / `\frac{dx}{dt}=` | `x/t` | `?` |
  | `dydx_frac` (v) | `y=x^2` / `\frac{dy}{dx}=` | `y/x` | `?` |
  | `dxdt_linea` (v) | `x=t^2` / `dx/dt=` | `t*x` | `?` |
  | `f_de_3` (v) | `f(x)=x^2+1` / `f(3)=` | `3*f` | `?` |
  | `f_de_x` (v) | `x=5` / `f(x)=x^2+1` / `f(x)=` | `5*f` | `?` |
  | `unidades` (v) | `m=70kg` / `g=9,8m/s^2` / `mg=` | `686,0 g⋅km/s²` | texto que empieza por `686` y acaba en `N` |
  | `sumatorio` | `\sum_{i=1}^{3} i=` (límites debajo y encima, `s12.sumrow`) | `?` | `6` |
  | `e_a_la_5` | `e^5=` | `?` | contiene `148,4` (los resultados salen con coma desde la 1.4) |
  | `cifra_pequena` | `x+30=` con el 0 al 72 % y la base alineada (`s04.num2`) | subíndice | LaTeX `x+30=` |
  | `cifra_exponente` | `x+3^0=` con el 0 al 60 % y subido 0,4 del alto | exponente | sigue siendo exponente |
  | `hueco_numeros` | `12` y `34` separados 3 altos, y `=` | `1234` | `?` |
  | `exponente_negativo` | `e^{-x}` (LaTeX) | `e^{-x}` | igual (no debe romperse) |
  | `prima` | `f'(x)=` | `?` | `?`, con LaTeX `f'(x)=` |
- **Aceptación:** `$PY medir.py --sondas` da 0 resultados erróneos; los 88 tests anteriores siguen pasando.
- **Devuelve:** `MEDIDA` con `sondas_erroneas: 8 -> 0` y nº de sondas en verde.

### T4. Entorno

- **Rama:** `feat/entorno`. **Agente:** S-B. **Bloque A:** ninguna.
- **Leer:** `om/{pagina,evaluate,layout,glossary}.py` (ya con T3), `tests/{test_pagina,test_sondas,
  sondas_util}.py`, sección «Propuesta: módulo `entorno.py`» de la auditoría.
- **Lógica:**
  1. Estructuras (`om/entorno.py`):

     ```python
     @dataclass
     class Definicion:
         renglon: int
         nombre: str                 # "x", "x_1", "yhat", "f"
         tipo: str                   # "valor" | "cantidad" | "expr" | "funcion"
         param: str | None = None    # solo "funcion" (un parámetro)
         expr: object = None         # sympy, sin sustituir
         cantidad: object = None     # pint

     class Entorno:
         def __init__(self, definiciones): ...
         def vista(self, i, solo_encima=False): ...     # -> Vista con lo que vale en el renglón i
     class Vista:
         funciones: dict; alias: dict; valores: dict; cantidades: dict
         def resolver(self, expr): ...                   # -> (expr, cambiado)
     def definir(tokens, i, vista, units): ...           # -> Definicion | None
     ```

  2. **Analizar sin sustituir.** `_Parser(tokens, glossary, funciones=())`: `_symbol` devuelve siempre el símbolo
     (salvo las constantes del glosario del cuaderno, `pi` y `e`). El entorno de la página deja de pasarse como
     `glossary`.
  3. **Definición** (`definir`): renglón que no acaba en «=», con dos partes separadas por un único «=».
     - Lado izquierdo `letra` (con subíndice o acento, sin exponente) → nombre.
     - Lado izquierdo `letra ( letra )`, con las dos letras distintas y la primera no definida arriba como valor
       → función de un parámetro. (SUPUESTO: no hay funciones de varios parámetros; la coma ya separa expresiones.)
     - Lado derecho: número con unidad → `cantidad` (lógica actual). Si no, se analiza sin sustituir; si falla, no
       es definición. Con `L` = nombres libres menos el parámetro: `nombre ∈ L` → no es definición (es una
       ecuación); función → `funcion`; `L` vacío y es un número → `valor`; si no → `expr`.
  4. **Qué vale en el renglón `i`** (`vista`): la última definición de cada nombre por encima de `i`; si no hay
     ninguna encima, la primera de la página (regla actual). Con `solo_encima=True` no hay ese segundo caso.
  5. **`resolver(expr)`**, en este orden:
     1. Ligadas: `Lg` = variables de integración, de derivada, índices de ∑ y ∏ y variable de límite de `expr`.
     2. Llamadas y alias, hasta 8 vueltas o hasta que nada cambie: cada `f(a)` con `f` función se cambia por
        `cuerpo.subs(param, a)`; cada `f__dn(a)` (prima), por `diff(cuerpo, param, n).subs(param, a)`; cada alias
        `y ∉ Lg` se sustituye por su expresión (`expr.subs`). Si a las 8 vueltas quedan alias (ciclo), se dejan.
     3. Proteger: en cada nodo `Integral`, `Sum`, `Product`, `Derivative`, `Limit`, sus variables ligadas se
        cambian por `Dummy` (`nodo.xreplace`), de dentro afuera; se guarda el mapa inverso.
     4. Valores: `expr.xreplace({Symbol(n): v})` para cada valor de la vista.
     5. Deshacer el paso 3. `cambiado` = se aplicó algo en 2 o en 4.

     *Por qué este orden.* Un alias (`y = x²`) es una identidad: debe verse dentro de la derivada para que `dy/dx`
     sea `2x`, así que va antes de proteger. Un valor (`x = 5`) no puede entrar donde `x` es la variable de la
     operación (`∫2x dx` no es `∫10 d5`): por eso se protege antes de sustituir valores.
  6. **`evaluate.answer(tree, glossary=None, units=None, vista=None)`**: tras analizar, `expr, cambiado =
     vista.resolver(expr)`. Si `cambiado` y quedan variables libres, el resultado se muestra siempre (se salta la
     condición «solo si es más corto»). Las fichas `deriv` con función y las primas dejan de lanzar `ParseError`:
     `f` conocida → `Derivative(Symbol(f), …)` o `Function(f"{f}__d{n}")(arg)`.
  7. **`pagina.page`**:
     - Las definiciones de tipo `expr` cuyo valor, resuelto con la vista de su renglón, es un número muestran la
       marca `variable` «= valor».
     - `x+y=` con `x = 5` muestra `y + 5`.
     - Revisión de pasos: `_check_row` del renglón `i` usa `entorno.vista(i − 1, solo_encima=True)`.

       *Por qué.* Al comparar el renglón `i` con el `i−1` solo valen como datos las definiciones anteriores a los
       dos. Una definición que está en el `i−1` o más abajo es la conclusión del desarrollo (`x = 7`), no un dato:
       si se sustituye, cualquier paso «cuadra».
  8. **Caché por renglón** en `pagina.py`: diccionario de módulo (máx. 512 entradas, se vacía al llenarse) con
     clave `(fichas del renglón como texto, opciones, valores que usa como texto)`.
- **Ediciones:** nuevo `om/entorno.py`; `om/evaluate.py`, `om/pagina.py`; nuevo `tests/test_entorno.py`;
  `tests/test_sondas.py`.
- **Tests** (todas con «Variables»):

  | Sonda | Renglones | Esperado |
  |-------|-----------|----------|
  | `f_de_3` | `f(x)=x^2+1` / `f(3)=` | `10` |
  | `f_de_x` | `x=5` / `f(x)=x^2+1` / `f(x)=` | `26` |
  | `f_sin_x` | `f(x)=x^2+1` / `f(x)=` | `x**2 + 1` |
  | `dxdt_frac`, `dxdt_linea` | `x=t^2` / `\frac{dx}{dt}=` y `dx/dt=` | `2*t` |
  | `dydx_frac` | `y=x^2` / `\frac{dy}{dx}=` | `2*x` |
  | `prima` | `f(x)=x^2` / `f'(x)=` | `2*x` |
  | `integral_con_x` | `x=5` / `\int 2xdx=` | `x**2 + C` |
  | `integral_definida_con_x` | `x=5` / `\int_0^1 x^2dx=` | `1/3` |
  | `derivada_con_x` | `x=5` / `\frac{d}{dx}(x^2)=` | `2*x` |
  | `sumatorio_con_i` | `i=2` / `\sum_{i=1}^{3} i=` | `6` |
  | `definicion_ensena` | `x=5` / `y=2x+3` | Renglón 1: marca `variable`, `= 13` |
  | `entorno_parcial` | `x=5` / `x+y=` | `y + 5` |
  | `pasos_mal` (con «Revisar pasos») | `2x+4=10` / `2x=14` / `x=7` / `x=3` | Marca `error` en los renglones 1 y 3; ninguna en el 2 |
  | `pasos_bien` (con «Revisar pasos») | `a=2` / `b=3` / `a+b=5` | Ninguna marca `error` |
  | `ciclo` | `x=y+1` / `y=x+1` / `x=` | No se cuelga; sin resultado numérico |
  | `rendimiento` | Los 40 renglones de `s07.mix` | Media de 5 llamadas `< 0,2 s` |
- **Aceptación:** todas las sondas en verde; `$PY medir.py --sondas` da 0 erróneos; página de 40 renglones en menos
  de 0,2 s.
- **Devuelve:** `MEDIDA` con sondas en verde / total y tiempo de la página.

### T5. Corte: rayas primero

- **Rama:** `feat/corte-rayas`. **Agente:** S-A. **Bloque A:** ninguna.
- **Leer:** `om/{segment,dots,recognizer,layout,synth,classifier}.py`, `medir.py`,
  `tests/{test_segment,test_dots}.py`; §6 M5.
- **Lógica:**
  1. `SegmentParams` gana (todos `float`, porque `load` convierte a `float`): `bars_first = 0.0`, `bar_ratio =
     1.5`, `bar_flat = 0.25`, `stack_margin = 0.35`. `_close` usa `stack_margin` en lugar del `0.35` fijo y aplica
     M5.3 cuando `bars_first > 0`.
  2. `git mv om/dots.py om/reparar.py`; `fix_dotted_letters` se conserva; nueva `reparar(syms, strokes,
     classifier, typical, params)` según M5.4, que la llama al final. `tests/test_dots.py` importa del módulo
     nuevo.
  3. `layout.bar_role` (M5.5), usada en `_parse_region` (pasos 1 y 2, sin cambiar su resultado) y en
     `recognizer.recognize` antes del contexto.
  4. `elegir_corte.py` (M5.2): escribe la tabla de los 16 casos y el ganador en
     `$ONEPEPE_DATA/math/eleccion_corte.json`.
  5. Poner los ganadores como valores por defecto de `SegmentParams`. Si ningún caso mejora al actual (M0.6),
     dejar los valores actuales y decirlo.
  6. El modelo de pares (N1b) no se hace aquí: queda como T5b (ver «Supuestos», §10).
- **Ediciones:** `om/segment.py`, `om/reparar.py` (antes `dots.py`), `om/recognizer.py`, `om/layout.py`, nuevo
  `elegir_corte.py`, `tests/test_segment.py`, `tests/test_dots.py`, nuevo `tests/test_reparar.py`.
- **Tests** (clasificador falso de `test_segment.py`, ampliado con «\frac» para la raya larga):

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_raya_no_traga_numerador` | «1» encima de una raya de ancho `3·típico`, a `0,2·típico` | `_close` falso con `bars_first = 1`; verdadero con 0 |
  | `test_igual_sigue_junto` | Dos rayas iguales de ancho `2·típico`, a `0,3·típico` | `_close` verdadero |
  | `test_dividido` | Raya con un punto encima y otro debajo | `_close` verdadero con cada punto |
  | `test_cruce` | Raya larga y trazo vertical que la atraviesa por su mitad | `_close` verdadero |
  | `test_roce_no_basta` | Trazo vertical cuyo extremo baja `0,05·h` por debajo de la raya | `_close` falso |
  | `test_repara_igual_partido` | Dos «-» apilados no seguidos en el tiempo | Un solo «=» |
  | `test_no_repara_sin_ganancia` | «-» junto a «1» con ganancia `< 0,5` | Sin cambios |
  | `test_papel_raya` | Raya con símbolos encima y debajo; raya sola | `frac`; `minus` |
- **Aceptación** (plan; rayas en la medida cruzada por tener `n = 34`): raya de fracción con corte exacto `≥ 85 %`;
  «=» `≥ 92 %`; juntados `≤ 4 %`; partidos no suben.
  `$PY medir.py --cruzada --salida t5.json && $PY medir.py --comparar t2.json t5.json`.
- **Devuelve:** `MEDIDA` con `raya_corte`, corte exacto de «=», `juntado`, `partido`, y la variante y `λ` elegidos.

### T6. Léxico

- **Rama:** `feat/lexico`. **Agente:** S-A. **Bloque A:** ninguna.
- **Leer:** `om/{layout,evaluate,recognizer,context,symbols,entorno,pagina}.py`, `tests/{test_sondas,
  test_evaluate,test_funciones}.py`.
- **Lógica** (`om/lexico.py`; no importa `layout` al cargar, para evitar el ciclo):
  1. **Números** — `unir_numeros(syms, típico, crear)`, llamada desde `layout._parse_row` antes de los guiones.
     Dos cifras seguidas son del mismo número si `|fondo_b − fondo_a| ≤ 0,2·max(h_a, h_b)` y
     `0 ≤ hueco < 0,8·típico`. Una coma pequeña (`alto < 0,4·típico`, a ras de la base) entre dos cifras
     con huecos `< 0,6·típico` es el separador decimal. El punto no: desde la 1.4 multiplica (`3.4` = 12), así que
     corta el número y sigue siendo un símbolo aparte. Resultado: `Sym` con `node = ("num", "12.5")` y la caja
     unión. `evaluate._tokens` convierte `("num", texto)` en la ficha `num`. El nodo `\,` de T3 se sigue
     insertando entre números separados.
  2. **Corrección dentro de un número** — `corregir_entidades(cand_lists, boxes, típico, fijos)`, llamada desde
     `recognizer.recognize` después del contexto y antes de decidir la sustitución. Parecidos:
     `S→5, O→0, o→0, I→1, |→1, l→1, q→9, g→9, b→6, B→8, Z→2, z→2, s→5`. Para un símbolo con primera candidata `L`
     (probabilidad `p_L`) y cifra parecida `d` (probabilidad `p_d`, 0 si no está entre las candidatas):

     ```text
     s = ln((p_d + 0,02) / p_L) + 2,0·n_cifras + 1,0·geom + π_L − θ        se cambia a d si s > 0
     ```

     - `n_cifras ∈ {0, 1, 2}`: vecinos inmediatos (misma base, hueco `< 0,8·típico`) cuya primera candidata es una
       cifra con `p ≥ 0,5` (los fijos cuentan).
     - `geom = 1` si su alto es `≥ 0,8` del alto de esos vecinos y su fondo dista `≤ 0,2` de ese alto; si no, 0.
     - `π_L = 1,5` para `S, O, o, I, |, q, B, Z` (raras como variable pegada a un número); 0 para `l, s, z, g, b`.
     - `θ = 2,5`. Se ajusta en la rejilla `{2,0; 2,5; 3,0}` con la medida cruzada: gana el mayor `numeros_2mas`
       sin que baje el acierto de las letras (M0.6).

     *Por qué esta forma.* Es el logaritmo de la razón de probabilidades entre «es una cifra» y «es una letra»: el
     primer término es lo que dice la red; cada uno de los otros suma la evidencia de un hecho independiente
     (vecinos que son cifras, tamaño de cifra, letra improbable ahí); `θ` es la desconfianza de partida. Ejemplos:
     «1S3» con `p_S = 0,6`, `p_5 = 0,2` → `s = −1,0 + 4 + 1 + 1,5 − 2,5 = +3,0` (cambia); «2b» con `p_b = 0,8`,
     `p_6 = 0,05` → `s = −2,4 + 2 + 1 + 0 − 2,5 = −1,9` (no cambia: «2b» es dos por b).
  3. **Derivadas:** `_leibniz` se mueve de `evaluate.py` a `lexico.py` sin cambios (`evaluate` la importa).
     Se añade el caso `d^n f / d v^n` en línea si aparece en las sondas.
  4. **Unidades:** sin cambios respecto a T3 (SUPUESTO: basta para las sondas).
- **Ediciones:** nuevo `om/lexico.py`; `om/layout.py`, `om/evaluate.py`, `om/recognizer.py`; nuevo
  `tests/test_lexico.py`; `tests/test_sondas.py`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_numero_con_cifra_pequena` | «3» y «0» al 75 %, base alineada | Un nodo `("num", "30")` |
  | `test_hueco_separa` | «12» y «34» con hueco `1,0·típico` | Dos nodos `num` y `\,` en medio |
  | `test_decimal` | `3 , 5` | `("num", "3.5")` |
  | `test_punto_multiplica` | `3 . 4 =` | Dos nodos `num` y el punto en medio; resultado `12` |
  | `test_exponente_no_es_numero` | `2^{10}` | Base `2`, exponente `("num", "10")` |
  | `test_corrige_S_entre_cifras` | «1», «S» (`0,6`; «5» `0,2`), «3» | Primera candidata «5» |
  | `test_no_corrige_2b` | «2», «b» (`0,8`; «6» `0,05`) | Sigue «b» |
  | `test_no_toca_fijos` | Un símbolo fijo «S» entre cifras | No cambia |
  | Sonda `segunda_derivada_y` (con «Variables») | `y=x^3` / `\frac{d^2y}{dx^2}=` | `6*x` |
- **Aceptación** (plan): `numeros_2mas ≥ 90 %` en la medida cruzada; sondas `dx/dt`, `dy/dx` y `d²y/dx²` en verde.
- **Devuelve:** `MEDIDA` con `numeros_2mas`, `corte_y_etiqueta` y `θ` elegido.

### T7. Clasificador v2

- **Ramas:** `feat/extraer-simbolos` (T7a) y `feat/clasificador-v2` (T7b). **Agente:** S-B (T7a), S-A (T7b).
- **Bloque A:** A3 = no → sin CROHME. A3 = sí → S-B busca la página oficial de CROHME, comprueba la licencia,
  escribe `descargar_crohme.sh` y añade el origen; si en 10 minutos no lo consigue, lo salta y lo apunta. A6 = no →
  sin red convolucional (paso 6).
- **Leer:** T7a: `om/{alinear,inkml,entreno,symbols,classifier}.py`, `train_classifier.py`. T7b:
  `om/{features,classifier,segment,personal,recognizer,reparar}.py`, `train_classifier.py`, `reentrenar.py`,
  `medir.py`, `tests/{test_segment,test_personal}.py`; §6 M7.
- **Lógica:**
  1. **T7a:** `extraer_simbolos.py` según M7.7, con el modelo de T2. Guarda el progreso cada 1 000 fórmulas y se
     puede reanudar. Se lanza en segundo plano; el agente devuelve cuando acaba la primera ronda.
  2. **T7b, rasgos:** `om/features.py` según M7.2–M7.5. `symbol_features_v1` = el código actual sin tocar.
  3. **Contexto:** `segment.segment` calcula `linea` una vez (con los trazos y con las cajas de `known`, que recibe
     por un parámetro nuevo `known_boxes=()`) y la pasa a `candidates_w_junk(strokes, k, linea=linea)` solo si
     `getattr(classifier, "usa_linea", False)`. `SymbolClassifier.usa_linea = (feature_version >= 2)`.
     `probabilities`, `embedding` y `probabilities_and_embedding` aceptan `linea=None`. `recognizer` y `reparar`
     la propagan.
  4. **Entrenamiento:** `train_classifier.py --rasgos 2 --extraidos RUTA` añade el origen «extraído» (tope 400 por
     clase, 10 copias deformadas por muestra) y el contexto (M7.5). Receta de M2 sin cambios.
  5. **kNN:** rejilla de M7.6; los ganadores pasan a ser las constantes de `personal.py`.
  6. **Solo con A6 = sí:** red convolucional pequeña (entrada 16×16 con 9 canales: rejilla más 8 orientaciones;
     dos capas 3×3 de 16 y 32 filtros con reducción 2×2; capa densa de 128), entrenada con PyTorch e inferida en
     numpy desde el `.npz`. Se queda solo si mejora el top-1 (M0.6) y cumple el tiempo.
  7. **Curva de aprendizaje:** entrenar con el 25, 50, 75 y 100 % de las expresiones propias de entrenamiento y
     apuntar el top-1 de validación de cada uno.
  8. Pliegues nuevos con `entrenar_pliegues.py --receta v2`, en `$ONEPEPE_DATA/pliegues/v2/`. Después,
     recalibrado (§5).
- **Ediciones:** nuevo `extraer_simbolos.py`; `om/features.py`, `om/classifier.py`, `om/segment.py`,
  `om/personal.py`, `om/recognizer.py`, `om/reparar.py`, `train_classifier.py`; nuevo `tests/test_features.py`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_v1_intacto` | Un símbolo fijo | `symbol_features_v1` da el mismo vector que antes del cambio (valores guardados en el test) |
  | `test_tamano_vector` | Cualquier símbolo | 556 valores |
  | `test_orden_no_importa` | Dos trazos en los dos órdenes | Bloque de direcciones idéntico (`atol 1e-6`) |
  | `test_sentido_no_importa` | Un trazo y el mismo al revés | Bloque de direcciones idéntico |
  | `test_continuidad` | Un trazo recto a 22,4° y el mismo a 22,6° | Diferencia máxima del bloque `< 0,1` (sin reparto entre cubos sería `≈ 0,7`) |
  | `test_contexto_distingue_tamano` | La misma forma con alto `0,6·típico` y `1,0·típico` | `c2` distinto; el resto del vector igual |
  | `test_sin_contexto` | `linea=None` | Los 6 valores de contexto son 0 |
  | `test_modelo_viejo_funciona` | `.npz` sin `feature_version` | Se evalúa con v1 y no falla |
  | `test_linea_base_robusta` | Fondos alineados más uno muy bajo | `corte` a menos de `0,05·típico` de los alineados |
- **Aceptación** (plan; red sola, corte bueno): top-1 `≥ 80 %` y top-5 `≥ 95 %` en el examen y en la medida
  cruzada; `ms_por_renglon < 30`. `$PY medir.py --sin-knn --salida t7.json` y lo mismo con `--cruzada`.
- **Devuelve:** `MEDIDA` con `red_top1`, `red_top5`, `corte_y_etiqueta`, `ms_por_renglon`, símbolos extraídos y la
  curva de aprendizaje en una línea.

### T8. Confianza

- **Rama:** `feat/confianza`. **Agente:** S-A, en el árbol principal (toca Rust). **Bloque A:** ninguna.
- **Leer:** `om/{recognizer,personal,pagina,server,symbols}.py`, `medir.py`; `crates/rnote-engine/src/engine/
  math.rs` líneas 60–110 (`MathSymbol`, `MathOverlay`), 861–940 (`math_apply_response`), 1038–1180 (archivo
  oculto), 1285–1420 (`math_lines_json`, `math_apply_page`) y 1708–1750 (`draw_math_overlays`); §6 M8.
- **Lógica:**
  1. `recognizer.recognize`: guardar `y_red` (primera candidata de `segment`, antes de `personal.adjust`) y
     `y_knn` (de `personal.neighbors`). `clean` solo se calcula si se cumple M8.2. `umbral = max(threshold,
     confianza.json["umbral"])`. Cada símbolo devuelve además `"sure": bool` (cumple M8.2).
  2. `calibrar_umbral.py` (M8.3) → `confianza.json`.
  3. Rust: `MathSymbol` gana `confidence: f64` (de la respuesta; 1.0 en las correcciones a mano; en el archivo
     oculto, campo con `#[serde(default = …)]` a 1.0: el `.rnote` no cambia). `KnownSymbol` gana `p: f64` y
     `math_lines_json` lo envía.
  4. `pagina.page`: cada marca lleva `"sure"` = `mín p de los símbolos del renglón ≥ umbral_renglon`; sin `p` en
     la petición, `true`.
  5. Rust: `Mark` y `MathOverlay` ganan `sure: bool` (por defecto `true`). `draw_math_overlays`: si no es seguro,
     el texto lleva « ?» al final y la opacidad baja de 150 a 85 (de 170 a 100 en `variable`).
- **Ediciones:** `om/recognizer.py`, `om/pagina.py`, nuevo `calibrar_umbral.py`, `math.rs`,
  `tests/test_pagina.py`, nuevo `tests/test_confianza.py`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_wilson` | `CS(0, 100)`; `CS(0, 300)`; `CS(5, 100)` | `0,0162`; `0,0054`; `0,0859` (±0,0005) |
  | `test_no_sustituye_si_knn_contradice` | Red «x» 0,99; vecino cercano «y» | `clean` nulo, `sure` falso |
  | `test_sustituye_misma_forma` | Red «x», final «X» | Se sustituye (`same_shape`) |
  | `test_umbral_es_el_mayor` | App 0,95; `confianza.json` 0,98; `c = 0,96` | No se sustituye |
  | `test_marca_dudosa` | Renglón con un símbolo `p = 0,4` y `umbral_renglon = 0,7` | Marca con `sure` falso |
  | `test_sin_p_es_seguro` | Petición sin `p` | `sure` verdadero |
  | Rust `sure_defaults_true` | Respuesta `page` sin `sure` | El `MathOverlay` tiene `sure == true` |
  | Rust `confidence_survives_sidecar` | Guardar y cargar el archivo oculto | Misma `confidence`; archivo viejo sin el campo → 1.0 |
- **Aceptación** (plan): `sust_erroneas ≤ 2 %` con `sust_cobertura ≥ 60 %` en la medida cruzada y en el examen.
  `cargo test -p rnote-engine math` y `cargo check -p rnote` en verde.
- **Devuelve:** `MEDIDA` con `sust_erroneas`, `sust_cobertura`, `τ*`, `τ_r` y la cota de Wilson.

### T9. Renglón vivo

- **Rama:** `feat/renglon-vivo`. **Agente:** S-A, árbol principal.
- **Bloque A:** A5 sin páginas reales → **saltada**, pendiente en §9. Con páginas: paso 0; el resto solo si la
  medida supera el 3 %.
- **Leer:** `rnote_extract.py`, `om/{server,recognizer,segment}.py`; `math.rs` líneas 140–180 (estado), 548–610
  (`math_take_request`), 861–940 (`math_apply_response`), 1038–1180 (archivo oculto: `symbols[].original`,
  `times`); `crates/rnote-ui/src/mathhelper.rs` líneas 1–60 y 170–245.
- **Lógica:**
  1. **Paso 0:** `reproducir_pagina.py PAGINA.rnote`: lee el archivo oculto (`symbols` con `original`, `times` por
     trazo), ordena los trazos por tiempo, forma lotes cortando en pausas `> 0,8 s` (y 1,2 y 1,5) y cuenta los
     símbolos de 2 o más trazos repartidos en lotes distintos. Salida: tabla como la de la auditoría.
  2. **Regla:** si con 0,8 s son `≤ 3 %`, la tarea acaba aquí (se apunta la medida). Si con 1,2 s bajan a `≤ 3 %`,
     solo se hace el paso 3. Si no, pasos 3 a 6.
  3. **Retardo configurable:** `RECOGNITION_DELAY` pasa a ser un ajuste de la app (`math_delay_ms`, de 800 a 1500;
     por defecto el menor valor medido con `≤ 3 %`).
  4. **Protocolo.** La petición `recognize` gana `"reopen": [{"id", "strokes", "times", "latex"}]`: los símbolos
     no bloqueados del renglón, con `original` no vacío, reconocidos hace menos de 10 s, que tocan un trazo nuevo
     (hueco `< 0,6·tamaño`) o son el último del renglón. No van en `known`. El auxiliar corta juntos los trazos
     reabiertos y los nuevos (por tiempo). Cada símbolo de la respuesta lleva `"from": [ids reabiertos que usa]`
     y, si es idéntico a uno reabierto (mismos trazos y etiqueta), `"keep": id`.
  5. **Aplicar** (`math_apply_response`): `keep` → no se toca nada (conserva id y trazos). Si no: se quitan los
     símbolos de `from` (sus trazos actuales) y se crea el nuevo; su `original` es la unión de los puntos. Todo en
     un único `record()`: un solo deshacer.
  6. **Bloqueo y cola:** `MathSymbol` gana `locked: bool` (verdadero al corregir a mano; nunca se reabre) y
     `pending_clean: Option<…>`. El último símbolo del renglón no se sustituye por el limpio hasta que hay otro a
     su derecha o pasan 2,5 s (`TAIL_DELAY` en `mathhelper.rs`).
- **Ediciones:** nuevo `reproducir_pagina.py`; `math.rs`, `mathhelper.rs`, `om/server.py`, `om/recognizer.py`;
  `tests/test_server.py`; `es.po` (H-I) si hay textos.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_lotes_por_pausa` | Tiempos con una pausa de 1 s dentro de un «=» | 1 símbolo partido con 0,8; 0 con 1,2 |
  | `test_reopen_une` | `reopen` con la primera raya de un «=» y la segunda como trazo nuevo | Un «=» con `from = [id]` |
  | `test_reopen_keep` | `reopen` con una «x» y un trazo nuevo lejos | La «x» vuelve con `keep` |
  | Rust `reopen_replaces_symbol` | Respuesta con `from` | El símbolo viejo desaparece; hay uno nuevo con los puntos de los dos |
  | Rust `locked_is_not_reopened` | Símbolo corregido a mano | No aparece en `reopen` |
  | Rust `reopen_is_one_undo` | Reabrir y deshacer | Un solo paso vuelve al estado anterior |
- **Aceptación** (plan): `medir.py --incremental 0.8` igual a «todo de una vez» (±0,5 puntos) en la medida cruzada;
  tests de Rust en verde.
- **Devuelve:** `MEDIDA` con el % de símbolos partidos por lotes (0,8; 1,2; 1,5) y la diferencia incremental.

### T10. Las N mejores

- **Rama:** `feat/n-mejores`. **Agente:** S-A. **Bloque A:** ninguna.
- **Leer:** `om/{recognizer,context,layout,evaluate,lexico,symbols}.py`, `tests/{test_server,test_evaluate}.py`;
  §6 M10.
- **Lógica:**
  1. `layout.parse_with_order(syms)` (M10.3): cada `Sym` creado en `_parse_region` y `_parse_row` lleva `members`,
     la lista de índices originales en orden LaTeX. `parse(syms)` sigue devolviendo solo el árbol.
  2. `context.reading_order(symbols, boxes)`: usa `parse_with_order` con las primeras candidatas; si falla, el
     orden actual.
  3. `om/haz.py`: `gramatica(tree) -> float` (la `G` de M10.1) y `mejor_lectura(cand_lists, boxes, fijos) ->
     [índice de candidata por símbolo]` (M10.2).
  4. `recognizer.recognize`: tras el contexto y el léxico, `mejor_lectura`; la candidata elegida pasa a ser la
     primera. Antes de la decisión de sustitución de T8.
  5. Pesos de `G`: rejilla del premio `{2,0; 3,0; 4,0}` con la medida cruzada; gana el mayor `renglon_perfecto`.
- **Ediciones:** nuevo `om/haz.py`; `om/recognizer.py`, `om/context.py`, `om/layout.py`; nuevo
  `tests/test_haz.py`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_orden_del_arbol` | `\frac{a}{b}` con cajas | Orden: raya, a, b |
  | `test_parentesis` | `(x+1` seguido de un símbolo con candidatas `l` 0,5 y `)` 0,4 | Elige `)` |
  | `test_no_cambia_si_ya_vale` | `2+3=` con segundas candidatas cercanas | Sin cambios |
  | `test_margen_grande_no_se_abre` | Símbolo con 0,99 / 0,001 | No se prueba su segunda candidata |
  | `test_de_mejor_a_peor` | 3 símbolos con 2 candidatas | Los costes salen en orden no decreciente y sin repetidos |
  | `test_fijos_no_se_abren` | Símbolo fijo dudoso | No cambia |
  | `test_histeresis` | Mejora de `S` de 0,3 | No cambia |
- **Aceptación** (plan): `renglon_perfecto` sube (M0.6) en la medida cruzada y `ms_por_renglon ≤ 50`.
- **Devuelve:** `MEDIDA` con `renglon_perfecto`, `latex_exacto`, `ms_por_renglon` y el premio elegido.

### T11. Captura de fallos

- **Rama:** `feat/captura-fallos`. **Agente:** S-C, árbol principal; textos y `es.po`: H-I. **Bloque A:** ninguna.
- **Leer:** `om/{server,personal}.py`, `medir.py`, `tests/test_server.py`; `mathhelper.rs` líneas 233–245 (`ask`),
  446–700 (`correct_symbol`, `other_symbol_box`, `explain_overlay`) y 781–925 (`show_candidates`); `math.rs`
  líneas 1430–1450 (`math_overlay_at`) y 524–547 (`math_symbol_at`); `crates/rnote-ui/po/es.po` (solo el final);
  CONTEXTO-IA.md §3.4 (textos).
- **Lógica:**
  1. Orden nueva `{"cmd": "report_failure", "strokes", "times", "expected", "read", "notebook_dir"}`: añade una
     línea a `$ONEPEPE_DATA/math/fallos.jsonl` con `id` (12 hex), `date`, `expected`, `read`, `strokes`, `times`.
     Responde `{"ok": true, "id"}`. Si `expected` está vacío: `{"error": …}`.
  2. Motor: `math_failure_request(pos, expected) -> Option<String>`: reúne los trazos originales y los tiempos de
     los símbolos del renglón que hay en `pos` y el LaTeX leído.
  3. Interfaz: botón «This is wrong…» en la ventanita de candidatas (`show_candidates`) y en la del «?»
     (`explain_overlay`). Abre un cuadro con una entrada de texto que trae el LaTeX leído; al aceptar, envía la
     orden y muestra el aviso «Saved for training». Textos en inglés con `gettext`; traducción en `es.po`.
  4. `medir.py --fallos` ya existe desde T0: aquí solo se comprueba con un test que lee lo que guarda la orden.
  5. Pantalla de ejemplos (lista con borrar y cambiar etiqueta sobre las órdenes `examples`, `forget`, `relabel`):
     **fuera de esta ejecución**; queda en §9 (SUPUESTO: es interfaz nueva que nadie puede revisar aquí).
- **Ediciones:** `om/server.py`, `math.rs`, `mathhelper.rs`, `es.po`, `tests/test_server.py`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `test_report_failure_guarda` | Orden con 2 trazos y `expected = "x+1"` | Una línea en `fallos.jsonl` con esos campos; `ok` |
  | `test_report_failure_vacio` | `expected = ""` | `error`; no escribe |
  | `test_medir_lee_fallos` | `fallos.jsonl` con una expresión | `medir.py --fallos` cuenta 1 renglón |
  | Rust `failure_request_has_strokes` | Renglón con dos símbolos reconocidos | La petición lleva sus trazos originales y `read` |
- **Aceptación** (plan): tests del servidor; `cargo check -p rnote`; `msgfmt -c` sin errores; un fallo guardado
  aparece en la medida. La interfaz, en §9.
- **Devuelve:** `MEDIDA` con «no aplica»; `TESTS` completos.

### T12. Diseño fluido

- **Rama:** `feat/diseno-fluido`. **Agente:** S-A, árbol principal; `es.po`: H-I. **Bloque A:** A7 = lote seguro.
  **En espera por decisión del usuario: es el último paso (O10), después de T13.**
- **Leer:** sección «Diseño fluido» del plan; `crates/rnote-ui/src/theme.rs`, `crates/rnote-ui/data/style.css`,
  `crates/rnote-ui/src/ribbon.rs` (solo donde se crea el `Stack` de pestañas), `crates/rnote-ui/src/canvas/mod.rs`
  (solo el `tick callback` y el borde del modo matemáticas), `math.rs` 861–940 y 1708–1750.
- **Lógica** (lote seguro; números = propuestas del plan):
  1. **(1) Tabla de movimiento** en `theme.rs`: `MOTION_FAST = 100`, `MOTION_NORMAL = 160`, `MOTION_SLOW = 240`
     (ms); curvas `cubic-bezier(0.2, 0, 0, 1)` (entrada) y `cubic-bezier(0.4, 0, 1, 1)` (salida), definidas donde
     `theme.rs` genera el CSS. Todas las `transition` de `style.css` pasan a usar esos valores. Test que falla si
     queda una duración suelta, como el de colores.
  2. **Reducir animaciones:** función única `animations_enabled()` (`gtk4::Settings::default()` →
     `is_gtk_enable_animations()`); todo lo de abajo no anima si es falso.
  3. **(9) Cinta:** el `Stack` de pestañas con `StackTransitionType::Crossfade` y 150 ms.
  4. **(12) Borde del modo matemáticas:** entra con fundido de 200 ms y 1 px menos de grosor.
  5. **(4) Resultados:** `MathOverlay` gana `born: Option<Instant>`. Función pura
     `overlay_alpha(ms_desde_que_nació) -> f64 = min(ms/120, 1)` con curva de entrada, y desplazamiento
     `4·(1 − alpha)` px. Mientras algún resultado tenga menos de 120 ms, el motor pide redibujar
     (`WidgetFlags.redraw`) en el `tick callback` que ya existe.
  6. **(2) Sustitución:** misma técnica: los trazos limpios recién insertados se dibujan con `alpha` creciente
     durante 160 ms. La tinta original no se conserva en pantalla (SUPUESTO: fundido de entrada solo; el fundido
     cruzado exige dibujar trazos ya borrados y queda en §9).
  7. Con A7 = «todo»: las propuestas 3, 5, 6, 7, 8, 10 y 11, cada una en un commit y con el mismo interruptor.
- **Ediciones:** `theme.rs`, `style.css`, `ribbon.rs`, `canvas/mod.rs`, `math.rs`.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | `theme::no_loose_durations` | `style.css` | Ninguna duración que no sea 100, 160 o 240 ms |
  | `math::overlay_alpha_curve` | 0, 60, 120, 500 ms | 0; entre 0 y 1; 1; 1 |
  | `math::no_animation_when_disabled` | Animaciones apagadas | `overlay_alpha` da 1 desde el principio |
- **Aceptación** (plan): `cargo check -p rnote` y `cargo test -p rnote theme` en verde; con «reducir animaciones»
  no se anima nada. El aspecto, en §9.
- **Devuelve:** `MEDIDA` con «no aplica»; lista de propuestas hechas y no hechas.

### T13. Cierre

- **Rama:** `feat/cierre`. **Agente:** S-A (valores por defecto y `promover_cierre.py`); H-I (todo lo demás).
- **Bloque A:** A4 (valores por defecto), A1 (promoción), A2 (metas).
- **Leer:** S-A: `math.rs` líneas 27–65 y sus tests de opciones. H-I: §8 y §9 de este documento, `ESTADO.md`,
  `VERSIONES.md` (cabecera), `CAMBIOS-ONEPEPE.md` (índice), `PLAN-PENDIENTE.md` (índice), `CONTEXTO-IA.md` §7 y §11.
- **Lógica:**
  1. Medida final (H-I): `medir.py` en examen y `--cruzada`, en modo normal y con `--incremental 0.8`; `--sondas`.
  2. Con A4 = sí **y** `sondas_erroneas = 0`: `MathOptions::default` con `equals: true`, `variables: true`
     (`check` sigue en `false`). Si las sondas no dan 0: no se cambia y pasa a §9.
  3. `promover_cierre.py [--aplicar]` (S-A): comprueba que los originales tienen la suma de `origen.sha256`
     (si no, para: el usuario ha usado la app mientras tanto → fusiona antes las correcciones nuevas, que son las
     líneas del original que no están en la copia); copia cada archivo real a `.bak` con fecha; instala
     `correcciones.jsonl`, `simbolos.npz` (el anterior pasa a `simbolos.anterior.npz`), `examen.json`,
     `examen_trazos.json`, `referencia.json`, `confianza.json`. Con A1 = sí, H-I lo ejecuta con `--aplicar`.
  4. Documentos (H-I): lista de pruebas con lápiz en `CAMBIOS-ONEPEPE.md` (la tabla B1 de §9); `ESTADO.md`;
     `PLAN-PENDIENTE.md` (lo saltado y lo bloqueado); `CONTEXTO-IA.md` §7 (módulos nuevos: `alinear`, `entorno`,
     `lexico`, `reparar`, `haz`; órdenes y campos nuevos); `VERSIONES.md`; `INFORME-CIERRE.md` (§8).
  5. No se borra ningún documento viejo (SUPUESTO: «limpiar documentos viejos» del plan se limita a marcar en
     `ESTADO.md` cuáles están superados).
- **Ediciones:** `math.rs`, nuevo `promover_cierre.py`, documentos.
- **Tests:**

  | Test | Entrada | Esperado |
  |------|---------|----------|
  | Rust `math_options_default` | `MathOptions::default()` | `equals` y `variables` verdaderos; `check` falso; `threshold` 0.95 |
  | `test_promover_copia` | Carpetas temporales «real» y «trabajo» | Existen los `.bak`; los archivos instalados son los de trabajo |
  | `test_promover_detecta_cambios` | El original cambia tras `origen.sha256` | Sin `--aplicar` avisa; con él, las líneas nuevas del original están en el resultado |
- **Aceptación** (plan): las metas de A2 medidas (cumplidas o no, con su cifra) y la lista de pruebas con lápiz
  escrita. La prueba con lápiz la hace el usuario (§9).
- **Devuelve:** `MEDIDA` con las seis metas: hoy → final.

## 8. Informe final automático

Lo escribe H-I en `INFORME-CIERRE.md` (raíz del repositorio; sin datos del usuario) y lo fusiona con T13. El
orquestador enseña al usuario solo este informe y §9. Plantilla:

```markdown
# Informe del cierre (<fecha>)
## En cinco líneas
<qué se ha hecho, qué no, y la cifra principal antes y después>
## Tareas
| Tarea | Estado (hecha / saltada / bloqueada / apagada) | Rama | Versión | Motivo si no está hecha |
## Cifras contra las metas
| Medida | Hoy (auditoría) | Tras T2 | Final examen (n, intervalo 90 %) | Final cruzada (n) | Incremental 0,8 s | Meta | ¿Cumplida? |
## Decisiones tomadas sin preguntar
<cada «SUPUESTO:» de DECISIONES.md, una línea>
## Qué está comprobado y cómo
| Parte | Compilado | Tests | Ejecutado con tus datos | Probado con lápiz |
## Lo que queda para ti
<la lista de §9, con B4 y B6 rellenos según lo ocurrido>
```

Reglas del informe: una fila por tarea aunque esté saltada; «¿Cumplida?» usa la regla de n pequeño de §0; nada se
da por probado con lápiz.

## 9. Bloque B: cierres finales (única lista para el usuario al terminar)

**B1. Pruebas con lápiz** (modo matemáticas encendido; con «=» y «Variables» encendidos):

| Nº | Escribe | Debe salir |
|----|---------|------------|
| 1 | `12+34=` | 46, y los símbolos se sustituyen por los limpios |
| 2 | `x=5` y, debajo, `2x+3=` | 13 |
| 3 | `y=2x+3` debajo de `x=5` | «= 13» al lado, en morado |
| 4 | `f(x)=x²+1` y, debajo, `f(3)=` | 10 |
| 5 | `x=t²` y, debajo, `dx/dt=` (en fracción y en línea) | 2t las dos veces |
| 6 | `(x+1)+3=` | x + 4 |
| 7 | Una fracción `1/2 + 1/3 =` con rayas largas | 5/6; ninguna raya se junta con un número |
| 8 | `e^5=` | e⁵ ≈ 148,4 |
| 9 | `25`, `19`, `100` deprisa | Números, sin letras dentro («2S», «1q») |
| 10 | Un «=» con una pausa de más de 1 s entre sus dos rayas | Un solo «=» (T5/T9) |
| 11 | Algo con mala letra a propósito | No se sustituye; el resultado sale tenue y con «?» |
| 12 | `2x+4=10`, `2x=14` con «Revisar pasos» | Marca de error en el segundo renglón |

**B2. Revisión de la interfaz:** botón «Esto está mal» en la ventanita de candidatas y en el «?» (T11): guarda y
avisa. Animaciones (T12): cambio de pestaña, borde del modo matemáticas, aparición de resultados y de símbolos
limpios; con «reducir animaciones» del sistema, nada se mueve.

**B3. Instalar lo nuevo.** Los `push` ya los ha hecho el integrador al cerrar cada oleada (A0):

```bash
cd /home/pepe/Pepe_Flock/One_Pepe && git checkout onepepe && git pull --ff-only
cd tools/onepepe-math && ./instalar.sh && cd ../..      # programa auxiliar
~/instalar-onepepe.sh                                   # compila e instala la app
```

**B4. Solo si A1 fue «no»:** instalar el almacén y el modelo nuevos (hace copia antes):

```bash
cd /home/pepe/Pepe_Flock/One_Pepe/tools/onepepe-math && ~/.local/share/onepepe/venv/bin/python promover_cierre.py --aplicar
```

**B5. Volver atrás** si algo va peor: los `.bak` con fecha están junto a cada archivo en
`~/.local/share/onepepe/math/`; el modelo anterior es `simbolos.anterior.npz`.

**B6. Pendiente que depende de ti** (el informe marca cuáles aplican):

- T9 (renglón vivo): hacen falta una o dos páginas reales de apuntes con el modo matemáticas activado.
- Terminar los dos sets de 200: hoy el examen tiene 22 expresiones; con los 400, tendrá 80.
- Dataset nuevo: ruta y formato.
- Pantalla de ejemplos (T11), fundido cruzado de la tinta (T12) y propuestas de diseño no incluidas en A7.
- Tareas bloqueadas, con su causa.
- Decidir si se borran los documentos viejos y las copias `.bak`.

## 10. Supuestos de este documento

Cada uno se copia en `DECISIONES.md` como «SUPUESTO:» cuando la tarea afectada se ejecuta.

1. **Receta de la medida cruzada.** Los modelos de pliegue usan el peso y las épocas elegidos en la validación, que
   está dentro de los pliegues: optimismo despreciable (dos números), a cambio de 941 símbolos no vistos.
2. **`raya_papel`.** La meta «raya de fracción, corte y papel correcto» se mide como en T0 (acepta `-` o `\frac`
   si el renglón da los mismos `\frac`). El 21 % de la auditoría era «corte y etiqueta `\frac`»; T0 da las dos
   cifras para que la comparación sea limpia.
3. **Sustituciones erróneas.** La meta se mide en estricto (etiqueta distinta = error). Se informa aparte de las
   «visibles» (sin parejas de la misma forma).
4. **Modelo de pares (N1b del plan).** No se construye en T5. Solo se abre como T5b, tras T7a (que da los pares de
   MathWriting), si `juntado` sigue por encima del 4 %: regresión logística sobre hueco horizontal y vertical,
   solapes, razón de tamaños, planos o no y pausa, que entra en la DP en lugar de la penalización fija.
5. **Funciones de un solo parámetro** (T4). **Unidades** sin cambios tras T3 (T6).
6. **`e` es la constante** salvo que el glosario la defina (T3). Si escribes `e` como variable, defínela.
7. **CROHME:** no se fija aquí ninguna dirección de descarga; con A3 = sí la busca el agente y, si no la encuentra,
   se salta.
8. **Versiones:** una subida de la menor por oleada fusionada; el cierre no sube la mayor.
9. **Constantes de diseño sin medir aún** (son puntos de partida razonados; cada una tiene su rejilla y su regla de
   elección en la tarea): `LEJOS = 3,0`, `UNK = 0,05` (T0); topes 80 y 40·W, 15 % de trozos (T2); umbrales de raya
   1,5 y 0,25, cruce 0,15, ganancia 0,5 (T5); pesos 2,0, 1,0, 1,5 y `θ = 2,5` (T6); borrado de contexto 30 %
   (T7); premio 3,0 y castigos 2,0 (T10).
10. **Tiempo de cálculo** estimado, sin medir: un entrenamiento, de 3 a 6 minutos; pliegues, cuatro veces eso por
    receta (dos recetas); extracción de símbolos, de 15 a 40 minutos por ronda.
11. **Nada de esto está ejecutado.** Las firmas, los números de línea y los umbrales citados salen del código leído
    el 2026-11-15 (la 1.3.5 del equipo, revisada después contra la 1.4 del remoto); los valores esperados de los tests salen de la auditoría o de cálculo a mano.
