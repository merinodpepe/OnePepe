#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Entrena el clasificador de símbolos con los símbolos sueltos de MathWriting y, si se da, con tu letra.

Uso (necesita numpy y scikit-learn; ver requirements-train.txt):
    ./descargar_mathwriting.sh
    python3 train_classifier.py ~/.local/share/onepepe/mathwriting/mathwriting-2024

Escribe el modelo en ~/.local/share/onepepe/math/simbolos.npz (unos pocos MB). No lo subas al repositorio.

Cómo evita las fugas (ARQUITECTURA-EJECUCION.md §6 M2): el reparto entrenamiento / validación se hace ANTES de deformar
las muestras (las copias de una muestra no cruzan de lado); la parada se decide con una validación sin deformar
(J = ½ NLL de tu letra + ½ NLL de MathWriting); y se calibra una temperatura T con esa validación.
El examen (`examen.json`) nunca entra.
"""

import argparse
import copy
import glob
import json
import os
import re
import sys
import time

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from onepepe_math import alinear, synth  # noqa: E402
from onepepe_math.classifier import DEFAULT_MODEL, JUNK, SymbolClassifier  # noqa: E402
from onepepe_math.features import N_FEATURES, N_FEATURES_V2, augment, linea, perturbar_linea, symbol_features  # noqa: E402
from onepepe_math.inkml import read_inkml  # noqa: E402
from onepepe_math.symbols import CORE, normalize_label  # noqa: E402

TOPE_EXTRAIDOS = 400  # símbolos extraídos de fórmulas (extraer_simbolos.py) por clase
COPIAS_EXTRAIDOS = 10
SOLTAR_LINEA = 0.3  # M7.5: fracción de muestras con contexto a las que se les borra
TOPE_PROPIO = 40  # copias propias por clase: como mucho TOPE_PROPIO · W
BASE_POR_CLASE = 80  # muestras base de MathWriting por clase
PESOS = (10, 30, 90)


def is_holdout(sample_id):
    """Un 20 % fijo de los símbolos nunca se usa para entrenar: sirve para las pruebas de fórmulas completas."""
    return int(sample_id[-2:], 16) % 5 == 0


def es_validacion_mw(sample_id):
    return int(sample_id[-4:-2], 16) % 8 == 0


def cargar_simbolos(dataset_dir, holdout=False):
    """Símbolos sueltos de MathWriting: [(etiqueta, trazos, id)]. Sin `holdout`, excluye el 20 % reservado."""
    out = []
    for path in sorted(glob.glob(os.path.join(dataset_dir, "symbols", "*.inkml"))):
        sample_id = os.path.splitext(os.path.basename(path))[0]
        if is_holdout(sample_id) != holdout:
            continue
        ann, strokes = read_inkml(path)
        if strokes:
            out.append((normalize_label(ann.get("label", "")), strokes, sample_id))
    return out


# ---------------------------------------------------------------- reparto y mezcla


def reparto_propio(items, conj, excluir_pliegue=None):
    """Tu letra: items de correcciones.jsonl -> (entrenamiento, validación), listas de (etiqueta, trazos, origen).
    Con `notebook = entreno:<set>#<id>` manda el reparto congelado `conj` (examen.json): el examen no entra nunca, la
    validación va aparte. Lo demás (correcciones a mano) entra todo en el entrenamiento. `excluir_pliegue`: quita las
    expresiones de ese pliegue de la medida cruzada."""
    train, val = [], []
    for it in items:
        label = JUNK if it.get("kind") == "junk" else it["latex"]
        row = (label, [[tuple(p) for p in s] for s in it["strokes"]], it.get("id"))
        m = re.match(r"entreno:(.+)#([^#]+)$", it.get("notebook") or "")
        if not m:
            train.append(row)
            continue
        s, i = m.groups()
        row = (row[0], row[1], f"{s}#{i}")
        if i in conj.get("examen", {}).get(s, ()):
            continue
        if excluir_pliegue is not None and alinear.pliegue(f"{s}#{i}") == excluir_pliegue:
            continue
        (val if i in conj.get("validacion", {}).get(s, ()) else train).append(row)
    return train, val


def copias_propias(n_c, w):
    """Copias deformadas por ejemplo propio de una clase con `n_c` ejemplos base."""
    return max(1, min(w, int(TOPE_PROPIO * w / n_c)))


def copias_trozo(n_simbolos, n_trozos):
    """Copias por trozo para que los trozos queden en torno al 15 % del total."""
    return int(min(max(round(0.15 * n_simbolos / n_trozos), 2), 30))


def featurizar(muestras, copias, rng, lineas=None, version=1, soltar=SOLTAR_LINEA):
    """(X, etiquetas, orígenes) de cada muestra [(etiqueta, trazos, origen)] y de sus `copias[i]` deformaciones.
    v2: `lineas[i]` (típico, pendiente, corte) o None; en las copias se perturba, y se borra con probabilidad `soltar`."""
    dim = N_FEATURES_V2 if version >= 2 else N_FEATURES
    X = np.empty((len(muestras) + int(sum(copias)), dim), dtype=np.float32)
    y, o, r = [], [], 0

    def ctx(lin):
        return None if lin is None or (version >= 2 and rng.random() < soltar) else lin

    for i, ((label, strokes, origen), n) in enumerate(zip(muestras, copias)):
        lin = lineas[i] if lineas is not None else None
        X[r] = symbol_features(strokes, ctx(lin), version)
        y.append(label)
        o.append(origen)
        r += 1
        for _ in range(n):
            X[r] = symbol_features(augment(strokes, rng), ctx(perturbar_linea(lin, rng)), version)
            y.append(label)
            o.append(origen)
            r += 1
    return X, y, o


def cargar_extraidos(ruta, rng, tope=TOPE_EXTRAIDOS):
    """Símbolos extraídos (jsonl de extraer_simbolos.py): hasta `tope` por clase -> (muestras, líneas)."""
    por_clase = {}
    with open(ruta, encoding="utf-8") as f:
        for texto in f:
            try:
                it = json.loads(texto)
                fila = (normalize_label(it["etiqueta"]), [[tuple(p) for p in s] for s in it["trazos"]], it["id"])
            except (ValueError, KeyError, TypeError):
                continue
            por_clase.setdefault(fila[0], []).append((fila, tuple(it["linea"]) if it.get("linea") else None))
    elegidas = [f for c in sorted(por_clase) for f in [por_clase[c][k] for k in rng.permutation(len(por_clase[c]))][:tope]]
    return [f for f, _ in elegidas], [ln for _, ln in elegidas]


# ---------------------------------------------------------------- red, parada y temperatura


def logits(X, coefs, intercepts):
    for i, (w, b) in enumerate(zip(coefs, intercepts)):
        X = X @ w + b
        if i < len(coefs) - 1:
            X = np.maximum(X, 0.0)
    return X


def nll(clf, X, y):
    if len(y) == 0:
        return None
    p = clf.predict_proba(X)
    return float(-np.mean(np.log(np.maximum(p[np.arange(len(y)), y], 1e-12))))


def ajustar_temperatura(z, y, w=None):
    """T = 1/β que minimiza la log-verosimilitud negativa de softmax(β·z) (Newton, 20 pasos; L es convexa en β)."""
    z = np.asarray(z, dtype=np.float64)
    w = np.ones(len(z)) if w is None else np.asarray(w, dtype=np.float64)
    fila = np.arange(len(z))
    beta = 1.0
    for _ in range(20):
        s = beta * z
        p = np.exp(s - s.max(axis=1, keepdims=True))
        p /= p.sum(axis=1, keepdims=True)
        e = (p * z).sum(axis=1)
        var = (p * z * z).sum(axis=1) - e * e
        g = (w * (e - z[fila, y])).sum() / w.sum()
        h = (w * var).sum() / w.sum()
        beta = float(np.clip(beta - g / max(h, 1e-6), 0.125, 4.0))
    return 1.0 / beta


def entrenar_red(Xtr, ytr, val_m, val_p, n_clases, args, epocas_fijas=None):
    """Una época por `partial_fit`; se guardan los pesos de menor J (o los de la última época con `epocas_fijas`).
    Devuelve (coefs, intercepts, J, épocas)."""
    from sklearn.neural_network import MLPClassifier

    clf = MLPClassifier(hidden_layer_sizes=(384, 192), alpha=1e-4, batch_size=256, learning_rate_init=1e-3,
                        random_state=args.seed)
    todas = np.arange(n_clases)
    mejor, sin_mejora, mejor_ep, j_mejor = None, 0, 0, np.inf
    for ep in range(1, (epocas_fijas or args.max_epocas) + 1):
        clf.partial_fit(Xtr, ytr, classes=todas)
        nl_p, nl_m = nll(clf, *val_p), nll(clf, *val_m)
        partes = [v for v in (nl_p, nl_m) if v is not None]
        j = sum(partes) / max(len(partes), 1)  # ½ NLL propia + ½ NLL MathWriting (o la única que haya)
        print(f"  época {ep}: NLL propia {nl_p}, NLL MathWriting {nl_m}, J {j:.4f}", file=sys.stderr)
        if epocas_fijas or j < j_mejor - 1e-9:
            j_mejor, mejor_ep, sin_mejora = j, ep, 0
            mejor = copy.deepcopy((clf.coefs_, clf.intercepts_))
        else:
            sin_mejora += 1
            if sin_mejora >= args.paciencia:
                break
    return mejor[0], mejor[1], j_mejor, mejor_ep


def top1_reservado(clf, dataset_dir):
    """(top-1, top-5, n) de un SymbolClassifier en el 20 % reservado de MathWriting."""
    conocidas = set(clf.labels)
    top1 = top5 = n = 0
    for label, strokes, _ in cargar_simbolos(dataset_dir, holdout=True):
        if label not in conocidas:
            continue
        p = clf.probabilities(strokes)
        orden = np.argsort(-p)
        n += 1
        top1 += clf.labels[orden[0]] == label
        top5 += label in {clf.labels[i] for i in orden[:5]}
    return top1 / max(n, 1), top5 / max(n, 1), n


# ---------------------------------------------------------------- programa


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("dataset", help="carpeta mathwriting-2024 descomprimida")
    ap.add_argument("-o", "--output", default=DEFAULT_MODEL)
    ap.add_argument("--augment", type=int, default=30, help="copias deformadas por muestra de MathWriting")
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--junk-formulas", type=int, default=3000)
    ap.add_argument("--junk-max", type=int, default=6000)
    ap.add_argument("--personal", default=None, help="tus ejemplos (correcciones.jsonl)")
    ap.add_argument("--personal-weight", default="30", choices=[str(w) for w in PESOS] + ["auto"],
                    help="copias deformadas por ejemplo tuyo; auto prueba 10, 30 y 90 y se queda con el de menor J")
    ap.add_argument("--epocas-fijas", type=int, default=None, help="sin parada: justo N épocas (los pliegues)")
    ap.add_argument("--excluir-pliegue", type=int, default=None, choices=range(4))
    ap.add_argument("--rasgos", type=int, default=1, choices=[1, 2], help="versión de los rasgos (2: M7.2-M7.5)")
    ap.add_argument("--extraidos", default=None, help="símbolos extraídos de fórmulas (simbolos.jsonl de T7a)")
    ap.add_argument("--max-epocas", type=int, default=60)
    ap.add_argument("--paciencia", type=int, default=5)
    args = ap.parse_args()

    rng = np.random.default_rng(args.seed)
    version = args.rasgos
    simbolos = cargar_simbolos(args.dataset)
    print(f"{len(simbolos)} símbolos cargados", file=sys.stderr)
    sym_tr = [s for s in simbolos if not es_validacion_mw(s[2])]
    sym_va = [s for s in simbolos if es_validacion_mw(s[2])]
    # Clase "trozo": grupos de trazos que NO son un símbolo completo. (1) trazos sueltos de símbolos de varios trazos
    # (del lado de su símbolo); (2) dos símbolos vecinos juntados, de fórmulas sintéticas (fórmula nº % 8 == 0 valida)
    junk_tr = [(JUNK, [s], i) for _, st, i in sym_tr if len(st) > 1 for s in st]
    junk_va = [(JUNK, [s], i) for _, st, i in sym_va if len(st) > 1 for s in st]
    for n, f in enumerate(synth.formulas(args.dataset, args.junk_formulas, seed=args.seed + 7, holdout=False)):
        truth = f["truth"]
        for (_, a), (_, b) in zip(truth, truth[1:]):
            if len(a) + len(b) <= 4:
                (junk_va if n % 8 == 0 else junk_tr).append((JUNK, [f["strokes"][i] for i in a + b], f"f{n}"))
    rng.shuffle(junk_tr)
    junk_tr = junk_tr[: args.junk_max]
    print(f"{len(junk_tr)} ejemplos de 'trozo'", file=sys.stderr)

    prop_tr, prop_va, lin_origen = [], [], {}
    if args.personal and os.path.exists(args.personal):
        items = []
        with open(args.personal, encoding="utf-8") as f:
            for line in f:
                try:
                    it = json.loads(line)
                    it["latex"], it["strokes"]
                    items.append(it)
                except (ValueError, KeyError, TypeError):
                    continue
        ruta = os.path.join(os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math",
                            "examen.json")
        conj = json.load(open(ruta, encoding="utf-8")) if os.path.exists(ruta) else {}
        prop_tr, prop_va = reparto_propio(items, conj, args.excluir_pliegue)
        if version >= 2:  # el renglón de cada expresión del entreno: todos sus trazos juntos
            exp = {}
            for it in items:
                m = re.match(r"entreno:(.+)#([^#]+)$", it.get("notebook") or "")
                if m and it.get("kind") != "junk":
                    exp.setdefault(m.groups(), []).extend([tuple(p) for p in s] for s in it["strokes"])
            lin_origen = {f"{s}#{i}": linea(t) for (s, i), t in exp.items()}
        print(f"{len(prop_tr)} ejemplos tuyos para entrenar y {len(prop_va)} para validar", file=sys.stderr)

    ext, lin_ext = [], []
    if args.extraidos and version >= 2:
        ext, lin_ext = cargar_extraidos(args.extraidos, rng)
        print(f"{len(ext)} símbolos extraídos de fórmulas", file=sys.stderr)

    clases = sorted({s[0] for s in sym_tr + sym_va + prop_tr + prop_va + ext} | {JUNK})
    idx = {c: i for i, c in enumerate(clases)}

    # MathWriting: como mucho BASE_POR_CLASE muestras base por clase, cada una con `augment` copias
    por_clase = {}
    for s in sym_tr:
        por_clase.setdefault(s[0], []).append(s)
    base_m = [s for c in sorted(por_clase) for s in [por_clase[c][k] for k in rng.permutation(len(por_clase[c]))
                                                      ][:BASE_POR_CLASE]]
    Xm, ym, _ = featurizar(base_m, [args.augment] * len(base_m), rng, version=version)
    Xe, ye, _ = featurizar(ext, [COPIAS_EXTRAIDOS] * len(ext), rng, lin_ext, version)

    def valid(muestras, lineas=None):
        X, y, _ = featurizar(muestras, [0] * len(muestras), rng, lineas, version, soltar=0.0)
        return X, np.array([idx[e] for e in y], dtype=int)

    Xvm, yvm = valid(sym_va + junk_va)
    Xvp, yvp = valid(prop_va, [lin_origen.get(s[2]) for s in prop_va] if version >= 2 else None)

    permitidas = [i for i, c in enumerate(clases) if c == JUNK or normalize_label(c) in CORE]
    en_permitidas = {c: k for k, c in enumerate(permitidas)}
    pesos = PESOS if args.personal_weight == "auto" else (int(args.personal_weight),)
    mejor = None
    t0 = time.time()
    for w in pesos:
        # tu letra: copias por ejemplo según el tamaño de su clase (tope 40·W por clase)
        propios = [s for s in prop_tr if s[0] != JUNK]
        n_c = {}
        for s in propios:
            n_c[s[0]] = n_c.get(s[0], 0) + 1
        lin_p = [lin_origen.get(s[2]) for s in propios] if version >= 2 else None
        Xp, yp, _ = featurizar(propios, [copias_propias(n_c[s[0]], w) for s in propios], rng, lin_p, version)
        n_s = len(Xm) + len(Xp) + len(Xe)
        trozos = junk_tr + [s for s in prop_tr if s[0] == JUNK]
        a_t = copias_trozo(n_s, len(trozos)) if trozos else 0
        lin_j = ([None] * len(junk_tr) + [lin_origen.get(s[2]) for s in prop_tr if s[0] == JUNK]) if version >= 2 else None
        Xj, yj, _ = featurizar(trozos, [a_t] * len(trozos), rng, lin_j, version)
        Xtr = np.concatenate([Xm, Xp, Xe, Xj])
        ytr = np.array([idx[e] for e in ym + yp + ye + yj], dtype=int)
        del Xp, Xj
        if len(Xtr) > 0 and w == pesos[-1]:
            del Xm, Xe  # la memoria manda: solo se liberan en la última vuelta de W
        mean = Xtr.mean(axis=0)
        std = Xtr.std(axis=0) + 1e-3
        Xtr -= mean  # en el sitio: la matriz es grande
        Xtr /= std
        print(f"entrenando con {len(Xtr)} muestras, {len(clases)} clases (W={w}) ...", file=sys.stderr)
        coefs, inters, j, ep = entrenar_red(Xtr, ytr, ((Xvm - mean) / std, yvm), ((Xvp - mean) / std, yvp),
                                            len(clases), args, args.epocas_fijas)
        del Xtr
        print(f"  W={w}: J {j:.4f} en {ep} épocas", file=sys.stderr)
        if mejor is None or j < mejor[0]:
            mejor = (j, w, ep, coefs, inters, mean, std)
    j, w, ep, coefs, inters, mean, std = mejor
    print(f"entrenado en {time.time() - t0:.0f} s", file=sys.stderr)

    # temperatura: validación sin deformar; tu letra y MathWriting pesan ½ cada una
    X = np.vstack([Xvp, Xvm])
    y = np.concatenate([yvp, yvm])
    peso = np.concatenate([np.full(len(yvp), 0.5 / max(len(yvp), 1)), np.full(len(yvm), 0.5 / max(len(yvm), 1))])
    if not len(yvp):
        peso[len(yvp):] = 1.0 / max(len(yvm), 1)
    z = logits((X - mean) / std, coefs, inters)[:, permitidas]
    ok = np.array([k in en_permitidas for k in y])
    T = ajustar_temperatura(z[ok], np.array([en_permitidas[k] for k in y[ok]]), peso[ok])
    print(f"temperatura T={T:.3f}, W={w}, épocas={ep}, J={j:.4f}", file=sys.stderr)

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    arrays = {"classes": np.array(clases), "mean": mean, "std": std, "n_layers": np.array(len(coefs)),
              "temperature": np.array(T), "feature_version": np.array(version), "epocas": np.array(ep),
              "personal_weight": np.array(w)}
    # versión: cambia en cada reentrenamiento, para recalcular los embeddings de tus ejemplos
    arrays["version"] = np.array(time.strftime("%Y%m%d-%H%M%S") + f"-{len(prop_tr)}")
    for i, (cw, cb) in enumerate(zip(coefs, inters)):
        arrays[f"W{i}"] = cw.astype(np.float32)
        arrays[f"b{i}"] = cb.astype(np.float32)
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
    lm = BigramModel.train(seqs, [c for c in clases if c != JUNK])
    print(f"modelo de contexto con {len(seqs)} fórmulas", file=sys.stderr)
    arrays["lm_vocab"] = np.array(lm.vocab)
    arrays["lm_unigram"] = lm.unigram
    arrays["lm_bigram"] = lm.bigram
    np.savez_compressed(args.output, **arrays)

    # el 20 % reservado de MathWriting (otros escritores, nunca visto)
    top1, top5, n = top1_reservado(SymbolClassifier(args.output), args.dataset)
    print(f"prueba ({n} símbolos no vistos): top-1 {top1:.3f}, top-5 {top5:.3f}")
    print(f"modelo guardado en {args.output}")


if __name__ == "__main__":
    main()
