# SPDX-License-Identifier: GPL-3.0-or-later
"""El reconocedor completo: trazos -> símbolos con candidatas -> símbolos limpios, LaTeX y resultado del "="."""

from . import glossary as gl
from . import glyphs
from .personal import D_MAX
from .context import apply_glossary, reading_order, share_within_groups, shape_groups, viterbi
from .evaluate import answer
from .layout import Sym, bar_role, parse, to_latex
from .haz import mejor_lectura
from .lexico import corregir_entidades
from .symbols import same_shape
from .reparar import fix_dotted_letters, reparar
from .segment import SegmentParams, bbox, linea_de, segment, typical_size


_RAYAS = {"-": "minus", r"\frac": "frac", r"\overline": "overline"}
_ETIQUETA = {"minus": "-", "frac": r"\frac", "overline": r"\overline"}


def _raya_con_su_papel(cand_lists, boxes, typical):
    """M5.5: la probabilidad de «-», «\\frac» y «\\overline» pasa a la etiqueta que le toca por su sitio."""
    out = []
    for i, cands in enumerate(cand_lists):
        if cands[0][0] not in _RAYAS:
            out.append(cands)
            continue
        otras = [Sym(c[0][0], b) for j, (c, b) in enumerate(zip(cand_lists, boxes)) if j != i]
        papel = _ETIQUETA[bar_role(boxes[i], otras, typical)]
        total = sum(p for c, p in cands if c in _RAYAS)
        out.append([(papel, total)] + [(c, p) for c, p in cands if c not in _RAYAS])
    return out


def _misma(a, b):
    return same_shape(a, b) or (a in _RAYAS and b in _RAYAS)


def recognize(
    strokes,
    classifier,
    personal=None,
    known=(),
    glossary_names=(),
    units=None,
    threshold=0.95,
    clean=True,
    equals=True,
    segment_params=None,
    times=None,
    umbral_min=0.0,
):
    """Reconoce una línea de matemáticas.

    `strokes`: trazos nuevos (listas de (x, y)) en orden de escritura. `known`: símbolos ya reconocidos antes en esa
    línea, como [{"latex", "box"}], que dan contexto. Devuelve un dict con "symbols", "latex", "result".
    """
    segment_params = segment_params or SegmentParams()
    cajas = [tuple(k["box"]) for k in known]
    syms = segment(strokes, classifier, params=segment_params, times=times, known_boxes=cajas)
    if syms:
        ln = linea_de(strokes, classifier, cajas)  # solo los modelos v2 miran el renglón
        if segment_params.repair > 0:
            syms = reparar(syms, strokes, classifier, typical_size(strokes), segment_params, linea=ln)
        else:
            syms = fix_dotted_letters(syms, strokes, classifier, typical_size(strokes), linea=ln)
    y_red = [s["candidates"][0][0] for s in syms]  # lo que dice la red sola (M8.2)
    groups_strokes = [[strokes[i] for i in s["strokes"]] for s in syms]
    boxes = [bbox(g) for g in groups_strokes]
    cand_lists = [s["candidates"] for s in syms]
    embeddings = [classifier.embedding(g) for g in groups_strokes]
    y_knn = [None] * len(syms)
    if personal is not None and len(personal):
        for i, e in enumerate(embeddings):
            cerca = [lab for d, lab, _ in personal.neighbors(e) if d <= D_MAX]
            y_knn[i] = cerca[0] if cerca else None
    if personal is not None and len(personal):
        cand_lists = [personal.adjust(g, c, embedding=e) for g, c, e in zip(groups_strokes, cand_lists, embeddings)]
    labels = gl.symbol_labels(glossary_names)
    if labels:
        cand_lists = [apply_glossary(c, labels) for c in cand_lists]
    # los que se parecen mucho entre sí comparten etiqueta
    groups = shape_groups(embeddings) if syms else []
    cand_lists = share_within_groups(cand_lists, groups)

    if segment_params.bar_role > 0 and syms:
        cand_lists = _raya_con_su_papel(cand_lists, boxes, typical_size(strokes))

    # contexto: todos los símbolos de la línea en orden de lectura (los ya conocidos, fijos)
    all_c = cand_lists + [[(k["latex"], 1.0)] for k in known]
    all_b = boxes + [tuple(k["box"]) for k in known]
    order = reading_order(all_c, all_b, segment_params.orden_arbol > 0)
    lm = getattr(classifier, "lm", None)
    if lm is not None and all_c:
        ranked = viterbi([all_c[i] for i in order], lm)
        for pos, i in enumerate(order):
            all_c[i] = ranked[pos]
    if segment_params.entidades > 0 and all_c:
        all_c = corregir_entidades(all_c, all_b, sorted(b[3] - b[1] for b in all_b)[len(all_b) // 2],
                                   range(len(syms), len(all_c)), segment_params.theta)
    if segment_params.mejor_lectura > 0 and all_c:  # M10: la lectura entera mejor; la elegida pasa a ser la primera
        sel = mejor_lectura(all_c, all_b, range(len(syms), len(all_c)), segment_params.premio)
        all_c = [c if s == 0 else [c[s]] + c[:s] + c[s + 1 :] for c, s in zip(all_c, sel)]
    cand_lists = all_c[: len(syms)]

    umbral = max(threshold, umbral_min)
    out_symbols = []
    for s, cands, box, g, yr, yk in zip(syms, cand_lists, boxes, groups, y_red, y_knn):
        top, p = cands[0]
        confidence = p * (1.0 - s.get("junk", 0.0))
        item = {
            "strokes": s["strokes"],
            "box": list(box),
            "candidates": [{"latex": c, "p": round(q, 4)} for c, q in cands[:6]],
            "confidence": round(confidence, 4),
            "group": g,
            "clean": None,
            # M8.2: confianza suficiente, la red coincide y el kNN no contradice
            "sure": confidence >= umbral and _misma(yr, top) and (yk is None or _misma(yk, top)),
        }
        if clean and item["sure"]:
            lines = glyphs.place_glyph(top, box)
            if lines is not None:
                item["clean"] = [[[round(x, 2), round(y, 2)] for x, y in line] for line in lines]
        out_symbols.append(item)

    # con sus otras lecturas, para unir funciones mal leídas («1n» → ln)
    layout_syms = [Sym(c[0][0], b, tuple(x for x, _ in c[:5])) for c, b in zip(cand_lists, boxes)] + [
        Sym(k["latex"], tuple(k["box"]), tuple(k.get("alts", ()))) for k in known
    ]
    tree = parse(layout_syms)
    result = answer(tree, units=units) if equals else None
    confidence = min([s["confidence"] for s in out_symbols] + [1.0])
    return {"symbols": out_symbols, "latex": to_latex(tree), "result": result, "confidence": round(confidence, 4)}
