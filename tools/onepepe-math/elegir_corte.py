#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Elección de la penalización λ y de las reglas del corte (ARQUITECTURA-EJECUCION.md §6 M5.2).

Rejilla λ ∈ {0; 0,1; 0,2; 0,3} × variante V0..V3 (16 casos) más el caso actual (V0, λ = 1,2). Error de un caso:
E = ½·E_propio (medida cruzada) + ½·E_sint (300 fórmulas sintéticas con letra no vista), con E = 100 − % corte exacto.
Restricción: partidos ≤ partidos del caso actual + 0,5 en las dos partes. Escribe $ONEPEPE_DATA/math/eleccion_corte.json.

    elegir_corte.py --sesiones "$ENTRENOS" --cruzada v1
"""

import argparse
import json
import os
import sys
from argparse import Namespace

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import medir as M  # noqa: E402
from onepepe_math import alinear, synth  # noqa: E402
from onepepe_math.classifier import SymbolClassifier  # noqa: E402
from onepepe_math.reparar import reparar
from onepepe_math.segment import SegmentParams, segment, typical_size  # noqa: E402

LAMBDAS = (0.0, 0.1, 0.2, 0.3)
# reglas de cada variante (se acumulan); bar_role fuera: no cambia el corte
VARIANTES = {
    0: {},
    1: {"bars_first": 1.0},
    2: {"bars_first": 1.0, "stack_margin": 0.0},
    3: {"bars_first": 1.0, "stack_margin": 0.0, "stack_gap": 0.6},
}
ACTUAL = (0, 1.2)


def params(v, lam, repair):
    return SegmentParams(symbol_penalty=lam, repair=repair, bar_role=0.0, **VARIANTES[v])


def sintetico(formulas, clf, sp):
    """(% corte exacto, % partido) del corte solo (sin red de contexto) sobre fórmulas con verdad conocida."""
    cuenta = {"exacto": 0, "partido": 0, "n": 0}
    for f in formulas:
        syms = segment(f["strokes"], clf, sp)
        if sp.repair > 0:
            syms = reparar(syms, f["strokes"], clf, typical_size(f["strokes"]), sp)
        pred = [s["strokes"] for s in syms]
        for e in M.estados([idx for _, idx in f["truth"]], pred):
            cuenta["n"] += 1
            cuenta[e] = cuenta.get(e, 0) + 1
    n = cuenta["n"]
    return 100 * cuenta["exacto"] / n, 100 * cuenta["partido"] / n, n


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--sesiones", required=True)
    ap.add_argument("--cruzada", required=True, metavar="RECETA")
    ap.add_argument("--conjunto", default="entreno")
    ap.add_argument("--repair", type=float, default=1.0, help="0 = la rejilla sin la reparación general")
    ap.add_argument("--formulas", type=int, default=300)
    a = ap.parse_args()
    datos = os.path.dirname(M.DATOS)
    carpeta = os.path.join(datos, "medidas", "t5-rejilla")
    os.makedirs(carpeta, exist_ok=True)

    conj = M.conjuntos()
    lineas = [l for l in M.cargar_renglones(a.sesiones) if M.en_conjunto(l, a.conjunto, conj)]
    ruta = os.path.join(M.DATOS, "referencia.json")
    congelada = json.load(open(ruta, encoding="utf-8")) if os.path.exists(ruta) else None
    modelos = {}

    def clf_de(l):
        k = alinear.pliegue(f"{l['set']}#{l['id']}")
        if k not in modelos:
            modelos[k] = M.Cache(SymbolClassifier(os.path.join(datos, "pliegues", a.cruzada, f"pliegue{k}.npz")))
        return modelos[k], None

    clf = M.Cache(SymbolClassifier())
    formulas = synth.formulas(os.environ["ONEPEPE_MATHWRITING"], a.formulas, seed=11, holdout=True)
    args = Namespace(incremental=None, ref="O", sin_variantes=False)

    casos = {}
    for v, lam in [ACTUAL] + [(v, lam) for v in VARIANTES for lam in LAMBDAS]:
        sp = params(v, lam, a.repair)
        out = M.medir(lineas, clf_de, args, sp, congelada)
        nombre = f"V{v}_l{lam}"
        archivo = os.path.join(carpeta, nombre + ".json")
        json.dump(out, open(archivo, "w", encoding="utf-8"), indent=1)
        m = out["metricas"]
        ex_s, pa_s, n_s = sintetico(formulas, clf, sp)
        casos[(v, lam)] = c = {
            "variante": v, "lambda": lam, "archivo": archivo,
            "propio": {k: m[k]["pct"] for k in ("corte_exacto", "juntado", "partido", "raya_corte")},
            "sint": {"corte_exacto": ex_s, "partido": pa_s, "n": n_s},
        }
        c["E"] = 0.5 * (100 - c["propio"]["corte_exacto"]) + 0.5 * (100 - ex_s)
        print(nombre, json.dumps(c, ensure_ascii=False), flush=True)

    base = casos[ACTUAL]
    ok = [c for k, c in casos.items() if k != ACTUAL
          and c["propio"]["partido"] <= base["propio"]["partido"] + 0.5
          and c["sint"]["partido"] <= base["sint"]["partido"] + 0.5]
    gana = None
    if ok:
        emin = min(c["E"] for c in ok)
        empate = [c for c in ok if c["E"] - emin < 0.3]
        gana = min(empate, key=lambda c: (c["variante"], -c["lambda"]))
        gana["comparacion"] = M.comparar(base["archivo"], gana["archivo"])
        # M0.6: se adopta si mejora E y la medida propia no empeora
        gana["adoptado"] = gana["E"] < base["E"] and gana["comparacion"]["veredicto"] != "empeora"
    res = {"actual": base, "casos": list(casos.values()), "ganador": gana,
           "defecto": {"variante": gana["variante"], "lambda": gana["lambda"]} if gana and gana["adoptado"] else None}
    json.dump(res, open(os.path.join(M.DATOS, "eleccion_corte.json"), "w", encoding="utf-8"), indent=1)
    print("GANADOR", json.dumps(gana, ensure_ascii=False))


if __name__ == "__main__":
    main()
