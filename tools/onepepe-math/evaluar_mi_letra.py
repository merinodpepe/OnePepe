#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: ¿ayudan tus correcciones? Prueba temporal sobre tus propios ejemplos.

Los ejemplos más antiguos de correcciones.jsonl hacen de "base" (lo que el kNN ya conoce) y los más recientes de
prueba. Compara, símbolo a símbolo:
  - red sola, frente a red + kNN (tus correcciones);
  - con --anterior, también el modelo anterior (útil tras reentrenar: "red nueva + kNN" frente al modelo viejo).

Mide el acierto top-1 y cuántos símbolos superan el umbral de sustitución (95 %) y de ellos cuántos están bien.
Todo es local; no se envía nada.

Uso:
    python3 evaluar_mi_letra.py [--correcciones RUTA] [--modelo RUTA] [--anterior RUTA] [--base 0.7]
"""

import argparse
import json
import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from onepepe_math.classifier import DEFAULT_MODEL, SymbolClassifier  # noqa: E402
from onepepe_math.personal import DEFAULT_PATH, PersonalModel  # noqa: E402


def load(path):
    items = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                item = json.loads(line)
                items.append((item.get("date", ""), item["latex"], item["strokes"]))
            except (ValueError, KeyError):
                continue
    items.sort(key=lambda x: x[0])
    return [(latex, strokes) for _, latex, strokes in items]


def score(clf, test, personal=None, threshold=0.95):
    ok = sure = sure_ok = 0
    for latex, strokes in test:
        cands, junk = clf.candidates_w_junk(strokes, 5)
        if personal is not None:
            cands = personal.adjust(strokes, cands)
        top, p = cands[0]
        ok += top == latex
        if p * (1 - junk) >= threshold:
            sure += 1
            sure_ok += top == latex
    return ok, sure, sure_ok


def report(name, test, result):
    ok, sure, sure_ok = result
    extra = f" ({sure_ok / sure:.0%})" if sure else ""
    print(f"{name:28s} acierto {ok}/{len(test)} = {ok / max(len(test), 1):.0%} · "
          f"sustituiría {sure}, bien {sure_ok}{extra}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--correcciones", default=DEFAULT_PATH)
    ap.add_argument("--modelo", default=DEFAULT_MODEL)
    ap.add_argument("--anterior", default=None, help="modelo anterior, para comparar tras reentrenar")
    ap.add_argument("--base", type=float, default=0.7, help="fracción de ejemplos antiguos que hacen de base")
    args = ap.parse_args()

    items = load(args.correcciones)
    if len(items) < 10:
        print(f"Solo hay {len(items)} ejemplos tuyos: hacen falta al menos 10 para medir algo.")
        return
    n_base = int(len(items) * args.base)
    base, test = items[:n_base], items[n_base:]
    print(f"{len(base)} ejemplos de base (los más antiguos), {len(test)} de prueba (los más recientes)\n")

    for label, path in [("modelo", args.modelo), ("anterior", args.anterior)]:
        if not path:
            continue
        clf = SymbolClassifier(path)
        with tempfile.TemporaryDirectory() as d:
            personal = PersonalModel(clf, os.path.join(d, "base.jsonl"), background=False)
            for latex, strokes in base:
                personal.add_correction(latex, strokes)
            report(f"{label}: red sola", test, score(clf, test))
            report(f"{label}: red + kNN", test, score(clf, test, personal))


if __name__ == "__main__":
    main()
