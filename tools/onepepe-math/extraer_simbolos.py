# SPDX-License-Identifier: GPL-3.0-or-later
"""Extrae símbolos de las fórmulas de MathWriting con el alineado ordenado (ARQUITECTURA-EJECUCION.md M7.7, M0.2).

Orden de las fórmulas: por u("onepepe-extraccion-v1", id). Una fórmula vale si el 80 % o más de sus símbolos cumple
A(j) (solo red); de ella entran los símbolos `aceptado(j)`. Se para al tener `--por-clase` símbolos de cada clase de
CORE o al llegar a `--tope` fórmulas. Se guarda el progreso cada `--cada` fórmulas y se puede reanudar.

    export ONEPEPE_DATA=... ONEPEPE_MATHWRITING=...
    python extraer_simbolos.py                 # train -> $ONEPEPE_DATA/mathwriting-extraido/simbolos.jsonl
    python extraer_simbolos.py --split valid   # 3 000 fórmulas de valid/ -> simbolos_valid.jsonl
"""

import argparse
import itertools
import json
import math
import multiprocessing
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from onepepe_math import alinear  # noqa: E402
from onepepe_math.classifier import SymbolClassifier  # noqa: E402
from onepepe_math.entreno import expected_symbols  # noqa: E402
from onepepe_math.inkml import read_inkml  # noqa: E402
from onepepe_math.features import linea  # noqa: E402,F401  (M7.5; vive en features.py)
from onepepe_math.segment import MAX_STROKES  # noqa: E402
from onepepe_math.symbols import CORE  # noqa: E402

MAX_FICHAS = 25
_CLF = None  # clasificador compartido con los procesos hijos (fork)


def formula_valida(tokens, n_trazos, core):
    """Fichas todas en `core`, como mucho 25, y n en [m, 4m]."""
    m = len(tokens)
    return 0 < m <= MAX_FICHAS and all(t in core for t in tokens) and m <= n_trazos <= MAX_STROKES * m


def supera_80(A):
    return bool(A) and sum(A) >= 0.8 * len(A)


def _analizar(item):
    """(id, filas) de una fórmula; filas = None si no vale (alineado imposible o menos del 80 %)."""
    fid, latex, strokes = item
    res = alinear.ordered_groups(strokes, expected_symbols(latex), _CLF, latex=latex, tardios=False)
    if res is None:
        return fid, None
    pt = alinear._Puntuador(strokes, _CLF)
    A = [lab in {t for t, _ in sorted(pt.p(tuple(ids)).items(), key=lambda x: -x[1])[:5]}
         for ids, lab in res["grupos"]]
    if not supera_80(A):
        return fid, None
    ext = [True] + A + [True]
    ok = [A[j] or (ext[j] and ext[j + 2]) for j in range(len(A))]
    ln = linea(strokes)
    ln = [round(float(x), 3) for x in ln] if ln else None
    return fid, [{"etiqueta": lab, "trazos": [[[round(x, 2), round(y, 2)] for x, y in strokes[i]] for i in ids],
                  "id": fid, "linea": ln} for (ids, lab), a in zip(res["grupos"], ok) if a]


def extraer(formulas, clf, salida, core=CORE, por_clase=400, tope=30000, cada=1000, procesos=1):
    """Procesa `formulas` [(id, latex, trazos)] en orden y añade los símbolos a `salida` (.jsonl). Reanuda desde
    `salida + '.progreso'`. Devuelve el progreso {"vistas", "validas", "bytes", "conteo", "segundos"}."""
    global _CLF
    _CLF = clf
    prog_path = salida + ".progreso"
    prog = {"vistas": 0, "validas": 0, "bytes": 0, "conteo": {}, "segundos": 0.0}
    if os.path.exists(prog_path) and os.path.exists(salida):
        with open(prog_path) as f:
            prog = json.load(f)
    os.makedirs(os.path.dirname(os.path.abspath(salida)), exist_ok=True)
    mode = "r+b" if os.path.exists(salida) else "w+b"
    out = open(salida, mode)
    out.truncate(prog["bytes"])  # restos de una parada a medias
    out.seek(prog["bytes"])
    t0 = time.time() - prog["segundos"]
    pool = multiprocessing.Pool(procesos) if procesos > 1 else None
    it = itertools.islice(formulas, prog["vistas"], None)

    def guardar():
        out.flush()
        os.fsync(out.fileno())
        prog.update(bytes=out.tell(), segundos=time.time() - t0)
        with open(prog_path + ".tmp", "w") as f:
            json.dump(prog, f)
        os.replace(prog_path + ".tmp", prog_path)

    def completo():
        return all(prog["conteo"].get(c, 0) >= por_clase for c in core)

    try:
        while prog["vistas"] < tope and not completo():
            lote = list(itertools.islice(it, min(cada - prog["vistas"] % cada, tope - prog["vistas"])))
            if not lote:
                break
            for fid, filas in (pool.map(_analizar, lote) if pool else map(_analizar, lote)):
                prog["vistas"] += 1
                if filas is not None:
                    prog["validas"] += 1
                    for fila in filas:
                        out.write((json.dumps(fila, ensure_ascii=False) + "\n").encode())
                        prog["conteo"][fila["etiqueta"]] = prog["conteo"].get(fila["etiqueta"], 0) + 1
                if completo():
                    break
            if prog["vistas"] % cada == 0 or completo() or prog["vistas"] >= tope:
                guardar()
                print(f"{prog['vistas']} fórmulas, {prog['validas']} válidas, {sum(prog['conteo'].values())} "
                      f"símbolos, {prog['segundos']:.0f} s", file=sys.stderr, flush=True)
        guardar()
    finally:
        out.close()
        if pool:
            pool.close()
    return prog


def formulas_mw(dataset, split):
    """Fórmulas de `dataset/split/` que pasan el filtro, en el orden del hash: genera (id, latex, trazos)."""
    carpeta = os.path.join(dataset, split)
    ids = sorted((f[:-6] for f in os.listdir(carpeta) if f.endswith(".inkml")),
                 key=lambda i: (alinear.u("onepepe-extraccion-v1", i), i))
    for fid in ids:
        path = os.path.join(carpeta, fid + ".inkml")
        with open(path, encoding="utf-8") as f:
            head = f.read(2500)
        a = head.find('type="normalizedLabel">')
        if a < 0:
            continue
        latex = head[a + len('type="normalizedLabel">') : head.find("</annotation>", a)]
        if r"\begin" in latex:
            continue
        tokens = expected_symbols(latex)
        if not tokens or len(tokens) > MAX_FICHAS or not all(t in CORE for t in tokens):
            continue
        _, strokes = read_inkml(path)
        if formula_valida(tokens, len(strokes), CORE):
            yield fid, latex, strokes


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--split", default="train", choices=["train", "valid"])
    ap.add_argument("--modelo", default=None)
    ap.add_argument("--tope", type=int, default=None, help="fórmulas (train 30000, valid 3000)")
    ap.add_argument("--por-clase", type=int, default=None, help="símbolos por clase (train 400; valid sin parar)")
    ap.add_argument("--cada", type=int, default=1000)
    ap.add_argument("--procesos", type=int, default=1)
    ap.add_argument("--salida", default=None)
    a = ap.parse_args()
    valid = a.split == "valid"
    data = os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe"))
    dataset = os.environ.get("ONEPEPE_MATHWRITING", os.path.expanduser("~/.local/share/onepepe/mathwriting/mathwriting-2024"))
    salida = a.salida or os.path.join(data, "mathwriting-extraido", "simbolos_valid.jsonl" if valid else "simbolos.jsonl")
    clf = SymbolClassifier(a.modelo or os.path.join(data, "math", "simbolos.npz"))
    r = extraer(formulas_mw(dataset, a.split), clf, salida, por_clase=a.por_clase or (math.inf if valid else 400),
                tope=a.tope or (3000 if valid else 30000), cada=a.cada, procesos=a.procesos)
    print(json.dumps({k: r[k] for k in ("vistas", "validas", "segundos")}), sum(r["conteo"].values()), "símbolos")


if __name__ == "__main__":
    main()
