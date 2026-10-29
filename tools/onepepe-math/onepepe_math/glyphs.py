# SPDX-License-Identifier: GPL-3.0-or-later
"""Símbolos limpios dibujados con trazos de una sola línea (fuentes Hershey).

Los datos de las fuentes vienen del paquete `Hershey-Fonts` (PyPI, envoltorio con licencia MIT). Créditos exigidos por la
nota de uso de las fuentes Hershey: diseñadas por el Dr. A. V. Hershey en el U.S. National Bureau of Standards; formato
de los datos de James Hurt (Cognition, Inc.). No se distribuyen en el formato original del NTIS.

Coordenadas: como en Rnote, con el eje y hacia abajo.
"""

from functools import lru_cache

# LaTeX -> (fuente Hershey, carácter de la fuente). Comprobado dibujando las tablas de las fuentes.
_MATHLOW = {
    r"\pm": "!", r"\mp": '"', r"\times": "#", r"\cdot": "$", r"\leq": "&", r"\geq": "'", "(": "(", ")": ")",
    "*": "*", "+": "+", "-": "-", ".": ".", "/": "/", r"\prod": ":", r"\sum": ";", "<": "<", "=": "=", ">": ">",
    r"\neq": "?", r"\equiv": "@", "[": "[", "]": "]", r"\propto": "^", r"\infty": "_", r"\circ": "`",
    r"\sqrt": "c", r"\in": "h", r"\rightarrow": "i", r"\uparrow": "j", r"\leftarrow": "k", r"\downarrow": "l",
    r"\partial": "m", r"\nabla": "n", r"\int": "p", r"\exists": "v", r"\div": "x", r"\|": "y", r"\perp": "z",
    r"\{": "{", r"\angle": "|", r"\}": "}", r"\therefore": "~", r"\backslash": "\\", r"\subset": "d",
    r"\cup": "e", r"\supset": "f", r"\cap": "g",
}
# griego (fuente "greek", alfabeto latino -> griego)
_GREEK = {
    r"\alpha": "a", r"\beta": "b", r"\gamma": "g", r"\delta": "d", r"\epsilon": "e", r"\varepsilon": "e",
    r"\zeta": "z", r"\eta": "h", r"\theta": "q", r"\vartheta": "q", r"\iota": "i", r"\kappa": "k", r"\lambda": "l", r"\mu": "m",
    r"\nu": "n", r"\xi": "x", r"\pi": "p", r"\rho": "r", r"\sigma": "s", r"\tau": "t", r"\upsilon": "u",
    r"\phi": "f", r"\varphi": "f", r"\chi": "c", r"\psi": "y", r"\omega": "w",
    r"\Gamma": "G", r"\Delta": "D", r"\Theta": "Q", r"\Lambda": "L", r"\Xi": "X", r"\Pi": "P", r"\Sigma": "S",
    r"\Upsilon": "U", r"\Phi": "F", r"\Psi": "Y", r"\Omega": "W",
}
_ROMAN_CHARS = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ,'!%:;?|"

# Símbolos que no están en las fuentes: dibujados a mano, en una caja de 0..1 (y hacia abajo).
_MANUAL = {
    # dos virgulillas
    r"\approx": [
        [(0.0, 0.40), (0.15, 0.30), (0.35, 0.30), (0.5, 0.40), (0.65, 0.50), (0.85, 0.50), (1.0, 0.40)],
        [(0.0, 0.70), (0.15, 0.60), (0.35, 0.60), (0.5, 0.70), (0.65, 0.80), (0.85, 0.80), (1.0, 0.70)],
    ],
    r"\sim": [[(0.0, 0.55), (0.15, 0.45), (0.35, 0.45), (0.5, 0.55), (0.65, 0.65), (0.85, 0.65), (1.0, 0.55)]],
    # flecha doble ⇒ (dos rayas y una punta)
    r"\Rightarrow": [
        [(0.0, 0.40), (0.85, 0.40)],
        [(0.0, 0.60), (0.85, 0.60)],
        [(0.70, 0.20), (1.0, 0.50), (0.70, 0.80)],
    ],
    # flecha doble con dos puntas ⇔
    r"\Leftrightarrow": [
        [(0.15, 0.40), (0.85, 0.40)],
        [(0.15, 0.60), (0.85, 0.60)],
        [(0.30, 0.20), (0.0, 0.50), (0.30, 0.80)],
        [(0.70, 0.20), (1.0, 0.50), (0.70, 0.80)],
    ],
}


def font_and_char(latex):
    """(fuente, carácter) del glifo, o None si no hay glifo Hershey para ese símbolo."""
    if latex in _MATHLOW:
        return ("mathlow", _MATHLOW[latex])
    if latex in _GREEK:
        return ("greek", _GREEK[latex])
    if len(latex) == 1 and latex in _ROMAN_CHARS:
        return ("futural", latex)
    return None


def supported_symbols():
    """Todos los símbolos LaTeX que se pueden dibujar limpios."""
    return sorted(set(_MATHLOW) | set(_GREEK) | set(_ROMAN_CHARS) | set(_MANUAL))


@lru_cache(maxsize=None)
def _font(name):
    from HersheyFonts import HersheyFonts

    f = HersheyFonts()
    f.load_default_font(name)
    return f


def _merge_segments(segments):
    """Une segmentos consecutivos (a, b), (b, c)... en polilíneas."""
    lines = []
    for a, b in segments:
        a, b = (float(a[0]), float(a[1])), (float(b[0]), float(b[1]))
        if lines and lines[-1][-1] == a:
            lines[-1].append(b)
        else:
            lines.append([a, b])
    return lines


@lru_cache(maxsize=None)
def glyph_polylines(latex):
    """Polilíneas del glifo en coordenadas propias de la fuente (y hacia abajo). None si no hay glifo."""
    if latex in _MANUAL:
        return tuple(tuple(line) for line in _MANUAL[latex])
    fc = font_and_char(latex)
    if fc is None:
        return None
    font, char = fc
    lines = _merge_segments(_font(font).lines_for_text(char))
    if not lines:
        return None
    return tuple(tuple(line) for line in lines)


def _bounds(lines):
    xs = [p[0] for line in lines for p in line]
    ys = [p[1] for line in lines for p in line]
    return min(xs), min(ys), max(xs), max(ys)


# símbolos que son casi líneas horizontales: se ajustan por su anchura y no por su altura
_FLAT = {"-", "=", r"\sim", r"\approx", r"\equiv", r"\rightarrow", r"\leftarrow", r"\Rightarrow", r"\Leftrightarrow"}


def place_glyph(latex, target_box):
    """Polilíneas del símbolo limpio colocadas en `target_box` = (x0, y0, x1, y1) del símbolo escrito a mano.

    Se centra en la caja y se escala para ocupar su altura (o su anchura, en los símbolos planos), sin deformarlo.
    Devuelve None si no hay glifo.
    """
    lines = glyph_polylines(latex)
    if lines is None:
        return None
    gx0, gy0, gx1, gy1 = _bounds(lines)
    gw, gh = max(gx1 - gx0, 1e-9), max(gy1 - gy0, 1e-9)
    tx0, ty0, tx1, ty1 = target_box
    tw, th = max(tx1 - tx0, 1e-9), max(ty1 - ty0, 1e-9)
    if latex in _FLAT or gh < 0.2 * gw:
        scale = tw / gw
        # que no crezca en altura más de lo razonable
        scale = min(scale, 1.5 * max(th, 0.3 * tw) / gh)
    elif gw < 0.2 * gh:
        # símbolos muy estrechos (1, l, |, paréntesis): por la altura
        scale = th / gh
    else:
        # el resto: el que mejor encaje sin salirse mucho, priorizando la altura
        scale = min(th / gh, 1.25 * tw / gw)
    gcx, gcy = (gx0 + gx1) / 2, (gy0 + gy1) / 2
    tcx, tcy = (tx0 + tx1) / 2, (ty0 + ty1) / 2
    return [[(tcx + (x - gcx) * scale, tcy + (y - gcy) * scale) for x, y in line] for line in lines]
