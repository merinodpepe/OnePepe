#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Entrena el clasificador de símbolos con los símbolos sueltos de MathWriting.

Uso (necesita numpy y scikit-learn; ver requirements-train.txt):
    ./descargar_mathwriting.sh
    python3 train_classifier.py ~/.local/share/onepepe/mathwriting/mathwriting-2024

Escribe el modelo en ~/.local/share/onepepe/math/simbolos.npz (unos pocos MB). No lo subas al repositorio.
"""

import argparse
import json
import glob
import os
import sys
import time

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from onepepe_math import synth  # noqa: E402
from onepepe_math.classifier import DEFAULT_MODEL, JUNK  # noqa: E402
from onepepe_math.features import augment, symbol_features  # noqa: E402
from onepepe_math.inkml import read_inkml  # noqa: E402
from onepepe_math.symbols import normalize_label  # noqa: E402


def is_holdout(sample_id):
    """Un 20 % fijo de los símbolos nunca se usa para entrenar: sirve para las pruebas de fórmulas completas."""
    return int(sample_id[-2:], 16) % 5 == 0


def load_symbols(dataset_dir, holdout=False):
    """Símbolos sueltos de MathWriting: lista de (etiqueta, trazos). Sin `holdout`, excluye el 20 % reservado."""
    samples = []
    for path in sorted(glob.glob(os.path.join(dataset_dir, "symbols", "*.inkml"))):
        sample_id = os.path.splitext(os.path.basename(path))[0]
        if is_holdout(sample_id) != holdout:
            continue
        ann, strokes = read_inkml(path)
        if strokes:
            samples.append((normalize_label(ann.get("label", "")), strokes))
    return samples


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dataset", help="carpeta mathwriting-2024 descomprimida")
    ap.add_argument("-o", "--output", default=DEFAULT_MODEL)
    ap.add_argument("--augment", type=int, default=30, help="variantes deformadas por muestra")
    ap.add_argument("--test-per-class", type=int, default=5)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--junk-formulas", type=int, default=3000)
    ap.add_argument("--junk-max", type=int, default=6000)
    ap.add_argument(
        "--personal",
        default=None,
        help="tus ejemplos (correcciones.jsonl): entran en el entrenamiento con más peso que los de MathWriting",
    )
    ap.add_argument("--personal-augment", type=int, default=90, help="variantes deformadas por ejemplo tuyo")
    args = ap.parse_args()

    from sklearn.neural_network import MLPClassifier

    rng = np.random.default_rng(args.seed)
    samples = load_symbols(args.dataset)
    print(f"{len(samples)} símbolos cargados", file=sys.stderr)
    by_class = {}
    for label, strokes in samples:
        by_class.setdefault(label, []).append(strokes)
    # Clase "trozo": grupos de trazos que NO son un símbolo completo, para que la agrupación no parta ni junte de más.
    # (1) trazos sueltos de símbolos de varios trazos (una raya del "=", el punto de la "i")
    junk = []
    for label, strokes in samples:
        if len(strokes) > 1:
            for s in strokes:
                junk.append([s])
    # (2) dos símbolos vecinos juntados, en fórmulas sintéticas hechas con símbolos de entrenamiento
    for f in synth.formulas(args.dataset, args.junk_formulas, seed=args.seed + 7, holdout=False):
        truth = f["truth"]
        for (_, a), (_, b) in zip(truth, truth[1:]):
            if len(a) + len(b) <= 4:
                junk.append([f["strokes"][i] for i in a + b])
    rng.shuffle(junk)
    by_class[JUNK] = junk[: args.junk_max]
    print(f"{len(by_class[JUNK])} ejemplos de 'trozo'", file=sys.stderr)
    personal = []
    if args.personal and os.path.exists(args.personal):
        with open(args.personal, encoding="utf-8") as f:
            for line in f:
                try:
                    item = json.loads(line)
                    label = JUNK if item.get("kind") == "junk" else item["latex"]
                    personal.append((label, [[tuple(p) for p in s] for s in item["strokes"]]))
                except (ValueError, KeyError, TypeError):
                    continue
        for label, _ in personal:
            by_class.setdefault(label, [])
        n_junk = sum(1 for label, _ in personal if label == JUNK)
        print(f"{len(personal)} ejemplos tuyos ({n_junk} de 'trozo')", file=sys.stderr)
    classes = sorted(by_class)
    index = {c: i for i, c in enumerate(classes)}

    train_x, train_y, test_x, test_y = [], [], [], []
    for c in classes:
        items = by_class[c]
        order = rng.permutation(len(items))
        n_test = min(args.test_per_class, len(items) // 4)
        for j, k in enumerate(order):
            strokes = items[k]
            if j < n_test:
                test_x.append(symbol_features(strokes))
                test_y.append(index[c])
                continue
            train_x.append(symbol_features(strokes))
            train_y.append(index[c])
            for _ in range(args.augment if c != JUNK else 2):
                train_x.append(symbol_features(augment(strokes, rng)))
                train_y.append(index[c])
    # tus ejemplos: siempre en el entrenamiento (la prueba con tu letra la hace evaluar_mi_letra.py)
    for label, strokes in personal:
        train_x.append(symbol_features(strokes))
        train_y.append(index[label])
        for _ in range(args.personal_augment if label != JUNK else max(args.personal_augment // 6, 2)):
            train_x.append(symbol_features(augment(strokes, rng)))
            train_y.append(index[label])
    train_x, test_x = np.array(train_x), np.array(test_x)
    train_y, test_y = np.array(train_y), np.array(test_y)
    mean = train_x.mean(axis=0)
    std = train_x.std(axis=0) + 1e-3
    print(f"entrenando con {len(train_x)} muestras, {len(classes)} clases ...", file=sys.stderr)
    t0 = time.time()
    clf = MLPClassifier(
        hidden_layer_sizes=(384, 192),
        alpha=1e-4,
        batch_size=256,
        learning_rate_init=1e-3,
        max_iter=40,
        early_stopping=True,
        n_iter_no_change=4,
        random_state=args.seed,
        verbose=False,
    )
    clf.fit((train_x - mean) / std, train_y)
    print(f"entrenado en {time.time() - t0:.0f} s", file=sys.stderr)

    # además, el 20 % reservado (otros escritores/muestras, nunca vistos)
    for label, strokes in load_symbols(args.dataset, holdout=True):
        if label in index:
            test_x = np.vstack([test_x, symbol_features(strokes)[None, :]])
            test_y = np.append(test_y, index[label])
    p = clf.predict_proba((test_x - mean) / std)
    top = np.argsort(-p, axis=1)
    top1 = float(np.mean(top[:, 0] == test_y))
    top5 = float(np.mean([y in t[:5] for y, t in zip(test_y, top)]))
    print(f"prueba ({len(test_y)} símbolos no vistos): top-1 {top1:.3f}, top-5 {top5:.3f}")

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    arrays = {"classes": np.array(classes), "mean": mean, "std": std, "n_layers": np.array(len(clf.coefs_))}
    # versión: cambia en cada reentrenamiento, para recalcular los embeddings de tus ejemplos
    arrays["version"] = np.array(time.strftime("%Y%m%d-%H%M%S") + f"-{len(personal)}")
    for i, (w, b) in enumerate(zip(clf.coefs_, clf.intercepts_)):
        arrays[f"W{i}"] = w.astype(np.float32)
        arrays[f"b{i}"] = b.astype(np.float32)
    # modelo de pares de símbolos, de las etiquetas de las fórmulas de entrenamiento
    from onepepe_math.context import BigramModel
    from onepepe_math.latex_tokens import visible_tokens

    seqs = []
    for path in glob.glob(os.path.join(args.dataset, "train", "*.inkml")):
        with open(path, encoding="utf-8") as f:
            head = f.read(2000)
        start = head.find('type="normalizedLabel">')
        if start < 0:
            continue
        label = head[start + len('type="normalizedLabel">') : head.find("</annotation>", start)]
        seqs.append(visible_tokens(label))
    lm = BigramModel.train(seqs, [c for c in classes if c != JUNK])
    print(f"modelo de contexto con {len(seqs)} fórmulas", file=sys.stderr)
    arrays["lm_vocab"] = np.array(lm.vocab)
    arrays["lm_unigram"] = lm.unigram
    arrays["lm_bigram"] = lm.bigram
    np.savez_compressed(args.output, **arrays)
    print(f"modelo guardado en {args.output}")


if __name__ == "__main__":
    main()
