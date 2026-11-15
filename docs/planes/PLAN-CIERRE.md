# Plan de cierre de OnePepe: matemáticas (2026-11-15)

El porqué de cada tarea: [`AUDITORIA-MATEMATICAS.md`](../auditorias/AUDITORIA-MATEMATICAS.md).

Este documento dice qué hacer, en orden, para terminar el modo matemáticas. Las medidas y sus causas están en la
auditoría. Cómo ejecutarlo con agentes: [`ARQUITECTURA-EJECUCION.md`](ARQUITECTURA-EJECUCION.md).

> **Nota (2026-11-15).** Medido y escrito sobre la versión 1.3.5 que había en el equipo. El remoto ya iba por la
> 1.4 (el punto multiplica y la coma es el decimal), que en matemáticas solo cambia `evaluate._tokens`,
> `layout._NEVER_SCRIPT` y que los resultados salen con coma. Las medidas de corte y etiqueta no dependen de eso.
> Las 8 sondas con resultado erróneo se repitieron sobre la 1.4 y siguen igual (la de unidades da ahora
> «686,0 g⋅km/s²»). Sobre la 1.4 pasan 88 tests.

## Arquitectura final

Esto es el objetivo, no lo que hay hoy. Dos partes: la aplicación en Rust y un programa auxiliar en Python.

```text
APP (Rust)                                     PROGRAMA AUXILIAR (Python, sin estado)
----------                                     -------------------------------------
lápiz -> trazos + tiempos                      orden `recognize`:
math.rs (cola; renglón vivo: reabre            1 corte (rayas primero + modelo de pares + DP)
  vecinos no fijados; aplica diferencias;      2 clasificador v2 (rasgos con tamaño relativo;
  sustitución retrasada en la cola)               kNN personal; confianza calibrada)
-> anotaciones                                 3 reparación (trazos tardíos y compuestos)
                                               4 léxico (números, funciones, diferenciales,
                                                  derivadas, unidades)
                                               5 estructura (línea base del renglón)
                                               6 N mejores + gramática

                                               orden `page`:
                                               7 entorno (valores, expresiones, funciones,
                                                  unidades; ligadas; dependencias; caché)
                                               8 evaluación SymPy con confianza y traza
```

**Datos locales** (nunca a git):

- `simbolos.npz`: con temperatura y versión de rasgos.
- `correcciones.jsonl`: con id de expresión.
- `examen.json`: congelado.
- `fallos.jsonl`.
- `corte.json`.

**Herramientas:**

- `alinear.py` (alineado ordenado).
- `medir.py` (banco de medida).
- `reimportar_entrenos.py`.
- `train_classifier.py`, sin fugas.
- `reentrenar.py`, con examen.

**Invariantes** (se mantienen siempre):

- El formato .rnote queda intacto. Todo va en archivo oculto o en ajustes.
- El motor Rust no sabe matemáticas.
- El auxiliar no guarda estado entre peticiones.
- Todo es local.
- «Si no mejora, no se queda».

## Lista priorizada

Criterio: primero lo que permite medir, luego lo que enseña resultados falsos, y luego lo que más sube el acierto por
menos esfuerzo.

| Prioridad | Tarea | Qué es | Esfuerzo | Depende de |
|-----------|-------|--------|----------|------------|
| P0 | T0 Banco de medida | Alineado ordenado y examen congelado. Sin esto no se puede aceptar nada | Bajo-medio | Nada |
| P0 | T1 Reimportar entrenos | Reimportar los entrenos con el alineado ordenado y control de calidad | Bajo | T0 |
| P0 | T2 Entrenamiento sin fugas | Reparto por expresión, deformar después, parada real, mezcla equilibrada, calibración | Medio | T0, T1 |
| P0 | T3 Evaluación correcta | Quitar los 8 resultados erróneos y los fallos de estructura (paréntesis + operador, límites de ∑, cifra pequeña, hueco entre números, prima, e) | Medio | T0 |
| P0 | T4 Entorno | Funciones, expresiones, variables ligadas, definiciones que enseñan su valor, pasos con variables | Medio | T3 |
| P1 | T5 Corte: rayas primero | Rayas primero, apilados estrictos, penalización 0–0,3, reparación general | Medio | T0 (mejor tras T2) |
| P1 | T6 Léxico | Números, corrección dentro de la entidad, diferenciales y derivadas (Leibniz, orden n, en línea), unidades | Medio | T3, T5 |
| P1 | T7 Clasificador v2 | Más datos genéricos y rasgos con tamaño relativo; comparar con una red convolucional pequeña | Alto | T0–T2 |
| P1 | T8 Confianza | Regla de sustitución con umbral calibrado y acuerdo red–kNN; resultado dudoso visible | Bajo | T2 |
| P2 | T9 Renglón vivo | Reabrir los vecinos no fijados (previa medida de pausas reales) | Alto (Rust + Python) | T5 |
| P2 | T10 N mejores | N mejores más gramática; contexto en orden del árbol | Medio | T6, T7 |
| P2 | T11 Captura de fallos | «Esto está mal» y pantalla de ejemplos | Medio | T0 |
| P3 | T12 Diseño fluido | Animaciones y transiciones (sección «Diseño fluido») | Medio (opcional) | Nada; mejor tras T8 |
| P3 | T13 Cierre | Pruebas con lápiz, opciones encendidas por defecto, documentación, versión | Bajo | Todo lo anterior |

Metas propuestas para dar el proyecto por terminado. Se miden en el examen congelado, con expresiones no vistas:

| Medida                                          | Hoy                   | Meta      |
|-------------------------------------------------|-----------------------|-----------|
| Símbolos con corte y etiqueta                   | 44–66 % (no visto)    | ≥ 90 %    |
| Renglones todos bien                            | 5–28 %                | ≥ 60 %    |
| Números de 2 o más cifras bien                  | 63–78 %               | ≥ 90 %    |
| Raya de fracción, corte y papel correcto        | 21 %                  | ≥ 90 %    |
| Sustituciones erróneas                          | 9–10,5 %              | ≤ 2 %     |
| Resultados erróneos en la batería de sondas     | 8                     | 0         |

Todo lo anterior también debe cumplirse en modo incremental (0,8 s).

Son propuestas. El usuario decide.

## Reglas comunes para los subagentes

- Leer `CLAUDE.md` y `CONTEXTO-IA.md` §3. Responder en español. Rama `feat/<nombre>` o `fix/<nombre>` desde `onepepe`.
  Fusionar con `git merge --no-ff` solo si pasan los tests. `git push origin onepepe:main` solo cuando el usuario lo
  pida. Nunca `--force` ni borrar ramas.
- No tocar el formato .rnote. Nada de modelos, datasets ni letra del usuario en git. Comprobar `git status` y
  `git ls-files` antes de cada commit.
- Los datos del usuario (`~/.local/share/onepepe`, `~/Documentos/OnePepe` entrenos) son de solo lectura, salvo en la
  tarea que lo diga, y siempre con copia previa. Para pruebas: `ONEPEPE_DATA=<carpeta temporal>`.
- Tests: `cd tools/onepepe-math && ~/.local/share/onepepe/venv/bin/python -m unittest discover -s tests` (hoy 85
  pasan, 1 saltado). Rust: `cargo test -p rnote-engine math` y `cargo check -p rnote` (ver `BUILDING.md`).
- Cada tarea: primero un test que falla, luego el arreglo, y luego medida antes y después con `medir.py`. Si no mejora,
  no se queda, y se documenta en `DECISIONES.md`. Subir la versión (`meson.build` y `app.metainfo.xml.in.in`) y
  apuntarla en `VERSIONES.md`. Estado en `ESTADO.md`. Decir siempre qué está compilado, qué tiene tests, qué se ha
  ejecutado y qué se ha probado con lápiz.
- Modelo recomendado: Sonnet para código y pruebas; Haiku para documentación, tests repetitivos y traducciones
  (`es.po`); Fable solo para diseñar T7 y T9, y para revisar fusiones delicadas.

## Hojas de ruta

Cada tarea sigue las reglas comunes de arriba.

### T0. Banco de medida

- **Rama:** `feat/banco-de-medida`
- **Modelo recomendado:** Sonnet
- **Depende de:** nada
- **Archivos:** nuevos `onepepe_math/alinear.py`, `tools/onepepe-math/medir.py`, `tests/test_alinear.py`,
  `tests/test_sondas.py`.
- **Pasos:**
  1. `ordered_groups(strokes, tokens, clf)`: DP que respeta el orden de los tokens en el orden de escritura. Para
     `\frac`, `\sqrt`, acentos y límites, prueba las variantes de orden (raya antes, en medio o después; acento
     antes o después) y se queda con la mejor. Devuelve también la puntuación, para poder filtrar.
  2. `medir.py`: lee los `.entreno.json` (ruta por argumento) y construye la referencia. Separa el examen por hash fijo
     del id de expresión (15–20 %, estratificado por set) y lo escribe en `$ONEPEPE_DATA/math/examen.json`. Informa
     de: corte exacto, juntado, partido, etiqueta, renglón perfecto, LaTeX exacto, números de 2 o más cifras, rayas de
     fracción y sustituciones erróneas, por categoría. Modos `--incremental 0.8` y `--sin-knn`. Salida JSON y tabla.
     `--comparar a.json b.json`.
  3. `test_sondas.py`: las 28 sondas como tests. Las que hoy fallan llevan `expectedFailure`, para que se vayan
     poniendo verdes.
  4. Punto de partida: los scripts de `~/.local/share/onepepe/auditoria-2026-11-15/`.
- **Aceptación:** `medir.py` reproduce las cifras de la auditoría (±1 punto) y deja el examen congelado. En «2x+1=7»,
  el alineado da 1, 2 y 2 trazos para 1, `=` y 7.

### T1. Reimportar los entrenos

- **Rama:** `fix/reimportar-entrenos`
- **Modelo recomendado:** Sonnet
- **Depende de:** T0
- **Archivos:** `onepepe_math/entreno.py` (`import_session` usa `ordered_groups` y un filtro de calidad; guarda el id
  de expresión en `notebook`, p. ej. «entreno:general#a001»). Nuevo `reimportar_entrenos.py`.
- **Pasos:**
  1. Copia de `correcciones.jsonl` a `correcciones.<fecha>.bak`.
  2. Quitar los ejemplos con `notebook` «entreno:*».
  3. Reimportar todo menos el examen.
  4. Informe de cuántos entran y cuántos van a revisión.
- **Aceptación:** tests. El usuario lo ejecuta con UNA orden.
- **Cuidado:** toca `correcciones.jsonl`, con copia previa. El agente no lo ejecuta sobre los datos reales sin permiso
  del usuario.

### T2. Entrenamiento sin fugas

- **Rama:** `fix/entrenamiento-sin-fugas`
- **Modelo recomendado:** Sonnet
- **Depende de:** T0, T1
- **Archivos:** `train_classifier.py`, `reentrenar.py`, `classifier.py` (leer `temperature`), `tests/test_reentrenar.py`.
- **Pasos:**
  1. Reparto por muestra (MathWriting) y por expresión (propio) ANTES de deformar.
  2. Bucle de épocas propio, con parada sobre la validación real (`MLPClassifier` con `warm_start` o `partial_fit`).
  3. Topes por clase y origen.
  4. `--personal-weight` elegido en validación entre 10, 30 y 90.
  5. Los «trozos» con la misma deformación que los símbolos.
  6. Temperatura ajustada en validación y guardada en el .npz.
  7. `reentrenar.py` acepta el modelo nuevo según el criterio de la receta 4.3, punto 7. Nunca usa el examen para
     entrenar.
- **Aceptación:** en el examen (no visto), el modelo nuevo mejora al actual en símbolos con corte y etiqueta, y baja
  las sustituciones erróneas. MathWriting reservado no cae más de 2 puntos.

### T3. Evaluación correcta

- **Rama:** `fix/evaluacion-correcta`
- **Modelo recomendado:** Sonnet
- **Depende de:** T0
- **Archivos:** `onepepe_math/layout.py` (`_parse_row`, `_parse_region`), `onepepe_math/evaluate.py` (`_tokens`,
  `_symbol`), `onepepe_math/pagina.py` (`_unit_text`), `tests/test_sondas.py`.
- **Pasos:** cada subtarea tiene su test en `test_sondas.py`.
  1. **3a.** `layout._parse_row`: un operador o relación tras `)` nunca abre subíndice ni exponente, salvo que sea
     claramente pequeño y desplazado. Los planos no cuentan como «más pequeños» por defecto.
  2. **3b.** Límites de `∑`, `∏` y `lim` por geometría, con `=` y `→` permitidos dentro.
  3. **3c.** Cifras: base alineada significa mismo número. Hueco máximo. La prima como marca de derivada, no como
     exponente.
  4. **3d.** Fracción: el filtro vertical usa el alto del renglón, no la mediana (caso d²/dx²).
  5. **3e.** «e» como constante. `kg` y unidades de varias letras.
- **Aceptación:** 0 resultados erróneos mostrados en `test_sondas`. Los 85 tests anteriores siguen pasando.

### T4. Entorno

- **Rama:** `feat/entorno`
- **Modelo recomendado:** Sonnet
- **Depende de:** T3
- **Archivos:** nuevo `onepepe_math/entorno.py`. `pagina.py` y `evaluate.py` lo usan. Nuevo `tests/test_entorno.py`.
- **Pasos:** lo de la sección «Propuesta: módulo `entorno.py`» de la auditoría, en este orden:
  1. Analizar sin sustituir, y variables ligadas.
  2. Funciones del usuario y llamadas.
  3. Alias de expresión y derivadas con alias.
  4. Definiciones que enseñan su valor, y entorno parcial.
  5. Pasos con variables.
  6. Caché por renglón.
- **Aceptación:** todas las sondas 1–28 en verde. Página de 40 renglones en menos de 0,2 s.

### T5. Corte: rayas primero

- **Rama:** `feat/corte-rayas`
- **Modelo recomendado:** Sonnet
- **Depende de:** T0 (mejor tras T2)
- **Archivos:** `segment.py`, `dots.py` pasa a `reparar.py`, `tests/test_segment.py`.
- **Pasos:** N1a, penalización elegida en validación, y N1c. N1b (modelo de pares) va como segundo paso, medido aparte.
- **Aceptación (examen):** raya de fracción con corte exacto ≥ 85 % (hoy 52,6 %). «=» ≥ 92 % (hoy 83,9 %). Juntados
  totales ≤ 4 % (hoy 8,9 % F). No suben los partidos.

### T6. Léxico

- **Rama:** `feat/lexico`
- **Modelo recomendado:** Sonnet
- **Depende de:** T3, T5
- **Archivos:** nuevo `onepepe_math/lexico.py`. `layout.py` y `evaluate.py` consumen sus nodos. `recognizer.py` aplica
  la corrección dentro de la entidad antes de decidir la sustitución limpia.
- **Aceptación (examen):** números de 2 o más cifras bien ≥ 90 %. `dx/dt`, `dy/dx` y `d²y/dx²` en verde en las sondas.

### T7. Clasificador v2

- **Rama:** `feat/clasificador-v2`
- **Modelo recomendado:** diseño Fable, código Sonnet
- **Depende de:** T0–T2
- **Archivos:** `onepepe_math/features.py` (`symbol_features`), `onepepe_math/classifier.py`, `onepepe_math/segment.py`
  (pasa el tamaño típico y la línea base), `onepepe_math/personal.py` (kNN), `train_classifier.py`; nuevo
  `extraer_simbolos.py` (símbolos de las fórmulas de MathWriting).
- **Pasos:**
  1. Datos: símbolos de MathWriting `train/` con el alineado ordenado y filtro, o CROHME (el usuario decide la descarga).
  2. Rasgos v2: tamaño y posición relativos al renglón (el corte debe pasar el tamaño típico y la línea base a
     `symbol_features`). Histogramas de 8 direcciones en rejilla 4×4, que no dependen del orden ni del sentido.
     Rejilla 16×16 suavizada. Subir la versión de rasgos en el .npz.
  3. Comparar el MLP v2 contra una convolucional pequeña. Entrenar con PyTorch solo en entrenamiento, e inferir en
     numpy. Medir en el examen.
  4. Reajustar el kNN.
- **Aceptación (examen, red sola con corte bueno):** top-1 ≥ 80 % (hoy 54–59 %), top-5 ≥ 95 % (hoy 76–84 %), y
  < 30 ms por renglón.
- **Cuidado:** CROHME solo si el usuario decide descargarlo. Nunca a git.

### T8. Confianza

- **Rama:** `feat/confianza`
- **Modelo recomendado:** Sonnet
- **Depende de:** T2
- **Archivos:** `onepepe_math/recognizer.py` (regla de sustitución), `onepepe_math/pagina.py` (campo `sure`),
  `crates/rnote-engine/src/engine/math.rs` (`math_apply_page`, `draw_math_overlays`).
- **Pasos:**
  1. Umbral sobre confianza calibrada.
  2. Sustituir solo si además el kNN no contradice.
  3. El resultado de un renglón con símbolos dudosos se dibuja como dudoso (campo `sure` en las marcas de `page`; Rust
     lo pinta más tenue con «?»).
- **Aceptación:** sustituciones erróneas ≤ 2 %, manteniendo ≥ 60 % de símbolos sustituidos.

### T9. Renglón vivo

- **Rama:** `feat/renglon-vivo`
- **Modelo recomendado:** diseño Fable, código Sonnet (Rust + Python)
- **Depende de:** T5
- **Archivos:** nuevo `tools/onepepe-math/reproducir_pagina.py`; `crates/rnote-engine/src/engine/math.rs`
  (`math_take_request`, `math_apply_response`), `crates/rnote-ui/src/mathhelper.rs` (`RECOGNITION_DELAY`),
  `onepepe_math/server.py` y `onepepe_math/recognizer.py`.
- **Pasos:**
  1. Paso 0 (barato): `reproducir_pagina.py` lee una página real y su archivo oculto (`times`), y simula los lotes.
     Medir cuántos símbolos quedan partidos.
  2. Solo si son más del 3 %: protocolo `reopen` (ids, trazos originales y bloqueado). Respuesta con `from`. Diferencia
     en `math_apply_response`. Sustitución retrasada en la cola. Retardo configurable. Un solo deshacer por petición.
- **Aceptación:** modo incremental igual a «todo de una vez» (±0,5 puntos). Tests de Rust para reabrir, bloquear y
  deshacer.

### T10. N mejores

- **Rama:** `feat/n-mejores`
- **Modelo recomendado:** Sonnet
- **Depende de:** T6, T7
- **Archivos:** nuevo `onepepe_math/haz.py`; `onepepe_math/recognizer.py`, `onepepe_math/context.py`
  (`reading_order`).
- **Pasos:**
  1. El contexto (pares de símbolos) recorre los símbolos en el orden del árbol de `layout`, no de izquierda a derecha.
  2. Para los símbolos con menos margen entre su primera y su segunda lectura, probar las alternativas (haz ≤ 32).
  3. Preferir la lectura que se puede analizar: paréntesis cerrados, operador entre operandos, función conocida.
- **Aceptación:** sube los renglones perfectos en el examen sin subir el tiempo por encima de 50 ms.

### T11. Captura de fallos

- **Rama:** `feat/captura-fallos`
- **Modelo recomendado:** Sonnet para Rust y Python; Haiku para textos y `es.po`.
- **Depende de:** T0
- **Archivos:** `onepepe_math/server.py`, `crates/rnote-ui/src/mathhelper.rs`, `crates/rnote-ui/po/es.po`.
- **Pasos:**
  1. Orden nueva `report_failure`: guarda trazos, tiempos y lo que debía decir en `fallos.jsonl` (local, nunca a git).
  2. Botón en la ventanita de candidatas y en el «?».
  3. Pantalla de ejemplos. Las órdenes `examples`, `forget` y `relabel` ya existen.
  4. `medir.py` acepta `fallos.jsonl` como otro origen de expresiones.
- **Aceptación:** tests del servidor; `cargo check -p rnote`; `msgfmt -c` sin errores; un fallo guardado aparece en
  la medida. La parte de la interfaz, probada por el usuario.

### T12. Diseño fluido

- **Rama:** `feat/diseno-fluido` (una rama por bloque si se hace por partes)
- **Modelo recomendado:** Sonnet; Haiku para los textos y `es.po`
- **Depende de:** nada; las propuestas 3 y 5 encajan mejor tras T8 y T9
- **Archivos:** `crates/rnote-ui/src/theme.rs`, `crates/rnote-ui/data/style.css`, `crates/rnote-ui/src/ribbon.rs`,
  `crates/rnote-ui/src/canvas/mod.rs`, `crates/rnote-engine/src/engine/math.rs` (anotaciones).
- **Pasos:** las propuestas 1 a 12 de la sección «Diseño fluido», en el orden sugerido de esa sección.
- **Aceptación:** `cargo check -p rnote` y `cargo test -p rnote theme`; con «reducir animaciones» del sistema no se
  anima nada. Las animaciones solo se pueden juzgar en el equipo del usuario.

### T13. Cierre

- **Rama:** `feat/cierre`
- **Modelo recomendado:** Haiku para documentos; Sonnet para el cambio de valores por defecto
- **Depende de:** todo lo anterior
- **Archivos:** `CAMBIOS-ONEPEPE.md`, `ESTADO.md`, `PLAN-PENDIENTE.md`, `CONTEXTO-IA.md`, `VERSIONES.md`;
  `crates/rnote-engine/src/engine/math.rs` (`MathOptions::default`).
- **Pasos:**
  1. Lista de pruebas con lápiz en `CAMBIOS-ONEPEPE.md`.
  2. Encender «=» y «Variables» por defecto, si el usuario lo aprueba.
  3. Limpiar documentos viejos.
  4. Versión.
- **Aceptación:** las metas de la «Lista priorizada» cumplidas en el examen y la lista de pruebas con lápiz hecha por
  el usuario.
- **Cuidado:** encender «=» y «Variables» por defecto solo si el usuario lo aprueba.

### Orden y paralelismo

T0 → (T1→T2) en paralelo con (T3→T4) → T5 → T6 → T8 → T7 → T10; T9 y T11 cuando haya hueco; T12–T13 al final. T3 y
T4 no necesitan datos del usuario y pueden empezar ya.

## Diseño fluido (tarea secundaria)

Base técnica (código): hoy hay 10 transiciones CSS de 100–200 ms en `style.css`, un `Revealer` para la cinta y un tick
callback en el lienzo. Las anotaciones de matemáticas se dibujan de golpe (`draw_math_overlays`), y la sustitución de
un símbolo es instantánea. libadwaita trae `adw::TimedAnimation` y `adw::SpringAnimation`. Respetar siempre el ajuste
del sistema «reducir animaciones» (`gtk-enable-animations`).

1. **Tabla de movimiento** en `theme.rs`, como los tokens de color: rápida 100 ms, normal 160 ms, lenta 240 ms. Curva
   de entrada `cubic-bezier(0.2, 0, 0, 1)`, de salida `(0.4, 0, 1, 1)`. Muelle (amortiguación 0,8) para lo que se
   arrastra. Un test que prohíba duraciones sueltas en `style.css`, como el de colores.
2. **Sustitución del símbolo con transición** (ya apuntada en `PLAN-PENDIENTE` 6.3). La tinta se desvanece mientras el
   símbolo limpio aparece, en 160–200 ms, como superposición temporal. El .rnote no cambia.
3. **«Pensando»:** un subrayado tenue que respira bajo los trazos en cola durante la espera de 0,8 s. La cola aún no
   fijada del renglón vivo lleva línea de puntos. El usuario ve cuándo se va a reconocer.
4. **Resultados que entran con suavidad:** aparecer y deslizar 4 px en 120 ms. Al cambiar el valor, fundido cruzado. El
   «?» con un solo pulso. Dibujar el resultado como matemáticas, no como texto sans con una línea «LaTeX: …». El
   LaTeX se muestra al tocar.
5. **Duda visible:** los símbolos de baja confianza con subrayado de puntos en el morado de marca. La ventanita de
   candidatas nace desde el símbolo (escala 0,96→1, 140 ms).
6. **Tachar:** los trazos borrados se encogen y desvanecen en 150 ms.
7. **Trazo orgánico:** suavizado de la punta en tiempo real (media móvil corta, solo en el tramo vivo). Final de trazo
   afinado según la velocidad. Medir la latencia del lápiz antes de tocar nada.
8. **Zoom y desplazamiento:** inercia al soltar la mano. Ctrl+0 y «ancho de página» animados 200 ms (`temporary_zoom`
   ya existe). Zoom anclado al punto bajo el lápiz.
9. **Cinta:** cambio de pestaña con fundido de 150 ms (`gtk::Stack` con Crossfade). Grupos que se reordenan sin saltos
   al estrechar la ventana.
10. **Árbol del cuaderno:** abrir y cerrar secciones con desplegado animado. La barra morada de la página activa se
    desliza de una fila a otra. Al cambiar de página, fundido corto del lienzo.
11. **Barra de opciones flotante:** muelle al soltarla y al pegarse a un lado. Plegado animado.
12. **Modo matemáticas:** el borde naranja entra con fundido de 200 ms y es más fino. El progreso de «Reentrenar» es
    continuo, no a saltos.

Orden sugerido: 1 → 2, 3, 4 (lo que más se nota en matemáticas) → 9, 10, 11 → 8 → 7.

Todo se prueba en el equipo del usuario (GTK 4.22). En la nube no se ven animaciones.

## Qué necesito de ti

1. Abrir Matemáticas y comprobar que «=» y «Variables» están encendidos. Vienen apagados.
2. Decir qué es el dataset nuevo (formato, cuántas expresiones, si lleva tiempos por trazo, si está escrito con el
   tamaño y zoom habituales).
3. Aprobar:
   - a) reimportar los entrenos (T1; toca `correcciones.jsonl`, con copia previa);
   - b) las metas de «terminado» (sección «Lista priorizada»);
   - c) descargar CROHME o no (T7);
   - d) encender «=» y «Variables» por defecto.
4. Terminar los dos sets de 200. Hay 115 de 400 escritas. Más expresiones significa un examen más fiable.
5. Una o dos páginas reales de apuntes de matemáticas con el modo activado, para medir las pausas reales (T9).
6. Los documentos están subidos (rama `feat/auditoria-plan-cierre`, fusionada en `onepepe`). Las decisiones de este
   apartado ya están respondidas en §1 de `ARQUITECTURA-EJECUCION.md`.
