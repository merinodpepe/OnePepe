# SPDX-License-Identifier: GPL-3.0-or-later
"""Corte "forzado": dada una expresión escrita y su LaTeX, el corte en símbolos que mejor encaja con los símbolos que
debe haber. Sirve para convertir tus páginas etiquetadas (mis_expresiones, mis_simbolos) en ejemplos de tu letra:
símbolos con su etiqueta y "trozos" (medio símbolo, o dos símbolos juntos) para enseñar a cortar.
"""

import math
from collections import Counter

from .segment import MAX_STROKES


def forced_groups(strokes, tokens, classifier):
    """Agrupa los trazos (en orden de escritura) en exactamente len(tokens) símbolos de 1 a MAX_STROKES trazos
    seguidos, maximizando la probabilidad de que cada grupo sea alguno de los símbolos esperados.

    Devuelve [(índices de trazos, etiqueta)] o None si no se puede (más símbolos que trazos, etc.).
    """
    n, m = len(strokes), len(tokens)
    if m == 0 or n < m or n > m * MAX_STROKES:
        return None
    index = {c: i for i, c in enumerate(classifier.classes)}
    wanted = [t for t in set(tokens) if t in index]
    if not wanted:
        return None
    cache = {}

    def group_score(a, b):
        if (a, b) not in cache:
            p = classifier.probabilities(strokes[a:b])
            best = max(wanted, key=lambda t: p[index[t]])
            cache[(a, b)] = (math.log(max(p[index[best]], 1e-9)), best)
        return cache[(a, b)]

    # best[end][count] = (puntuación, inicio)
    best = [[(-math.inf, None)] * (m + 1) for _ in range(n + 1)]
    best[0][0] = (0.0, None)
    for end in range(1, n + 1):
        for count in range(1, min(m, end) + 1):
            for size in range(1, min(MAX_STROKES, end) + 1):
                start = end - size
                prev = best[start][count - 1][0]
                if prev == -math.inf:
                    continue
                score = prev + group_score(start, end)[0]
                if score > best[end][count][0]:
                    best[end][count] = (score, start)
    if best[n][m][0] == -math.inf:
        return None
    groups = []
    end, count = n, m
    while count > 0:
        start = best[end][count][1]
        groups.append(list(range(start, end)))
        end, count = start, count - 1
    groups.reverse()
    # etiquetas: cada grupo con el símbolo esperado más probable, sin gastar más símbolos de los que hay
    left = Counter(tokens)
    labeled = []
    order = sorted(range(len(groups)), key=lambda i: -group_score(groups[i][0], groups[i][-1] + 1)[0])
    labels = [None] * len(groups)
    for i in order:
        g = groups[i]
        p = classifier.probabilities(strokes[g[0] : g[-1] + 1])
        options = [t for t in left if left[t] > 0 and t in index]
        if not options:
            return None
        label = max(options, key=lambda t: p[index[t]])
        left[label] -= 1
        labels[i] = label
    for g, label in zip(groups, labels):
        labeled.append((g, label))
    return labeled


def junk_examples(strokes, labeled, max_strokes=MAX_STROKES):
    """Ejemplos de "trozo" sacados de un corte correcto: cada trazo suelto de un símbolo de varios trazos, y cada par
    de símbolos vecinos juntados (si no pasan de `max_strokes` trazos)."""
    out = []
    for g, _ in labeled:
        if len(g) > 1:
            for i in g:
                out.append([strokes[i]])
    for (a, _), (b, _) in zip(labeled, labeled[1:]):
        if len(a) + len(b) <= max_strokes:
            out.append([strokes[i] for i in a + b])
    return out
