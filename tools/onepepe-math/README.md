# onepepe-math (prototipo, Fase 3)

Herramientas para el reconocimiento de matemáticas de OnePepe. De momento **no están conectadas a la app**: sirven para
reunir datos de tu letra y probar el reconocedor fuera de Rnote. El diseño está en [`MATEMATICAS-INFORME.md`](../../MATEMATICAS-INFORME.md).

| Archivo | Para qué sirve |
|---|---|
| `rnote_extract.py` | Saca los trazos de un `.rnote` a JSON, los separa por renglones y dibuja un SVG por renglón. Solo usa la biblioteca estándar de Python. |
| `expresiones.txt` | Las 30 expresiones que te pido que escribas, en LaTeX, en orden. |
| `descargar_mathwriting.sh` | Descarga el dataset público MathWriting fuera del repositorio (`~/.local/share/onepepe/`). |
| `tests/` | Tests del extractor, con un `.rnote` sintético ("2+3=") generado por el motor de Rnote. |

## Lo que te pido (≈ 30–40 minutos)

1. Abre OnePepe (o Rnote) y crea un documento nuevo. Usa el **pincel normal**, color negro, sin tinta a formas.
2. Escribe las 30 expresiones de `expresiones.txt` **en orden, una por renglón**, dejando **un renglón vacío** entre una y
   otra. Escribe como en clase: rápido y con tu letra de siempre. No escribas los números de las expresiones.
   - Si te equivocas, bórralo con el borrador y vuelve a escribirlo (lo borrado no se extrae).
   - No escribas el resultado después de "=".
3. Guárdalo como `mis_expresiones_1.rnote`. Si puedes, otro día repite la lista en `mis_expresiones_2.rnote`.
4. En otra página, `mis_simbolos.rnote`, escribe **5 veces seguidas** cada uno de estos grupos que se confunden,
   un grupo por renglón: `x × X K k κ` · `1 l | I` · `0 O o θ` · `2 z Z` · `5 S s` · `u μ v ν` · `t +` · `a α` ·
   `B β` · `n η` · `e ε` · `p ρ` · `w ω` · `9 g q` · `d ∂` · `Δ ∇` · `Σ ∑` · `∫ f`.
5. Comprueba la extracción y pásame los archivos (ver abajo).

## Comandos

Desde la carpeta del repositorio:

```bash
cd tools/onepepe-math
python3 rnote_extract.py ~/ruta/mis_expresiones_1.rnote -o expresiones_1.json \
    --svg svg_1 --labels expresiones.txt
```

Abre los archivos `svg_1/renglon_XX.svg` con el navegador o el visor de imágenes: cada uno debe contener **una sola
expresión completa**. Si el programa avisa de que el número de renglones no coincide con el de expresiones, es que
dos expresiones están demasiado juntas o una fracción alta se partió: dímelo y lo ajusto.

Los datos se quedan en tu equipo. **No los subas al repositorio** (un fork de un proyecto público suele ser público).
Para dármelos, lo más sencillo es una de estas dos vías:

- **Adjuntarlos en el chat** de esta sesión, si tu app te deja adjuntar archivos.
- **Subirlos a tu Google Drive** en una carpeta, p. ej. `OnePepe-datos`, y decirme el nombre: esta sesión tiene
  acceso de lectura a tu Drive y puedo descargarlos desde ahí.

Archivos a enviar: `mis_expresiones_1.rnote` (y `_2` si lo hiciste) y `mis_simbolos.rnote`. Los `.json` no hacen falta:
los genero yo.

Tests:

```bash
python3 -m unittest discover -s tests
```
