# Reconocimiento y resolución de matemáticas — informe (Fase 3, primer entregable)

Fecha: 2026-10-26. Estado: **investigación y prototipo de datos**. Nada de esto está aún en la app.
Herramientas y lista de expresiones: [`tools/onepepe-math/`](tools/onepepe-math/README.md).

## 1. Resumen

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

### 2.3 Protocolo entre Rnote y el programa (borrador)

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

## 6. Plan de las etapas (cuando me respondas)

1. **Aritmética y álgebra** con griegas y subíndices: clasificador + agrupación + estructura (potencias, subíndices,
   fracciones, raíces) + "=" + capa gris + LaTeX entendido. Interruptor del modo matemáticas con borde de color.
2. **Unidades y constantes** con comprobación dimensional y glosario por cuaderno.
3. **Ecuaciones, derivadas e integrales.**
4. **Matrices, sumatorios y límites.**

Cada etapa se mide primero con MathWriting y después con tu letra; no paso a la siguiente sin tu prueba.

## 7. Riesgos que veo

- **Tu letra "poco lineal":** el paso 4 es el más delicado. Tus muestras dirán cuánta tolerancia hace falta.
- **Mayúsculas/minúsculas y griegas parecidas** (x/×/κ, u/μ, v/ν): sin contexto son casi imposibles; por eso el
  reescoring y las correcciones por grupo son la parte más valiosa.
- **Python en Flatpak:** si usas la versión Flatpak, el programa auxiliar tiene que ir dentro del Flatpak (añadir
  SymPy/pint/SciPy/onnxruntime al manifiesto) o lanzarse fuera con `flatpak-spawn --host`. Lo decidiré cuando
  sepa cómo compilas en tu equipo.

## Fuentes

- MathWriting: [artículo (arXiv 2404.10690)](https://arxiv.org/html/2404.10690v2), [repositorio y descarga](https://github.com/google-research/google-research/tree/master/mathwriting), [KDD 2025](https://dl.acm.org/doi/10.1145/3711896.3737436), [MathWriting+ en Zenodo](https://zenodo.org/record/14975367)
- Uni-MuMER: [Towards Scalable Training for HMER (arXiv 2508.09220)](https://arxiv.org/pdf/2508.09220)
- Otros modelos recientes: [TAMER](https://arxiv.org/pdf/2408.08578), [ICAL](https://arxiv.org/pdf/2405.09032), [grafos con atención (2410.18555)](https://arxiv.org/pdf/2410.18555)
- HASYv2: [Zenodo 259444](https://zenodo.org/records/259444), [artículo](https://ar5iv.labs.arxiv.org/html/1701.08380)
- SymPy, analizador LaTeX: [código de `parse_latex`](https://github.com/sympy/sympy/blob/95085aaa2de28fb98a3f4a07d4ca305b741e8a9a/sympy/parsing/latex/__init__.py), [proyecto GSoC del analizador Lark](https://summerofcode.withgoogle.com/programs/2023/projects/apj3AeJL)
