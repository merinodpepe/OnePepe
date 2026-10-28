#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: convierte una página tuya etiquetada en ejemplos de tu letra para el reconocedor.

Lee una página .rnote con una expresión por renglón y el archivo de expresiones en LaTeX (por ejemplo
`mis_expresiones_1.rnote` y `expresiones.txt`). Para cada expresión busca el corte en símbolos que mejor encaja con
los símbolos esperados y guarda en tu almacén local (~/.local/share/onepepe/math/correcciones.jsonl):
  - cada símbolo con su etiqueta (entra en el kNN al momento y en el próximo reentrenamiento);
  - "trozos" (medio símbolo, o dos símbolos juntos) para enseñar a cortar tu letra al reentrenar.
Después pulsa «Reentrenar» en la app (o ejecuta reentrenar.py). Nada sale de tu equipo.

Uso:
    python3 importar_mi_letra.py mis_expresiones_1.rnote expresiones.txt
"""

import argparse
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import rnote_extract  # noqa: E402
from onepepe_math.classifier import SymbolClassifier  # noqa: E402
from onepepe_math.forced import forced_groups, junk_examples  # noqa: E402
from onepepe_math.latex_tokens import _INVISIBLE, _TOKEN  # noqa: E402
from onepepe_math.personal import DEFAULT_PATH, PersonalModel  # noqa: E402
from onepepe_math.symbols import normalize_label  # noqa: E402

FUNCTIONS = ("sin", "cos", "tan", "ln", "log", "lim", "exp")


def expected_symbols(latex):
    """Los símbolos que se escriben a mano en una expresión (las funciones, letra a letra)."""
    latex = re.sub(r"\\(begin|end)\{[a-z]*\}", " ", latex).replace(r"\mathrm", "")
    out = []
    for t in _TOKEN.findall(latex):
        if t in _INVISIBLE or t in "{}":
            continue
        if t.startswith("\\") and t[1:] in FUNCTIONS:
            out += list(t[1:])
            continue
        out.append(normalize_label(t))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("rnote")
    ap.add_argument("expresiones")
    ap.add_argument("--almacen", default=DEFAULT_PATH)
    ap.add_argument("--modelo", default=None)
    args = ap.parse_args(argv)

    _version, snapshot = rnote_extract.load_rnote(args.rnote)
    strokes = rnote_extract.extract_strokes(snapshot)
    by_id = {s["id"]: s for s in strokes}
    lines = rnote_extract.split_lines(strokes)
    labels = rnote_extract.read_labels(args.expresiones)
    if len(lines) != len(labels):
        print(f"AVISO: {len(lines)} renglones y {len(labels)} expresiones; se usan en orden hasta donde lleguen.")

    clf = SymbolClassifier(args.modelo) if args.modelo else SymbolClassifier()
    store = PersonalModel(clf, args.almacen, background=False)
    n_sym = n_junk = skipped = 0
    for line, latex in zip(lines, labels):
        line_strokes = [[(p[0], p[1]) for p in by_id[i]["points"]] for i in line["stroke_ids"]]
        labeled = forced_groups(line_strokes, expected_symbols(latex), clf)
        if labeled is None:
            skipped += 1
            continue
        for group, label in labeled:
            store.add_correction(label, [line_strokes[i] for i in group], notebook=os.path.basename(args.rnote))
            n_sym += 1
        for piece in junk_examples(line_strokes, labeled):
            store.add_correction("__trozo__", piece, notebook=os.path.basename(args.rnote), kind="junk")
            n_junk += 1
    print(f"Guardados {n_sym} símbolos y {n_junk} trozos de tu letra en {args.almacen}"
          + (f" ({skipped} expresiones no encajaron y se saltaron)" if skipped else ""))
    print("Ahora pulsa «Reentrenar» en la app (Dibujar → Ayudas) o ejecuta: python3 reentrenar.py")


if __name__ == "__main__":
    main()
