# SPDX-License-Identifier: GPL-3.0-or-later
"""Agrupación de trazos en símbolos.

Los trazos llegan en el orden en que se escribieron. Un símbolo está formado por 1 a 4 trazos consecutivos que se
tocan o están muy cerca (p. ej. "=", "+", "x", "i", "π"). Entre todas las agrupaciones posibles se elige la que hace
más probable la lectura según el clasificador (programación dinámica), con una pequeña penalización por símbolo para
no partir de más.
"""

import math

MAX_STROKES = 4


def bbox(strokes):
    xs = [p[0] for s in strokes for p in s]
    ys = [p[1] for s in strokes for p in s]
    return (min(xs), min(ys), max(xs), max(ys))


def _close(a, b, typical):
    """¿Están las cajas a y b lo bastante cerca para ser el mismo símbolo?"""
    gap_x = max(0.0, max(a[0], b[0]) - min(a[2], b[2]))
    gap_y = max(0.0, max(a[1], b[1]) - min(a[3], b[3]))
    overlap_x = min(a[2], b[2]) - max(a[0], b[0])
    narrower = max(min(a[2] - a[0], b[2] - b[0]), 0.05 * typical)
    # uno encima del otro (=, i, j, :, ÷, ≤, Σ): solapan en horizontal y hay poco hueco vertical
    if overlap_x > min(0.3 * narrower, 0.2 * typical) - 0.35 * typical and gap_y < 1.0 * typical:
        return True
    # se tocan o casi (+, x, k, π, 4)
    return gap_x < 0.25 * typical and gap_y < 0.25 * typical


def typical_size(strokes):
    """Tamaño típico de un símbolo: la mediana de la altura de los trazos, sin contar los muy planos."""
    sizes = []
    for s in strokes:
        b = bbox([s])
        sizes.append(max(b[3] - b[1], (b[2] - b[0]) * 0.5))
    sizes.sort()
    return max(sizes[len(sizes) // 2], 1e-6) if sizes else 1.0


def segment(strokes, classifier, symbol_penalty=1.2, k=8):
    """Agrupa los trazos en símbolos.

    Devuelve una lista de dicts: {"strokes": [índices], "candidates": [(latex, p), ...]}.
    """
    n = len(strokes)
    if n == 0:
        return []
    typical = typical_size(strokes)
    boxes = [bbox([s]) for s in strokes]
    best = [(-math.inf, None)] * (n + 1)
    best[0] = (0.0, None)
    cache = {}
    for end in range(1, n + 1):
        for size in range(1, min(MAX_STROKES, end) + 1):
            start = end - size
            if best[start][0] == -math.inf:
                continue
            # los trazos de un grupo deben estar encadenados por cercanía
            if size > 1 and not all(
                any(_close(boxes[i], boxes[j], typical) for j in range(start, i)) for i in range(start + 1, end)
            ):
                continue
            key = (start, end)
            if key not in cache:
                cache[key] = classifier.candidates_w_junk(strokes[start:end], k)
            cands, _junk = cache[key]
            score = best[start][0] + math.log(max(cands[0][1], 1e-6)) - symbol_penalty
            if score > best[end][0]:
                best[end] = (score, start)
    symbols = []
    end = n
    while end > 0:
        start = best[end][1]
        cands, junk = cache[(start, end)]
        total = sum(p for _, p in cands) or 1.0
        symbols.append(
            {
                "strokes": list(range(start, end)),
                # probabilidades entre símbolos completos (sin la clase "trozo")
                "candidates": [(c, p / max(1.0 - junk, total)) for c, p in cands],
                "junk": junk,
            }
        )
        end = start
    symbols.reverse()
    return symbols
