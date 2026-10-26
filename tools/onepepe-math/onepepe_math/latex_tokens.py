# SPDX-License-Identifier: GPL-3.0-or-later
"""Partir una cadena LaTeX en símbolos visibles, en orden de lectura."""

import re

from .symbols import normalize_label

_TOKEN = re.compile(r"\\[A-Za-z]+|\\.|[^\s]")
# órdenes que no se dibujan como un símbolo propio
_INVISIBLE = {"^", "_", "{", "}", "&", "~", r"\\", r"\,", r"\;", r"\!", r"\left", r"\right", r"\big", r"\Big",
              r"\mathrm", r"\mathbf", r"\mathit", r"\text", r"\operatorname", r"\displaystyle", r"\limits",
              r"\begin", r"\end", r"\quad", r"\qquad", r"\mathcal", r"\mathbb"}


def visible_tokens(latex):
    """Símbolos visibles de una expresión LaTeX (p. ej. `\\frac{a}{b}` -> [\\frac, a, b])."""
    out = []
    for tok in _TOKEN.findall(latex):
        if tok in _INVISIBLE:
            continue
        out.append(normalize_label(tok))
    return out
