# SPDX-License-Identifier: GPL-3.0-or-later
"""Agrupación de trazos en símbolos.

Los trazos llegan en el orden en que se escribieron. Un símbolo está formado por 1 a 4 trazos consecutivos que se
tocan o están muy cerca (p. ej. "=", "+", "x", "i", "π"). Entre todas las agrupaciones posibles se elige la que hace
más probable la lectura según el clasificador (programación dinámica), con una pequeña penalización por símbolo para
no partir de más.
"""

import json
import math
import os
from dataclasses import asdict, dataclass, fields

MAX_STROKES = 4

DEFAULT_PARAMS_PATH = os.path.join(
    os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math", "corte.json"
)


@dataclass
class SegmentParams:
    """Reglas del corte en símbolos. Los valores por defecto son los de la letra de MathWriting; `calibrar_corte.py`
    los ajusta a tu letra y los guarda en ~/.local/share/onepepe/math/corte.json."""

    # dos trazos "se tocan" si los separa menos de esto (× tamaño típico de símbolo) en horizontal y en vertical
    touch_gap: float = 0.25
    # uno encima del otro (=, i, ÷): hueco vertical máximo (× tamaño típico)
    stack_gap: float = 1.0
    # penalización por cada símbolo (más alta: junta más trazos)
    symbol_penalty: float = 1.2
    # cuánto cuenta la probabilidad de que un grupo sea un "trozo" y no un símbolo completo
    junk_weight: float = 1.0
    # pausas entre trazos (si se conocen): por debajo de esto (s) favorece el mismo símbolo; por encima, lo contrario
    pause_same: float = 0.3
    pause_weight: float = 0.0

    @classmethod
    def load(cls, path=DEFAULT_PARAMS_PATH):
        """Los parámetros calibrados para tu letra, o los de por defecto si no hay."""
        try:
            with open(path, encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, ValueError):
            return cls()
        known = {f.name for f in fields(cls)}
        return cls(**{k: float(v) for k, v in data.items() if k in known})

    def save(self, path=DEFAULT_PARAMS_PATH):
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(asdict(self), f, indent=2)


def bbox(strokes):
    xs = [p[0] for s in strokes for p in s]
    ys = [p[1] for s in strokes for p in s]
    return (min(xs), min(ys), max(xs), max(ys))


def _close(a, b, typical, params=None):
    """¿Están las cajas a y b lo bastante cerca para ser el mismo símbolo?"""
    params = params or SegmentParams()
    gap_x = max(0.0, max(a[0], b[0]) - min(a[2], b[2]))
    gap_y = max(0.0, max(a[1], b[1]) - min(a[3], b[3]))
    overlap_x = min(a[2], b[2]) - max(a[0], b[0])
    narrower = max(min(a[2] - a[0], b[2] - b[0]), 0.05 * typical)
    # uno encima del otro (=, i, j, :, ÷, ≤, Σ): solapan en horizontal y hay poco hueco vertical
    if overlap_x > min(0.3 * narrower, 0.2 * typical) - 0.35 * typical and gap_y < params.stack_gap * typical:
        return True
    # se tocan o casi (+, x, k, π, 4)
    return gap_x < params.touch_gap * typical and gap_y < params.touch_gap * typical


def typical_size(strokes):
    """Tamaño típico de un símbolo: la mediana de la altura de los trazos, sin contar los muy planos."""
    sizes = []
    for s in strokes:
        b = bbox([s])
        sizes.append(max(b[3] - b[1], (b[2] - b[0]) * 0.5))
    sizes.sort()
    return max(sizes[len(sizes) // 2], 1e-6) if sizes else 1.0


def _pause_score(times, start, end, params):
    """Premio o castigo por las pausas: dentro del grupo deberían ser cortas; antes del grupo, larga."""
    if not times or params.pause_weight == 0.0:
        return 0.0
    score = 0.0
    for i in range(start + 1, end):
        pause = times[i][0] - times[i - 1][1]
        score += params.pause_weight * (params.pause_same - pause) / params.pause_same
    if start > 0:
        pause = times[start][0] - times[start - 1][1]
        score += params.pause_weight * (pause - params.pause_same) / params.pause_same
    return max(min(score, 3.0), -3.0) if end - start > 1 or start > 0 else 0.0


def segment(strokes, classifier, params=None, k=8, times=None):
    """Agrupa los trazos en símbolos.

    `times`: opcional, [(inicio, fin)] en segundos de cada trazo; las pausas cortas favorecen el mismo símbolo.
    Devuelve una lista de dicts: {"strokes": [índices], "candidates": [(latex, p), ...]}.
    """
    params = params or SegmentParams()
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
                any(_close(boxes[i], boxes[j], typical, params) for j in range(start, i))
                for i in range(start + 1, end)
            ):
                continue
            key = (start, end)
            if key not in cache:
                cache[key] = classifier.candidates_w_junk(strokes[start:end], k)
            cands, junk = cache[key]
            score = (
                best[start][0]
                + math.log(max(cands[0][1], 1e-6))
                + params.junk_weight * math.log(max(1.0 - junk, 1e-6))
                - params.symbol_penalty
                + _pause_score(times, start, end, params)
            )
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
