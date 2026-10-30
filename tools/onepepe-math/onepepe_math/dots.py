# SPDX-License-Identifier: GPL-3.0-or-later
"""La i y la j: un punto encima de un palo corto.

La red las confunde, entre otras cosas, porque el punto se corta aparte (sale «.» o «·»). Después del corte en
símbolos, un punto suelto justo encima de un palo corto se une con él y el grupo se vuelve a clasificar.

Medido con 400 fórmulas de MathWriting con i o j (símbolos reservados): i bien 141 → 147 de 358, j igual (31 de
105), el resto igual. Una segunda regla, «la j baja de la línea base y la i no», empeoraba la i (147 → 88 con la
línea base del renglón, 147 → 123 con la de los vecinos: la i de un subíndice también baja) y se quitó. Ver
DECISIONES.md.
"""

from .segment import bbox

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
