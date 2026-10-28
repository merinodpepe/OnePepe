# SPDX-License-Identifier: GPL-3.0-or-later
"""El reconocedor completo: trazos -> símbolos con candidatas -> símbolos limpios, LaTeX y resultado del "="."""

from . import glossary as gl
from . import glyphs
from .context import apply_glossary, reading_order, share_within_groups, shape_groups, viterbi
from .evaluate import answer
from .layout import Sym, parse, to_latex
from .segment import bbox, segment


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
):
    """Reconoce una línea de matemáticas.

    `strokes`: trazos nuevos (listas de (x, y)) en orden de escritura. `known`: símbolos ya reconocidos antes en esa
    línea, como [{"latex", "box"}], que dan contexto. Devuelve un dict con "symbols", "latex", "result".
    """
    syms = segment(strokes, classifier, params=segment_params, times=times)
    groups_strokes = [[strokes[i] for i in s["strokes"]] for s in syms]
    boxes = [bbox(g) for g in groups_strokes]
    cand_lists = [s["candidates"] for s in syms]
    embeddings = [classifier.embedding(g) for g in groups_strokes]
    if personal is not None and len(personal):
        cand_lists = [personal.adjust(g, c, embedding=e) for g, c, e in zip(groups_strokes, cand_lists, embeddings)]
    labels = gl.symbol_labels(glossary_names)
    if labels:
        cand_lists = [apply_glossary(c, labels) for c in cand_lists]
    # los que se parecen mucho entre sí comparten etiqueta
    groups = shape_groups(embeddings) if syms else []
    cand_lists = share_within_groups(cand_lists, groups)

    # contexto: todos los símbolos de la línea en orden de lectura (los ya conocidos, fijos)
    all_c = cand_lists + [[(k["latex"], 1.0)] for k in known]
    all_b = boxes + [tuple(k["box"]) for k in known]
    order = reading_order(all_c, all_b)
    lm = getattr(classifier, "lm", None)
    if lm is not None and all_c:
        ranked = viterbi([all_c[i] for i in order], lm)
        for pos, i in enumerate(order):
            all_c[i] = ranked[pos]
    cand_lists = all_c[: len(syms)]

    out_symbols = []
    for s, cands, box, g in zip(syms, cand_lists, boxes, groups):
        top, p = cands[0]
        confidence = p * (1.0 - s.get("junk", 0.0))
        item = {
            "strokes": s["strokes"],
            "box": list(box),
            "candidates": [{"latex": c, "p": round(q, 4)} for c, q in cands[:6]],
            "confidence": round(confidence, 4),
            "group": g,
            "clean": None,
        }
        if clean and confidence >= threshold:
            lines = glyphs.place_glyph(top, box)
            if lines is not None:
                item["clean"] = [[[round(x, 2), round(y, 2)] for x, y in line] for line in lines]
        out_symbols.append(item)

    layout_syms = [Sym(c[0][0], b) for c, b in zip(cand_lists, boxes)] + [
        Sym(k["latex"], tuple(k["box"])) for k in known
    ]
    tree = parse(layout_syms)
    result = answer(tree, units=units) if equals else None
    confidence = min([s["confidence"] for s in out_symbols] + [1.0])
    return {"symbols": out_symbols, "latex": to_latex(tree), "result": result, "confidence": round(confidence, 4)}
