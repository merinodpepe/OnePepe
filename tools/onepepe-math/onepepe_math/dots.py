# SPDX-License-Identifier: GPL-3.0-or-later
"""La i y la j: un punto encima de un palo corto.

La red las confunde porque el punto se corta aparte (sale «.» o «·») o porque la j es casi una i con cola. Dos
reglas, después del corte en símbolos:

1. Un punto suelto justo encima de un palo corto se une con él y el grupo se vuelve a clasificar.
2. Entre i y j decide la línea base del renglón: la j baja por debajo, la i no.

Mediciones y causa en PLAN-PENDIENTE.md §2, punto 4.
"""

from .segment import bbox

DOTS = {".", r"\cdot", "'", ","}
# lo que puede ser el palo de una i o una j cuando el punto se cortó aparte
STEMS = {"1", "|", "l", "i", "j", "I", "J", ";", "!", "t", r"\iota", "/", ")", "("}
# símbolos cuya parte de abajo marca la línea base (no bajan de ella ni son planos)
_BASELINE = set("0123456789acemnorsuvwxzABCDEFGHIKLMNOPQRSTUVWXYZbdhklt")


def _top(cands):
    return cands[0][0] if cands else None


def _baseline(boxes_labels, typical):
    """Línea base del renglón: la mediana de la parte de abajo de las letras y cifras que no bajan de ella."""
    bottoms = sorted(b[3] for b, lab in boxes_labels if lab in _BASELINE)
    if not bottoms:
        return None
    return bottoms[(len(bottoms) - 1) // 2]


def _i_or_j(cands, box, baseline, typical):
    """Ordena las candidatas para que i o j vaya primero según baje o no de la línea base."""
    p = dict(cands)
    if baseline is None or not ({"i", "j"} & set(list(p)[:2])):
        return cands
    below = box[3] - baseline
    if below > 0.25 * typical:
        winner = "j"
    elif below < 0.1 * typical:
        winner = "i"
    else:
        return cands
    loser = "i" if winner == "j" else "j"
    mass = p.get("i", 0.0) + p.get("j", 0.0)
    rest = [(c, q) for c, q in cands if c not in ("i", "j")]
    out = [(winner, max(mass, p.get(winner, 0.0)))] + rest
    if loser in p:
        out.append((loser, min(p[loser], 0.05)))
    return sorted(out, key=lambda c: -c[1])


def fix_dotted_letters(syms, strokes, classifier, typical, known=()):
    """Aplica las dos reglas a la salida de `segment` (lista de {"strokes", "candidates", "junk"}).

    `known`: símbolos ya reconocidos del renglón ({"latex", "box"}), que ayudan a situar la línea base.
    """
    syms = [dict(s) for s in syms]
    boxes = [bbox([strokes[i] for i in s["strokes"]]) for s in syms]
    # 1. unir el punto suelto con el palo de debajo
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
    keep = [i for i, s in enumerate(syms) if s is not None]
    syms = [syms[i] for i in keep]
    boxes = [boxes[i] for i in keep]
    # 2. i o j según la línea base
    labeled = [(b, _top(s["candidates"])) for s, b in zip(syms, boxes)]
    labeled += [(tuple(k["box"]), k["latex"]) for k in known]
    baseline = _baseline(labeled, typical)
    for s, box in zip(syms, boxes):
        s["candidates"] = _i_or_j(s["candidates"], box, baseline, typical)
    syms.sort(key=lambda s: s["strokes"][0])
    return syms
