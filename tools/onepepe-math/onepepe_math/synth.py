# SPDX-License-Identifier: GPL-3.0-or-later
"""Fórmulas sintéticas para probar el reconocedor sin datos del usuario.

Toma las cajas de cada símbolo de `synthetic-bboxes.jsonl` (MathWriting) y coloca en cada una un símbolo real escrito a
mano del 20 % reservado (nunca usado para entrenar el clasificador), como hace el propio MathWriting.
"""

import glob
import json
import os
import random

from .inkml import read_inkml
from .symbols import normalize_label


def is_holdout(sample_id):
    return int(sample_id[-2:], 16) % 5 == 0


def load_holdout_symbols(dataset_dir, holdout=True):
    by_label = {}
    for path in sorted(glob.glob(os.path.join(dataset_dir, "symbols", "*.inkml"))):
        sample_id = os.path.splitext(os.path.basename(path))[0]
        if is_holdout(sample_id) != holdout:
            continue
        ann, strokes = read_inkml(path)
        if strokes:
            by_label.setdefault(normalize_label(ann.get("label", "")), []).append(strokes)
    return by_label


def _fit(strokes, box):
    xs = [p[0] for s in strokes for p in s]
    ys = [p[1] for s in strokes for p in s]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    w, h = max(x1 - x0, 1e-6), max(y1 - y0, 1e-6)
    bx0, by0, bx1, by1 = box
    bw, bh = bx1 - bx0, by1 - by0
    scale = min(bw / w, bh / h) if bw > 1e-6 and bh > 1e-6 else (bw / w if bw > 1e-6 else bh / h)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    bcx, bcy = (bx0 + bx1) / 2, (by0 + by1) / 2
    return [[(bcx + (x - cx) * scale, bcy + (y - cy) * scale) for x, y in s] for s in strokes]


def formulas(dataset_dir, n, seed=0, max_symbols=12, holdout=True, vocabulary=None):
    """Genera n fórmulas: lista de dicts {"latex", "strokes", "truth": [(etiqueta, [índices de trazos])]}.

    Con `holdout` (por defecto) usa solo símbolos reservados que el clasificador no vio al entrenar.
    """
    rng = random.Random(seed)
    by_label = load_holdout_symbols(dataset_dir, holdout)
    path = os.path.join(dataset_dir, "synthetic-bboxes.jsonl")
    with open(path, encoding="utf-8") as f:
        lines = f.readlines()
    out = []
    while len(out) < n:
        entry = json.loads(rng.choice(lines))
        boxes = entry["bboxes"]
        if not boxes or len(boxes) > max_symbols:
            continue
        labels = [normalize_label(b["token"]) for b in boxes]
        if any(lab not in by_label for lab in labels):
            continue
        if vocabulary is not None and any(lab not in vocabulary for lab in labels):
            continue
        strokes, truth = [], []
        for b, lab in zip(boxes, labels):
            sym = _fit(rng.choice(by_label[lab]), (b["xMin"], b["yMin"], b["xMax"], b["yMax"]))
            idx = list(range(len(strokes), len(strokes) + len(sym)))
            strokes.extend(sym)
            truth.append((lab, idx))
        out.append({"latex": entry["normalizedLabel"], "strokes": strokes, "truth": truth})
    return out
