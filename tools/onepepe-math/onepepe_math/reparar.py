# SPDX-License-Identifier: GPL-3.0-or-later
"""Reparación del corte: la i y la j (punto cortado aparte) y, en general, trazos sobrantes (M5.4).

La i y la j: un punto encima de un palo corto.

La red las confunde, entre otras cosas, porque el punto se corta aparte (sale «.» o «·»). Después del corte en
símbolos, un punto suelto justo encima de un palo corto se une con él y el grupo se vuelve a clasificar.

Medido con 400 fórmulas de MathWriting con i o j (símbolos reservados): i bien 141 → 147 de 358, j igual (31 de
105), el resto igual. Una segunda regla, «la j baja de la línea base y la i no», empeoraba la i (147 → 88 con la
línea base del renglón, 147 → 123 con la de los vecinos: la i de un subíndice también baja) y se quitó. Ver
DECISIONES.md.
"""

import math

from .segment import MAX_STROKES, SegmentParams, bbox

DOTS = {".", r"\cdot", "'", ","}
# lo que puede ser el palo de una i o una j cuando el punto se cortó aparte
STEMS = {"1", "|", "l", "i", "j", "I", "J", ";", "!", "t", r"\iota", "/", ")", "("}


def _top(cands):
    return cands[0][0] if cands else None


def fix_dotted_letters(syms, strokes, classifier, typical):
    """Une los puntos sueltos con su palo en la salida de `segment` (lista de {"strokes", "candidates", "junk"})."""
    syms = [dict(s) for s in syms]
    boxes = [bbox([strokes[i] for i in s["strokes"]]) for s in syms]
    used = set()
    for d, (dsym, dbox) in enumerate(zip(syms, boxes)):
        dw, dh = dbox[2] - dbox[0], dbox[3] - dbox[1]
        is_dot = _top(dsym["candidates"]) in DOTS or max(dw, dh) < 0.3 * typical
        if not is_dot or max(dw, dh) > 0.45 * typical or d in used:
            continue
        dcx = (dbox[0] + dbox[2]) / 2
        best = None
        for s, (ssym, sbox) in enumerate(zip(syms, boxes)):
            if s == d or s in used or _top(ssym["candidates"]) not in STEMS:
                continue
            sh = sbox[3] - sbox[1]
            gap = sbox[1] - dbox[3]
            if (
                sbox[0] - 0.35 * typical <= dcx <= sbox[2] + 0.35 * typical
                and -0.15 * typical <= gap < 0.9 * typical
                and 0.3 * typical < sh < 1.6 * typical
                and (best is None or gap < best[1])
            ):
                best = (s, gap)
        if best is None:
            continue
        s = best[0]
        group = sorted(syms[s]["strokes"] + dsym["strokes"])
        cands, junk = classifier.candidates_w_junk([strokes[i] for i in group], 8)
        p = dict(cands)
        # solo si la red ve también una i o una j en el grupo: si no, se deja como estaba
        if p.get("i", 0.0) + p.get("j", 0.0) < 0.1:
            continue
        syms[s] = {"strokes": group, "candidates": cands, "junk": junk}
        boxes[s] = bbox([strokes[i] for i in group])
        used.update((s, d))
        syms[d] = None
    syms = [s for s in syms if s is not None]
    syms.sort(key=lambda s: s["strokes"][0])
    return syms


# M5.4: lo que puede sobrar (puntos, rayitas) y lo que puede resultar de unirlo
SOBRANTES = {".", r"\cdot", "'", ",", "-", "|", "1", "l", "/", ")", "("}
UNIDOS = {
    "=", r"\leq", r"\geq", r"\neq", r"\div", r"\pm", r"\equiv", r"\approx", "i", "j", "!", ":", ";", "t", "7",
    "H", "x", "+", "k", "f", "A", "F", "E", "T", r"\pi", "z", "Z", "y", "4", "5", r"\theta", r"\times", "*", r"\in",
}


def _gaps(a, b):
    return max(0.0, max(a[0], b[0]) - min(a[2], b[2])), max(0.0, max(a[1], b[1]) - min(a[3], b[3]))


def reparar(syms, strokes, classifier, typical, params=None):
    """Une trazos sobrantes con su pareja si la lectura unida es mucho más probable que las dos por separado."""
    params = params or SegmentParams()
    syms = [dict(s) for s in syms]
    memo = {}

    def lee(idx):
        k = tuple(idx)
        if k not in memo:
            memo[k] = classifier.candidates_w_junk([strokes[i] for i in idx], 8)
        return memo[k]

    def score(idx):
        cands, junk = lee(idx)
        return math.log(max(cands[0][1], 1e-6)) + math.log(max(1.0 - junk, 1e-6)) - params.symbol_penalty

    for _ in range(10):
        boxes = [bbox([strokes[i] for i in s["strokes"]]) for s in syms]
        best = None
        for d, dsym in enumerate(syms):
            if len(dsym["strokes"]) > 2:
                continue
            cands, junk = lee(dsym["strokes"])
            db = boxes[d]
            if not (cands[0][0] in SOBRANTES or junk >= 0.5 or max(db[2] - db[0], db[3] - db[1]) < 0.3 * typical):
                continue
            for s, ssym in enumerate(syms):
                if s == d:
                    continue
                union = sorted(dsym["strokes"] + ssym["strokes"])
                gx, gy = _gaps(db, boxes[s])
                overlap_x = min(db[2], boxes[s][2]) - max(db[0], boxes[s][0])
                near = (gx < 0.25 * typical and gy < 0.25 * typical) or (overlap_x > 0 and gy < 0.6 * typical)
                between = union[-1] - union[0] + 1 - len(union)
                if not near or between > 5 or len(union) > MAX_STROKES:
                    continue
                gain = score(union) - score(dsym["strokes"]) - score(ssym["strokes"])
                if gain > 0.5 and lee(union)[0][0][0] in UNIDOS and (best is None or gain > best[0]):
                    best = (gain, d, s, union)
        if best is None:
            break
        _, d, s, union = best
        cands, junk = lee(union)
        syms[s] = {"strokes": union, "candidates": cands, "junk": junk}
        del syms[d]
    syms.sort(key=lambda s: s["strokes"][0])
    return fix_dotted_letters(syms, strokes, classifier, typical)
