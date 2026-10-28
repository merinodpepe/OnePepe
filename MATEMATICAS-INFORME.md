# Reconocimiento y resolución de matemáticas — informe (Fase 3, primer entregable)

Fecha: 2026-10-26 (actualizado el mismo día con el cambio de plan). Estado: **investigación y prototipo de datos**.
Nada de esto está aún en la app. Herramientas y lista de expresiones: [`tools/onepepe-math/`](tools/onepepe-math/README.md).
El cambio de plan (parte A) estaba en `PLAN-PENDIENTE.md`; ya está hecho y ese archivo ahora solo recoge lo pendiente
(la versión original está en su historial de git).

## 0. Cambio de plan (parte A)

El objetivo principal ya no es calcular, sino **pasar tu letra a trazos limpios**. El cálculo del "=" queda como
función aparte. El reconocimiento (secciones 2.2 y siguientes) no cambia: es la base de las dos funciones.

### 0.1 Función principal: sustituir tu letra por símbolos limpios

- **Qué hace:** al reconocer un símbolo con confianza suficiente, borra tus trazos de ese símbolo y dibuja en su lugar
  el símbolo limpio, en la **misma posición y tamaño** (centrado en la caja de tu símbolo y escalado a su altura; los
  símbolos planos como "−" o "·" se escalan por la anchura). No se reordena ni se realinea nada: si tu exponente estaba
  torcido, el limpio queda en el mismo sitio.
- **Trazos normales de Rnote:** cada símbolo limpio se inserta como uno o varios trazos de pincel (`BrushStroke`) con tu
  color y grosor, seleccionables y borrables como cualquier otro. **No hace falta cambiar el formato `.rnote`**: ese
  tipo de trazo ya existe.
- **Símbolo dudoso:** si la probabilidad de la mejor candidata (después del reescoring con contexto) no llega al
  **umbral de confianza**, tus trazos se quedan tal cual.
- **Ajustes:** interruptor (activado/desactivado) y umbral de confianza de 50 % a 99 % (por defecto **95 %**, elegido
  midiendo, ver 0.4), en la configuración del pincel, como la tinta a formas. Todo ocurre solo dentro del modo
  matemáticas, que viene apagado.
- **Deshacer:** igual que tinta a formas: tu tinta original se guarda en el historial antes de sustituirla, y un Ctrl+Z
  la recupera.
- **Cuándo se dispara:** en el modo matemáticas, tras una pausa corta sin escribir (≈ 0,8 s), sobre los trazos nuevos de
  ese renglón; así el reconocedor ve el símbolo completo (p. ej. las dos rayas del "=") y su contexto.
- **Correcciones:** al tocar un símbolo limpio aparece la lista de candidatas. Al elegir otra, el símbolo limpio se
  redibuja con la nueva; con "todo el grupo", se redibujan todos los del grupo. Cada corrección se guarda como ejemplo de
  tu letra (con tus trazos originales) en `~/.local/share/onepepe/math/correcciones.jsonl`.
- **Corregir después de cerrar y volver a abrir:** confirmado por ti e **implementado**: un archivo oculto al lado de
  la página (`.Página 1.rnote.onepepe.json`) recuerda qué trazos forman cada símbolo, sus candidatas, el grupo y tus
  trazos originales, sin tocar el `.rnote`.

### 0.2 Fuente de los símbolos limpios

- **Elegida: fuentes Hershey** (Dr. A. V. Hershey, National Bureau of Standards, EE. UU.), en sus variantes de **trazo
  único** (Simplex para latinas y cifras, Greek Simplex para griegas, y los símbolos matemáticos). Son líneas, no
  contornos, así que cada glifo se convierte directamente en trazos de pincel normales, con el grosor que uses.
- **Licencia (según la nota de distribución que acompaña a los datos, p. ej. en la documentación de Ghostscript):** se
  pueden usar para cualquier fin, también comercial, siempre que (1) se incluyan los créditos a A. V. Hershey (NBS) y a
  James Hurt (Cognition, Inc., formato de los datos), y (2) no se redistribuyan en el formato original del gobierno de
  EE. UU. (NTIS). Pondré los créditos en un archivo junto a los datos y en `DECISIONES.md`. **Comprobaré al implementar
  que estén todos los símbolos que hacen falta** (∂, ∇, ∫, ∑, √, ±, ≈, ≠, ≤, ≥, →…); si falta alguno, se dibujará a mano
  como trazo y lo anotaré, o ese símbolo se dejará con tu letra.
- **Descartada como fuente principal: una fuente de contornos** como STIX Two Math (licencia SIL OFL 1.1). Sus glifos
  son contornos rellenos: pasarlos a trazos de pincel exige "esqueletizarlos", lo que los deforma, o meterlos como formas
  rellenas, que ya no se comportan como tinta al borrar.

### 0.3 Función aparte: resultado del "=" en gris

Se activa y desactiva por separado en los ajustes (por defecto desactivada). Sigue siendo una capa superpuesta, **no
tinta**, y no se muestra nada si la lectura no es válida o la confianza es baja. Qué hará:

| Lo que escribes | Lo que muestra |
|---|---|
| Expresión sin incógnitas y "=" (`3+4·5=`, `k_BT=` con T del glosario) | el valor: exacto si es una fracción sencilla, si no decimal con 4 cifras significativas; con unidades si las hay |
| Ecuación con una incógnita y "incógnita =" en el mismo renglón o en el siguiente (`2x+1=7, x=`) | la solución (`x = 3`); si hay varias, todas (`x = 2, x = 3`); "sin solución" si no la hay |
| Ecuación con una sola incógnita, sin indicar cuál, terminada en "=" vacío | igual que la anterior, despejando la única incógnita |
| Expresión con variables y "=" (`2a+3a-b=`) | la forma simplificada (`5a − b`) **solo si es distinta y no más larga** que lo escrito; `(a+b)²=` ya es simple, así que no muestra nada |
| Derivada (`d/dx(x³+2x)=`, `∂f/∂t` con f conocida) | la derivada |
| Integral indefinida / definida | primitiva "+ C" / valor |
| Varias incógnitas sin indicar cuál despejar, o algo ambiguo | nada |

Junto al resultado se muestra en pequeño el LaTeX entendido, para que compruebes la lectura.


### 0.4 Estado de la implementación (2026-10-26, sin tus datos)

Hecho y fusionado en `onepepe` (rama `feat/matematicas-limpias`); detalle y pruebas en `CAMBIOS-ONEPEPE.md`:

- **Programa auxiliar** `tools/onepepe-math` (Python): agrupación de trazos, clasificador (red pequeña, numpy en
  ejecución), contexto por bigramas + Viterbi, grupos por forma, glosario por cuaderno, clasificador personal con tus
  correcciones, estructura espacial, cálculo del "=" (tabla de 0.3: valor, despejar, simplificar, derivar, integrar,
  sumatorios, límites, unidades y comprobación dimensional) y glifos Hershey. 18 tests.
- **En Rnote:** modo matemáticas (botón y Ctrl+Alt+M, borde naranja), sustitución por símbolos limpios con umbral y
  deshacer, toque para corregir un símbolo o su grupo, resultado en gris con el LaTeX entendido, ajustes. Tests del
  motor con respuestas simuladas.
- **Fuente:** Hershey vía el paquete `Hershey-Fonts` de PyPI (envoltorio con licencia MIT; los datos de Hershey se
  usan con los créditos exigidos, ver 0.2). Comprobada visualmente la tabla de símbolos; θ sale con la forma ϑ (diseño
  de Hershey); ≈ y ∼ están dibujados a mano porque no están en la fuente.
- **Formato `.rnote`:** sin cambios. La relación símbolo ↔ trazos se guarda en el archivo oculto de 0.1
  (rama `feat/matematicas-archivo-oculto`), así que se puede corregir tras reabrir la página.

**Medidas** (fórmulas sintéticas de MathWriting con símbolos de escritores que el clasificador no vio; vocabulario
básico; **no es tu letra**):

| Medida | Resultado |
|---|---|
| Símbolo suelto: primera candidata / entre las 5 primeras | 61 % / 92 % (229 clases) |
| Agrupación de trazos en símbolos correcta | 90 % |
| Símbolos correctos con contexto / entre las 3 primeras | 66 % / 86 % |
| Sustitución con umbral 95 %: símbolos sustituidos / de ellos visualmente correctos | ~50 % / ~84 % |
| Estructura correcta con símbolos correctos (fórmula entera) | 76 % |
| Fórmula entera perfecta de punta a punta | ~10 % |
| Tiempo por fórmula | ~5 ms |

**Lectura:** el modelo genérico entrenado con unos 30 ejemplos por símbolo no basta para sustituir sin errores. Lo que
falta para que sea útil es justo lo que depende de ti: tus 30 expresiones (para medir y ajustar con tu letra) y tus
correcciones (el clasificador personal las aplica al momento). Por eso el modo matemáticas viene apagado y el umbral alto.

**Pendiente que sí depende de ti:** medir con tus expresiones, ajustar umbrales y la estructura a tu letra "poco
lineal". (El permiso de Flatpak para lanzar el programa auxiliar ya está en el manifiesto, a petición tuya.)

### 0.5 Medidas con tu letra y kNN sobre embeddings (2026-10-28)

**Tus datos:** `mis_expresiones_1.rnote`, las 30 expresiones (391 trazos). Se usan solo en la nube de la sesión y
**no se suben al repositorio**. El extractor partía las fracciones, raíces y exponentes altos en renglones aparte (43
en vez de 30); ahora junta los renglones separados por un hueco mucho menor que el típico entre expresiones y saca
30.

**Modelo genérico (solo MathWriting) con tu letra:**

| Medida | Resultado |
|---|---|
| Símbolos bien | 124/255 = **49 %** |
| Fórmulas enteras exactas | **0/30** |
| Sustituidos con seguridad ≥ 95 % | 118, de ellos bien 90 (**76 %**) |
| Expresiones partidas en el nº correcto de símbolos | 7/29 |

Dos causas principales:

- **La segmentación:** cómo se cortan los trazos en símbolos. Tu "=" o tu "+" a veces se juntan con el vecino y
  otras veces se parten.
- **Símbolos parecidos que se confunden:** x↔×, 0↔O/∘, α↔ν, k↔F, n↔w.

**Cómo se midió:**

- Los símbolos esperados se emparejan con los segmentos por asignación de coste mínimo.
- Para simular correcciones, los segmentos de las expresiones antiguas, con su etiqueta correcta, hacen de ejemplos
  tuyos:
  - los mal reconocidos cuentan como corregidos;
  - los bien reconocidos cuentan como aceptados sin tocar.

**División temporal (ejemplos de las expresiones 1-15, prueba en 16-30):**

| Variante | Símbolos bien | Sustituidos ≥ 95 % bien |
|---|---|---|
| Red sola | 60/136 = 44 % | 41/60 = 68 % |
| Red + kNN antiguo (`personal.py` anterior) | 59/136 = 43 % | 37/56 = 66 % |
| Red + kNN nuevo (τ=0,1, d_max=0,3, λ_max=0,6) | 59/136 = 43 % | 37/51 = **73 %** |
| **Red reentrenada con tus 99 ejemplos** (`--personal`) | **70/136 = 51 %** | **55/69 = 80 %** |
| Red reentrenada + kNN | 66/136 = 49 % | 49/59 = **83 %** |

En la división al revés (ejemplos de 16-30, prueba en 1-15), el kNN nuevo pasa de 54 % a 58 % de símbolos y de
84 % a 85 % de sustituciones correctas.

**Conclusiones:**

1. **El kNN** casi no cambia el acierto, pero sustituye menos y se equivoca menos al sustituir. Como su papel es
   cubrir lo que corriges entre reentrenamientos, se queda. Su medida de verdad será con tus correcciones reales:
   `tools/onepepe-math/evaluar_mi_letra.py` hace la prueba temporal que pide el plan (red sola, red + kNN y modelo
   anterior).
2. **Reentrenar con tus ejemplos** es lo que más ayuda: +7 puntos de símbolos y +12 de sustituciones correctas con
   solo 99 ejemplos. Con la letra de otras personas (MathWriting) el acierto no baja: top-1 0,603 frente
   a 0,61. Por eso `instalar.sh --entrenar` ya incluye tus correcciones si existen.
3. **Lo siguiente con más impacto:**
   - Ajustar la segmentación a tu forma de espaciar.
   - La página de símbolos sueltos (`mis_simbolos.rnote`), que da muchos ejemplos limpios por símbolo.
   - El botón "Reentrenar ahora" de la app.

**Qué quedó hecho del plan de kNN:**

- **Red:** devuelve probabilidades y embedding en una pasada, y tiene versión de modelo.
- **Almacén:** guarda trazos, etiqueta correcta y predicha, embedding, versión, fecha y cuaderno. Desde el programa
  auxiliar se puede listar, borrar y cambiar (`examples`, `forget`, `relabel`). Si cambia la versión del modelo, los
  embeddings se recalculan en segundo plano.
- **Vecinos:** por fuerza bruta detrás de `NeighborIndex`.
- **Mezcla:** la propuesta, con el almacén vacío idéntica a antes.
- **Corrección en bloque:** ya usaba el mismo embedding (`shape_groups`).
- **Reentrenamiento:** con `--personal`.
- **Falta:**
  - Comparar automáticamente "red nueva + kNN" con el modelo anterior dentro del futuro botón "Reentrenar ahora". El
    script ya lo hace con `--anterior`.
  - Una pantalla en la app para ver y borrar tus ejemplos.

## 1. Resumen (del primer entregable; el cambio de plan está en la sección 0)

- **Arquitectura:** un programa auxiliar en Python (`onepepe-math`) que Rnote lanza **una vez** y mantiene abierto,
  hablando por entrada/salida estándar con mensajes JSON (uno por línea). Mantengo tu propuesta con un cambio: el proceso
  vive mientras la app está abierta, para no cargar el modelo en cada "=" (eso tardaría segundos).
- **El reconocedor trabaja por símbolos, no por imagen completa.** Es lo único compatible con lo que pides: candidatas
  con probabilidad por símbolo, corrección de un grupo entero con un toque, y aprendizaje de tu letra.
- **Datos públicos:** **MathWriting** (Google, 2024–2025) se puede descargar desde aquí y es justo lo que hace falta:
  230 000 fórmulas manuscritas con sus trazos y su LaTeX, y además símbolos sueltos y la correspondencia trazo → símbolo.
  **HASYv2 y CROHME no se pueden descargar desde este entorno** (bloqueados por la red).
- **Cálculo:** SymPy 1.14 + pint 0.26 + SciPy 1.18 instalados y probados aquí: fracciones, raíces, derivadas,
  integrales, sumatorios, límites, matrices, unidades y comprobación dimensional funcionan.
- **Hallazgo importante:** el analizador de LaTeX de SymPy solo entiende **19 de las 30** expresiones de prueba
  (falla con Δ, π, η, ∂, ∇, ±, unidades, `\Pi`… y lee mal `\sin^2\theta`). Por eso el programa construirá la expresión
  de SymPy **desde su propio árbol de símbolos**; el LaTeX servirá solo para enseñarte lo que entendió.
- **Lo que necesito de ti:** 30 expresiones escritas con tu letra (lista en `tools/onepepe-math/expresiones.txt`)
  y una página de símbolos que se confunden. Instrucciones en `tools/onepepe-math/README.md`.

## 2. Arquitectura propuesta

```
Rnote (Rust)                                        onepepe-math (Python, proceso aparte)
───────────────────────────────────────────         ─────────────────────────────────────────────
pincel termina un trazo ──► ¿modo mates activo   ──► 1. agrupar trazos en símbolos (varias hipótesis)
                            o "=" sin resultado?     2. clasificador → top-k candidatas con probabilidad
                                                     3. reescoring con contexto (grupos, vecinos,
                                                        glosario, dimensiones)
                                                     4. estructura espacial (potencias, subíndices,
                                                        fracciones, raíces) tolerante al desorden
                                                     5. SymPy / pint / scipy.constants
capa superpuesta (no es tinta): ◄─────────────────  JSON: símbolos+candidatas, LaTeX, resultado,
  resultado en gris + LaTeX entendido                    confianza; nada si no es válido
toque en un símbolo → lista de candidatas ──────►    corrección → ~/.local/share/onepepe/math/
  (solo este / todo el grupo)                            correcciones.jsonl (ejemplo de tu letra)
```

### 2.1 Por qué no un modelo "imagen → LaTeX"

Los mejores resultados publicados en 2025 son modelos de extremo a extremo (p. ej. Uni-MuMER, ajuste fino de un modelo
de visión y lenguaje de varios miles de millones de parámetros). Los descarto como pieza central porque:
(a) devuelven una cadena, no candidatas por símbolo, así que no permiten tu corrección por grupos;
(b) pesan varios GB y necesitan GPU para ir fluidos; (c) no aprenden de tus correcciones sin reentrenar;
(d) no he encontrado ningún modelo así con pesos publicados, ligero y listo para CPU (búsqueda del 26-10-2026).
Pueden añadirse más adelante como **una hipótesis más** que compita en el paso 3.

### 2.2 Las cinco piezas

1. **Agrupación en símbolos.** Trazos consecutivos en el tiempo y cercanos (cajas que se solapan o casi) son candidatos a
   formar un símbolo (`+`, `=`, `x`, `π`, `Σ`, `≠`…). Se generan varias hipótesis (p. ej. "`=` o dos `−`") y decide el
   paso 3. El `.rnote` no guarda la hora de cada punto, pero dentro de la app sí la tengo (eventos del lápiz), y el orden
   de los trazos sí está en el archivo.
2. **Clasificador de símbolos** con salida top-k y probabilidades. Primera versión: red pequeña sobre la secuencia de
   puntos remuestreada (direcciones, curvatura, posición relativa), entrenada con los símbolos de MathWriting y exportada
   a ONNX para ir rápida en CPU (unos pocos MB). En paralelo, un **clasificador personal** por vecinos más cercanos sobre tus
   correcciones guardadas; sus votos se mezclan con los del modelo general y pesan más cuantos más ejemplos tuyos hay.
3. **Reescoring con contexto** (lo que pediste):
   - **Grupos por forma:** se comparan entre sí los símbolos de la página (distancia entre sus representaciones internas).
     Los parecidos forman un grupo; si la mayoría del grupo es claramente "K", los dudosos heredan la K.
   - **Vecinos:** reglas y un modelo de lenguaje de LaTeX muy pequeño (frecuencias de pares/tríos de símbolos sacadas
     de MathWriting): `×` es probable entre dos números; `x` junto a un exponente es variable; `l` entre cifras es `1`.
   - **Glosario por cuaderno:** archivo de texto en la carpeta del cuaderno con símbolos habituales (`k_B`, `T`, `ΔG`,
     `R`, `N_A`…) que suben la probabilidad a priori; los que confirmas se añaden solos.
   - **Dimensiones:** con pint, una lectura que suma metros con segundos se penaliza.
4. **Estructura espacial tolerante.** Analizador por reglas sobre las cajas de los símbolos: línea base, superíndices,
   subíndices, fracciones (barra horizontal larga con cosas encima y debajo), raíces (lo que cae bajo el `√`), con
   márgenes amplios y varias hipótesis cuando hay duda (¿`x²` o `x 2`?). MathWriting trae fórmulas "sintéticas" con la caja
   de cada símbolo, perfectas para afinar este paso sin tu letra.
5. **Cálculo.** El árbol de símbolos se convierte directamente en una expresión de SymPy (no vía LaTeX); unidades con
   pint (o `sympy.physics.units`) y constantes con `scipy.constants` (k_B, N_A, R, e, h, F…). Si algo no es una expresión
   válida o la confianza total es baja, **no se muestra nada**.

### 2.3 Protocolo entre Rnote y el programa (borrador; el implementado está documentado en `tools/onepepe-math/onepepe_math/server.py`)

Petición (una línea JSON):

```json
{"id": 17, "mode": "equals", "notebook": "/home/pepe/Apuntes/Biofisica",
 "strokes": [{"id": 0, "points": [[x, y, presión, t_ms], ...]}, ...]}
```

Respuesta:

```json
{"id": 17, "valid": true, "confidence": 0.86,
 "symbols": [{"strokes": [0, 1], "group": 4,
              "candidates": [{"latex": "k", "p": 0.71}, {"latex": "K", "p": 0.22}, {"latex": "\\kappa", "p": 0.04}]}],
 "latex": "k_{B}T", "result_latex": "4.28\\times10^{-21}\\,\\mathrm{J}", "result_text": "4,28·10⁻²¹ J"}
```

Corrección (la app la envía al elegir una candidata): `{"correct": {"strokes": [0, 1], "latex": "K", "scope": "group"}}`.

## 3. Librerías, modelos y datos candidatos

| Recurso | Qué es | ¿Descargable desde aquí? | Licencia | Uso |
|---|---|---|---|---|
| **MathWriting** (Google) | 230k fórmulas manuscritas online + 400k sintéticas, InkML con trazos y LaTeX; carpeta de símbolos sueltos y `symbols.jsonl` (trazos → símbolo) | **Sí** (storage.googleapis.com; 2,9 GB; extracto de 1,5 MB probado) | CC BY-NC-SA 4.0 (uso personal OK) | entrenar clasificador, segmentación y estructura; evaluar |
| MathWriting+ (Zenodo) | MathWriting con símbolos etiquetados y relaciones | **No** (zenodo.org bloqueado) | — | opcional |
| HASYv2 | 168k imágenes 32×32 de 369 símbolos | **No** (zenodo.org y huggingface.co bloqueados) | no comprobada | no imprescindible: MathWriting tiene trazos, que es mejor |
| CROHME 2014/16/19/23 | ~10k expresiones InkML, referencia académica | **No** (webs de CROHME/TC11 bloqueadas) | según edición | solo para comparar |
| Uni-MuMER (2025) | VLM de extremo a extremo, estado del arte | Código en GitHub: **no accesible** (fuera del ámbito de esta sesión); pesos no comprobados | — | descartado como pieza central (ver 2.1) |
| SymPy 1.14 (+ lark 1.3) | álgebra, derivadas, integrales, límites, sumatorios, matrices | **Sí** (PyPI), instalado y probado | BSD | cálculo |
| pint 0.26 | unidades y comprobación dimensional | **Sí**, probado | BSD | unidades |
| SciPy 1.18 (`scipy.constants`) | constantes físicas CODATA | **Sí**, probado | BSD | constantes |
| onnxruntime / PyTorch | ejecutar / entrenar el clasificador | **Sí** (PyPI responde; no instalados aún) | MIT / BSD | modelo |
| **Fuentes Hershey** (trazo único) | glifos latinos, griegos y matemáticos dibujados como líneas | no comprobado aún desde aquí (hay copias en muchos paquetes, p. ej. Ghostscript) | uso libre con créditos; no redistribuir en el formato original NTIS (ver 0.2) | símbolos limpios |
| STIX Two Math | fuente matemática de contornos | no comprobado | SIL OFL 1.1 | descartada como principal (ver 0.2) |

Comprobado en este entorno: `k_B·T` a 310 K = 4,28·10⁻²⁴ kJ (2,58 kJ/mol); sumar metros con segundos da error
dimensional, como debe.

## 4. Qué se puede probar en la nube y qué no

| Se puede aquí | Solo en tu equipo |
|---|---|
| Entrenar y medir el clasificador de símbolos con MathWriting (CPU; un modelo pequeño es viable) | Cómo se comporta con **tu** letra (hasta que me pases muestras) |
| Segmentación y estructura espacial con las fórmulas sintéticas de MathWriting (traen cajas por símbolo) | La capa gris sobre el lienzo, el borde de color del modo matemáticas, el toque para corregir |
| El protocolo JSON y el cálculo con SymPy/pint/SciPy, con tests | Velocidad real en tu ordenador |
| Tus `.rnote`, una vez me los pases, convertidos con el extractor | — |

## 5. Qué datos tuyos necesito

1. **`mis_expresiones_1.rnote`**: las 30 expresiones de `tools/onepepe-math/expresiones.txt`, una por renglón, con un
   renglón vacío entre ellas. Mezclan aritmética, álgebra, griegas, subíndices, unidades, constantes, derivadas,
   integrales, sumatorios, límites, una matriz y fórmulas de biofísica (ΔG, k_BT, Stokes-Einstein, Nernst, presión osmótica).
   Si puedes, una segunda copia otro día (`_2`): la variación de tu letra entre días es justo lo que hay que medir.
2. **`mis_simbolos.rnote`**: los grupos de símbolos que se confunden, 5 veces cada uno (lista en el README).
3. **Respuestas cortas:** (a) ¿qué signo usas para multiplicar (·, ×, nada)? (b) ¿escribes la coma decimal como "," o "."?
   (c) ¿unidades en el mismo renglón que el número (p. ej. "25 mM")? (d) ¿asignaturas y símbolos típicos de cada una para
   el glosario, además de biofísica?
4. **Cómo me los pasas:** adjuntos en el chat o en una carpeta de tu Google Drive (esta sesión puede leer tu Drive).
   **No los subas al repositorio.**

La herramienta para convertirlos ya está hecha y probada: `tools/onepepe-math/rnote_extract.py` (lee el `.rnote`,
separa renglones, asocia cada uno a su expresión de la lista y dibuja un SVG por renglón para revisar).

## 6. Plan de las etapas (cuando me respondas; actualizado con el cambio de plan)

1. **Aritmética y álgebra** con griegas y subíndices: clasificador + agrupación + reescoring + **sustitución por
   símbolos limpios** (con umbral, deshacer y ajustes) + corrección por símbolo o grupo. Interruptor del modo
   matemáticas con borde de color. Estructura espacial solo en lo necesario para el contexto.
2. **Resultado del "=" en gris** (función aparte): estructura (potencias, subíndices, fracciones, raíces), valor
   numérico y LaTeX entendido.
3. **Unidades y constantes** con comprobación dimensional y glosario por cuaderno.
4. **Ecuaciones a despejar, simplificar, derivadas e integrales** (tabla de 0.3).
5. **Matrices, sumatorios y límites.**

Cada etapa se mide primero con MathWriting y después con tu letra; no paso a la siguiente sin tu prueba.

## 7. Riesgos que veo

- **Tu letra "poco lineal":** el paso 4 es el más delicado. Tus muestras dirán cuánta tolerancia hace falta.
- **Mayúsculas/minúsculas y griegas parecidas** (x/×/κ, u/μ, v/ν): sin contexto son casi imposibles; por eso el
  reescoring y las correcciones por grupo son la parte más valiosa.
- **Python en Flatpak:** si usas la versión Flatpak, el programa auxiliar tiene que ir dentro del Flatpak (añadir
  SymPy/pint/SciPy/onnxruntime al manifiesto) o lanzarse fuera con `flatpak-spawn --host`. Lo decidiré cuando
  sepa cómo compilas en tu equipo.

## Fuentes

- Fuentes Hershey: [Wikipedia](https://en.wikipedia.org/wiki/Hershey_fonts), [nota de uso en la documentación de Ghostscript](https://web.mit.edu/ghostscript/src/gs6.01/doc/Hershey.htm), [solhsa.com](https://solhsa.com/hershey/)
- MathWriting: [artículo (arXiv 2404.10690)](https://arxiv.org/html/2404.10690v2), [repositorio y descarga](https://github.com/google-research/google-research/tree/master/mathwriting), [KDD 2025](https://dl.acm.org/doi/10.1145/3711896.3737436), [MathWriting+ en Zenodo](https://zenodo.org/record/14975367)
- Uni-MuMER: [Towards Scalable Training for HMER (arXiv 2508.09220)](https://arxiv.org/pdf/2508.09220)
- Otros modelos recientes: [TAMER](https://arxiv.org/pdf/2408.08578), [ICAL](https://arxiv.org/pdf/2405.09032), [grafos con atención (2410.18555)](https://arxiv.org/pdf/2410.18555)
- HASYv2: [Zenodo 259444](https://zenodo.org/records/259444), [artículo](https://ar5iv.labs.arxiv.org/html/1701.08380)
- SymPy, analizador LaTeX: [código de `parse_latex`](https://github.com/sympy/sympy/blob/95085aaa2de28fb98a3f4a07d4ca305b741e8a9a/sympy/parsing/latex/__init__.py), [proyecto GSoC del analizador Lark](https://summerofcode.withgoogle.com/programs/2023/projects/apj3AeJL)
