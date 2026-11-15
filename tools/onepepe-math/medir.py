#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Banco de medida del reconocedor (ARQUITECTURA-EJECUCION.md §6 M0 y T0).

Lee las sesiones de entreno (`*.entreno.json`), alinea cada renglón con su LaTeX (referencia), lo reconoce y mide corte,
etiqueta y sustituciones. Los datos de trabajo salen de $ONEPEPE_DATA (copia de trabajo); las sesiones solo se leen.

    medir.py --sesiones "$ENTRENOS" --conjunto todo --ref O --sin-variantes --solo-sin-frac
    medir.py --sesiones "$ENTRENOS" --crear-examen
    medir.py --sondas
    medir.py --comparar a.json b.json
"""

import argparse
import dataclasses
import glob
import json
import math
import os
import random
import re
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "tests"))

from onepepe_math import alinear, confianza  # noqa: E402
from onepepe_math.classifier import SymbolClassifier  # noqa: E402
from onepepe_math.entreno import _split_by_guides, expected_symbols, list_sets, load_set  # noqa: E402
from onepepe_math.forced import forced_groups  # noqa: E402
from onepepe_math.personal import PersonalModel  # noqa: E402
from onepepe_math.recognizer import recognize  # noqa: E402
from onepepe_math.segment import SegmentParams, bbox, linea_de, typical_size  # noqa: E402
from onepepe_math.symbols import same_shape  # noqa: E402

DATOS = os.path.join(os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math")
DIG = set("0123456789")
Z90 = 1.645


# ---------------------------------------------------------------- carga

def cargar_renglones(directorio):
    """Los renglones escritos de todas las sesiones de `directorio` (reparte por renglones guía).
    [{set, id, li, latex, strokes, times, tokens, kind}]"""
    out = []
    for f in sorted(glob.glob(os.path.join(directorio, "*.entreno.json"))):
        with open(f, encoding="utf-8") as fh:
            d = json.load(fh)
        items = {it["id"]: it for it in d["items"]}
        for iid, a in d["answers"].items():
            it = items.get(iid)
            if it is None or a.get("skipped") or not a.get("strokes"):
                continue
            raw, times = a["strokes"], a.get("times")
            keep = [i for i, s in enumerate(raw) if s]
            strokes = [[(float(p[0]), float(p[1])) for p in raw[i]] for i in keep]
            tm = [tuple(times[i]) for i in keep] if times and len(times) == len(raw) else None
            grupos = _split_by_guides(strokes, a.get("size", [1.0, 1.0]), len(it["lines"]), a.get("line_height"))
            for li, (latex, idx) in enumerate(zip(it["lines"], grupos)):
                out.append(dict(set=d["set"], id=iid, li=li, latex=latex, strokes=[strokes[i] for i in idx],
                                times=[tm[i] for i in idx] if tm else None, tokens=expected_symbols(latex),
                                kind=it.get("kind")))
    return out


def cargar_fallos(datos=DATOS):
    """`fallos.jsonl` (si existe) como otro origen: una línea por renglón {id, latex, strokes, times?}."""
    ruta = os.path.join(datos, "fallos.jsonl")
    out = []
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            for n, linea in enumerate(f):
                d = json.loads(linea)
                st = [[(float(p[0]), float(p[1])) for p in s] for s in d["strokes"]]
                out.append(dict(set="fallos", id=str(d.get("id", n)), li=0, latex=d.get("latex") or d["expected"], strokes=st,
                                times=[tuple(t) for t in d["times"]] if d.get("times") else None,
                                tokens=expected_symbols(d.get("latex") or d["expected"]), kind="expr"))
    return out


def sets_del_repo():
    return {s["name"]: [(i["id"], i["kind"]) for i in load_set(s["name"])["items"]] for s in list_sets()}


def conjuntos(datos=DATOS):
    """{"examen": {set: [ids]}, "validacion": ...}: el de examen.json si existe; si no, el reparto por hash."""
    ruta = os.path.join(datos, "examen.json")
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            return json.load(f)
    return alinear.reparto(sets_del_repo())


def en_conjunto(l, nombre, conj):
    if nombre == "todo":
        return True
    if l["set"] in conj["examen"]:
        es = "examen" if l["id"] in conj["examen"][l["set"]] else (
            "validacion" if l["id"] in conj["validacion"].get(l["set"], ()) else "entreno")
    else:  # otro origen: examen si u < 0,20
        es = "examen" if alinear.u("onepepe-examen-v1", f"{l['set']}#{l['id']}") < 0.20 else "entreno"
    return es == nombre


# ---------------------------------------------------------------- referencia

def clave(l):
    return f"{l['set']}#{l['id']}#{l['li']}"


def referencia(l, clf, pm, modo, sin_variantes, congelada=None):
    """[(índices de trazos, ficha)] en orden de escritura, o None. Si hay referencia congelada vigente, esa."""
    n = len(l["strokes"])
    if congelada and not sin_variantes and clave(l) in congelada and congelada[clave(l)]["n_trazos"] == n:
        c = congelada[clave(l)]
        return None if c["grupos"] is None or c["dudoso"] else list(zip(c["grupos"], c["etiquetas"]))
    if modo == "F":
        return forced_groups(l["strokes"], l["tokens"], clf)
    r = alinear.ordered_groups(l["strokes"], l["tokens"], clf, latex=l["latex"], variantes=not sin_variantes,
                               tardios=not sin_variantes)
    return r and [(g, e) for g, e in r["grupos"]]


def info_ref(l, grupos):
    """Para cada símbolo de referencia: idx, label, n, box y, si es cifra, el número al que pertenece."""
    st, tip = l["strokes"], typical_size(l["strokes"])
    refs = [dict(idx=list(g), label=lab, n=len(g), box=bbox([st[i] for i in g]), num_id=None) for g, lab in grupos]
    i = 0
    while i < len(refs):
        j = i
        if refs[i]["label"] in DIG:
            while j + 1 < len(refs) and refs[j + 1]["label"] in DIG and _adj(refs[j]["box"], refs[j + 1]["box"], tip):
                j += 1
            if j > i:
                for k in range(i, j + 1):
                    refs[k]["num_id"] = i
        i = j + 1
    return refs


def _adj(a, b, tip):
    ca, cb = (a[1] + a[3]) / 2, (b[1] + b[3]) / 2
    h = max(a[3] - a[1], b[3] - b[1], 1e-6)
    return abs(ca - cb) < 0.4 * max(h, 0.5 * tip) and -0.5 * tip < b[0] - a[2] < 1.0 * tip


def estados(refs, pred_groups):
    """Estado de cada grupo de referencia (listas de índices) frente a lo leído: exacto, juntado, partido, mixto."""
    dueno = {i: p for p, g in enumerate(pred_groups) for i in g}
    out = []
    for idx in refs:
        s, toca = set(idx), {dueno[i] for i in idx if i in dueno}
        if len(toca) == 1:
            out.append("exacto" if set(pred_groups[next(iter(toca))]) == s else "juntado")
        else:
            out.append("mixto" if any(not set(pred_groups[t]) <= s for t in toca) else "partido")
    return out


# ---------------------------------------------------------------- lectura

class Cache:
    """Clasificador con caché por identidad de trazos (el alineado y la lectura comparten pasadas por la red)."""

    def __init__(self, clf):
        self._c, self._m = clf, {}

    def __getattr__(self, n):
        return getattr(self._c, n)

    def _get(self, nombre, strokes, *a, **kw):
        k = (nombre, tuple(id(s) for s in strokes), a, tuple(kw.items()))
        if k not in self._m:
            self._m[k] = getattr(self._c, nombre)(strokes, *a, **kw)
        return self._m[k]

    def candidates_w_junk(self, strokes, k=5, **kw):
        return self._get("candidates_w_junk", strokes, k, **kw)

    def probabilities(self, strokes, **kw):
        return self._get("probabilities", strokes, **kw)

    def embedding(self, strokes):
        return self._get("embedding", strokes)


def _simbolo(r, idx=None):
    return dict(strokes=idx if idx is not None else r["strokes"], top=r["candidates"][0]["latex"],
                alts=[c["latex"] for c in r["candidates"][1:5]], box=r["box"], clean=r["clean"] is not None,
                c=r["confidence"], sure=r["sure"])


def leer(l, clf, pm, sp, pausa=None, **kw):
    """(símbolos leídos [{strokes, top, clean}], LaTeX leído). `pausa`: lotes por pausa (s), como la app."""
    if pausa is None:
        r = recognize(l["strokes"], clf, personal=pm, segment_params=sp, times=l["times"], equals=False, **kw)
        return [_simbolo(s) for s in r["symbols"]], r["latex"]
    tm, lotes = l["times"], [[0]]
    for i in range(1, len(tm)):
        if tm[i][0] - tm[i - 1][1] > pausa:
            lotes.append([i])
        else:
            lotes[-1].append(i)
    conocidos, syms, r = [], [], None
    for b in lotes:
        r = recognize([l["strokes"][i] for i in b], clf, personal=pm, known=conocidos, segment_params=sp,
                      times=[tm[i] for i in b], equals=False, **kw)
        for s in r["symbols"]:
            syms.append(_simbolo(s, [b[i] for i in s["strokes"]]))
            conocidos.append({"latex": syms[-1]["top"], "box": s["box"], "alts": syms[-1]["alts"]})
    return syms, r["latex"]


def norm(s):
    for a in (r"\left", r"\right", r"\,", r"\quad", r"\mathrm", r"\cdot", r"\ "):
        s = s.replace(a, "")
    return re.sub(r"[\s{}]", "", s)


# ---------------------------------------------------------------- métricas

def wilson(k, n):
    if not n:
        return None
    p, z2 = k / n, Z90**2
    c = (p + z2 / (2 * n)) / (1 + z2 / n)
    h = Z90 * math.sqrt(p * (1 - p) / n + z2 / (4 * n * n)) / (1 + z2 / n)
    return [round(100 * (c - h), 1), round(100 * (c + h), 1)]


def _m(k, n):
    return {"pct": round(100 * k / n, 1) if n else None, "k": k, "n": n, "ic90": wilson(k, n)}


def medir(lineas, clf_de, args, sp, congelada=None):
    """Lee y puntúa los renglones. `clf_de(l)` -> (clasificador, personal). Devuelve el dict de resultados."""
    cuenta = {k: [0, 0] for k in (
        "corte_exacto", "juntado", "partido", "mixto", "corte_y_etiqueta", "renglon_perfecto", "numeros_2mas",
        "raya_papel", "raya_corte", "raya_corte_etiqueta", "sust_erroneas", "sust_erroneas_visibles",
        "sust_cobertura", "latex_exacto", "red_top1", "red_top5")}
    por_expr, ms = {}, []
    sim_pred, ren_pred = [], []  # para calibrar_umbral.py: [c, acuerdo, ok] por símbolo y [mín c, perfecto] por renglón
    # --umbral U: umbral fijo y sin el de confianza.json (U = 0 mide todo, para calibrar)
    u = getattr(args, "umbral", None)
    kw = dict(threshold=u, umbral_min=0.0) if u is not None else dict(umbral_min=confianza.cargar().get("umbral", 0.0))

    def suma(k, ok, n=1):
        cuenta[k][0] += ok
        cuenta[k][1] += n

    for l in lineas:
        clf, pm = clf_de(l)
        t0 = time.perf_counter()
        syms, latex = leer(l, clf, pm, sp, args.incremental, **kw)
        ms.append(1000 * (time.perf_counter() - t0))
        suma("latex_exacto", norm(latex) == norm(l["latex"]))
        # los modelos v2 miran el renglón también en la medida de la red sola
        ln = linea_de(l["strokes"], clf)
        kw_ln = {"linea": ln} if ln else {}
        grupos = referencia(l, clf, pm, args.ref, args.sin_variantes, congelada)
        if grupos is None:
            continue
        refs = info_ref(l, grupos)
        pg = [s["strokes"] for s in syms]
        for r, e in zip(refs, estados([r["idx"] for r in refs], pg)):
            r["estado"] = e
            cuenta["corte_exacto" if e == "exacto" else e][0] += 1
            dueno = next(s for s in syms if r["idx"][0] in s["strokes"])
            r["pred"] = dueno["top"] if e == "exacto" else None
            r["ok"] = e == "exacto" and r["pred"] == r["label"]
            suma("corte_y_etiqueta", r["ok"])
            if r["label"] == r"\frac":
                suma("raya_corte", e == "exacto")
                suma("raya_corte_etiqueta", r["ok"])
                suma("raya_papel", e == "exacto" and r["pred"] in ("-", r"\frac")
                     and latex.count(r"\frac") == l["tokens"].count(r"\frac"))
            ps = clf.probabilities([l["strokes"][i] for i in r["idx"]], **kw_ln)
            por_lab = {}
            for lab, p in zip(getattr(clf, "labels", clf.classes), ps):
                por_lab[lab] = por_lab.get(lab, 0.0) + float(p)
            orden = sorted(por_lab, key=lambda x: -por_lab[x])
            suma("red_top1", orden[0] == r["label"])
            suma("red_top5", r["label"] in orden[:5])
        suma("renglon_perfecto", all(r["ok"] for r in refs))
        ren_pred.append([min([s["c"] for s in syms] + [1.0]), all(r["ok"] for r in refs)])
        numeros = {}
        for r in refs:
            if r["num_id"] is not None:
                numeros.setdefault(r["num_id"], []).append(r["ok"])
        for v in numeros.values():
            suma("numeros_2mas", all(v))
        dueno_ref = {i: r for r in refs for i in r["idx"]}
        for s in syms:
            r = dueno_ref.get(s["strokes"][0])
            sim_pred.append([s["c"], s["sure"], bool(r and r["estado"] == "exacto" and set(r["idx"]) == set(s["strokes"])
                                                     and s["top"] == r["label"])])
            suma("sust_cobertura", s["clean"])
            if s["clean"]:
                r = dueno_ref[s["strokes"][0]]
                exacto = r["estado"] == "exacto" and set(r["idx"]) == set(s["strokes"])
                suma("sust_erroneas", not (exacto and s["top"] == r["label"]), 1)
                suma("sust_erroneas_visibles", not (exacto and same_shape(s["top"], r["label"])), 1)
        e = por_expr.setdefault(f"{l['set']}#{l['id']}", [0, 0])
        e[0] += sum(r["ok"] for r in refs)
        e[1] += len(refs)

    n_refs = cuenta["corte_y_etiqueta"][1]
    for k in ("corte_exacto", "juntado", "partido", "mixto", "red_top1", "red_top5"):
        cuenta[k][1] = n_refs
    res = {k: _m(*v) for k, v in cuenta.items()}
    res["ms_por_renglon"] = round(sum(ms) / len(ms), 1) if ms else None
    return {"metricas": res, "por_expresion": {k: {"k": a, "n": b} for k, (a, b) in por_expr.items()},
            "renglones": len(lineas), "simbolos_pred": sim_pred, "renglones_pred": ren_pred}


# ---------------------------------------------------------------- examen y referencia congelada

def crear_examen(lineas, clf, pm, datos=DATOS):
    """Escribe examen.json (si no existe), referencia.json (añade lo que falta) y examen_trazos.json."""
    os.makedirs(datos, exist_ok=True)
    ruta = os.path.join(datos, "examen.json")
    conj = alinear.reparto(sets_del_repo())
    if os.path.exists(ruta):
        conj = json.load(open(ruta, encoding="utf-8"))
    else:
        json.dump(conj, open(ruta, "w", encoding="utf-8"), indent=1)
    for l in lineas:  # otro origen (fallos): examen si u < 0,20; solo se añaden ids nuevos
        if l["set"] not in sets_del_repo() and alinear.u("onepepe-examen-v1", f"{l['set']}#{l['id']}") < 0.20:
            ids = conj["examen"].setdefault(l["set"], [])
            if l["id"] not in ids:
                ids.append(l["id"])
    json.dump(conj, open(ruta, "w", encoding="utf-8"), indent=1)
    rutaref = os.path.join(datos, "referencia.json")
    ref = json.load(open(rutaref, encoding="utf-8")) if os.path.exists(rutaref) else {}
    for l in lineas:
        k = clave(l)
        if k in ref and ref[k]["n_trazos"] == len(l["strokes"]):
            continue
        r = alinear.ordered_groups(l["strokes"], l["tokens"], clf, latex=l["latex"], tardios=True)
        dudoso = r is None or not all(alinear.aceptados(r, l["strokes"], clf, pm))
        ref[k] = {"n_trazos": len(l["strokes"]), "dudoso": dudoso,
                  "grupos": r and [g for g, _ in r["grupos"]], "etiquetas": r and [e for _, e in r["grupos"]]}
    json.dump(ref, open(rutaref, "w", encoding="utf-8"), indent=1)
    sel = [l for l in lineas if l["id"] in conj["examen"].get(l["set"], ())]
    trazos = {clave(l): dict(l, strokes=[[list(p) for p in s] for s in l["strokes"]], referencia=ref[clave(l)])
              for l in sel}
    json.dump(trazos, open(os.path.join(datos, "examen_trazos.json"), "w", encoding="utf-8"))
    n_ids = sum(len(v) for v in conj["examen"].values())
    escritos = {(l["set"], l["id"]) for l in sel}
    return n_ids, len(escritos), len(ref), sum(r["dudoso"] for r in ref.values())


# ---------------------------------------------------------------- comparar y sondas

def comparar(a, b, remuestreos=2000):
    """M0.6: P = fracción de remuestreos de expresiones con (aciertos de B − aciertos de A) > 0."""
    ea, eb = (json.load(open(f, encoding="utf-8"))["por_expresion"] for f in (a, b))
    delta = [eb[k]["k"] - ea[k]["k"] for k in sorted(set(ea) & set(eb))]
    rng = random.Random(7)
    p = sum(sum(rng.choice(delta) for _ in delta) > 0 for _ in range(remuestreos)) / remuestreos
    veredicto = "mejora" if p >= 0.80 else "empeora" if p <= 0.20 else "empate"
    return {"P": p, "veredicto": veredicto, "expresiones": len(delta), "delta_total": sum(delta)}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--sesiones")
    ap.add_argument("--conjunto", choices=["examen", "validacion", "entreno", "todo"], default="examen")
    ap.add_argument("--crear-examen", action="store_true")
    ap.add_argument("--cruzada", metavar="RECETA", help="modelos en $ONEPEPE_DATA/pliegues/RECETA/pliegue<k>.npz")
    ap.add_argument("--incremental", type=float, metavar="PAUSA")
    ap.add_argument("--sin-knn", action="store_true")
    ap.add_argument("--umbral", type=float, help="umbral de sustitución fijo (0 para calibrar); sin él, confianza.json")
    ap.add_argument("--modelo")
    ap.add_argument("--ref", choices=["O", "F"], default="O")
    ap.add_argument("--sin-variantes", action="store_true")
    ap.add_argument("--solo-sin-frac", action="store_true")
    ap.add_argument("--param", action="append", default=[], metavar="CLAVE=VALOR", help="parámetro del corte/reconocedor")
    ap.add_argument("--salida")
    ap.add_argument("--comparar", nargs=2, metavar=("A", "B"))
    ap.add_argument("--sondas", action="store_true")
    ap.add_argument("--fallos", action="store_true")
    args = ap.parse_args()

    if args.comparar:
        out = comparar(*args.comparar)
    elif args.sondas:
        from sondas_util import SONDAS, erroneas

        malas = erroneas()
        out = {"sondas_erroneas": len(malas), "n_sondas": len(SONDAS), "erroneas": malas}
    else:
        if not args.sesiones:
            ap.error("falta --sesiones")
        lineas = cargar_renglones(args.sesiones) + (cargar_fallos() if args.fallos else [])
        clf = Cache(SymbolClassifier(args.modelo) if args.modelo else SymbolClassifier())
        pm = None if args.sin_knn else PersonalModel(clf, background=False)
        if args.crear_examen:
            n_ids, escritos, n_ref, dudosos = crear_examen(lineas, clf, pm)
            out = {"examen_ids": n_ids, "examen_escritos": escritos, "referencia": n_ref, "dudosos": dudosos}
        else:
            conj = conjuntos()
            lineas = [l for l in lineas if en_conjunto(l, args.conjunto, conj)
                      and not (args.solo_sin_frac and r"\frac" in l["tokens"])]
            ruta = os.path.join(DATOS, "referencia.json")
            congelada = json.load(open(ruta, encoding="utf-8")) if os.path.exists(ruta) else None
            if args.cruzada:
                if args.conjunto == "examen":
                    ap.error("--cruzada mide lo que no es examen")
                modelos = {}

                def clf_de(l):
                    k = alinear.pliegue(f"{l['set']}#{l['id']}")
                    if k not in modelos:
                        ruta_k = os.path.join(os.path.dirname(DATOS), "pliegues", args.cruzada, f"pliegue{k}.npz")
                        modelos[k] = Cache(SymbolClassifier(ruta_k))
                    return modelos[k], None
            else:
                def clf_de(l):
                    return clf, pm
            sp = SegmentParams.load()
            for kv in args.param:
                k, v = kv.split("=")
                sp = dataclasses.replace(sp, **{k: float(v)})
            out = medir(lineas, clf_de, args, sp, congelada)
    texto = json.dumps(out, ensure_ascii=False, indent=1)
    if args.salida:
        with open(args.salida, "w", encoding="utf-8") as f:
            f.write(texto)
    if "metricas" in out:
        for k, v in out["metricas"].items():
            print(f"{k}: {v['pct'] if isinstance(v, dict) else v}" + (f" (n={v['n']}, ic90 {v['ic90']})" if isinstance(v, dict) else ""))
        print("renglones:", out["renglones"])
    else:
        print(texto)


if __name__ == "__main__":
    main()
