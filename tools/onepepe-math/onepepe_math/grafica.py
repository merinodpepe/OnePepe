# SPDX-License-Identifier: GPL-3.0-or-later
"""Gráficas de y = f(x) (fase 6 de MyScript y PLAN-PENDIENTE.md §2, «Gráficas de funciones»).

La app manda los renglones seleccionados ya reconocidos; aquí se calculan los puntos con numpy y se devuelven como
líneas (ejes, marcas, números y curvas) dentro de una caja de `size` unidades, con el eje y hacia abajo. La app las
dibuja como trazos normales, así que se mueven, se borran, se deshacen y se ven también en Rnote.

Se entiende:
- `y = expresión`, `f(x) = expresión` o una expresión con una sola variable;
- varias curvas a la vez (un renglón cada una);
- el intervalo en un renglón aparte: `x ∈ [0; 2π]` (con «;» o con coma); si no, de −10 a 10.
"""

import math

import numpy as np
import sympy as sp

from .evaluate import RELATIONS, ParseError, _Parser, _split, _tokens
from .glyphs import place_glyph
from .layout import parse, to_latex
from .pagina import _row_syms


def _range(tokens):
    """[a, b] de un renglón «x ∈ [a; b]», o None."""
    if len(tokens) < 6 or tokens[1][0] != "sym" or tokens[1][1] != r"\in":
        return None
    inner = tokens[2:]
    if inner[0][1] not in ("[", "(") or inner[-1][1] not in ("]", ")"):
        return None
    inner = inner[1:-1]
    for sep in (";", ","):
        parts = [p for p, _ in _split(inner, {sep})]
        if len(parts) == 2 and all(parts):
            a, b = (float(_Parser(p, {}).full()) for p in parts)
            if b > a:
                return a, b
    return None


def _function(tokens):
    """(expresión, variable) de un renglón, o None."""
    parts = _split(tokens, RELATIONS)
    if len(parts) == 2 and parts[0][1] == "=":
        lhs, rhs = parts[0][0], parts[1][0]
        expr = _Parser(rhs, {}).full()
        # «f(x) = …»: la variable va entre paréntesis
        if len(lhs) == 4 and lhs[1][1] == "(" and lhs[3][1] == ")":
            var = _Parser([], {})._symbol(lhs[2])
            return expr, var
        if len(lhs) != 1:
            return None
    elif len(parts) == 1:
        expr = _Parser(tokens, {}).full()
    else:
        return None
    free = sorted(expr.free_symbols, key=lambda s: s.name)
    if len(free) != 1:
        return None
    return expr, free[0]


def _nice_step(span, ticks=6):
    raw = span / ticks
    mag = 10 ** math.floor(math.log10(raw))
    for m in (1, 2, 2.5, 5, 10):
        if m * mag >= raw:
            return m * mag
    return 10 * mag


def _number_text(v, step):
    if abs(v) < step * 1e-6:
        return "0"
    decimals = max(0, -int(math.floor(math.log10(step)))) if step < 1 else 0
    return f"{v:.{decimals}f}"


def _text_lines(text, x, y, height):
    """Líneas de un número escrito con los glifos limpios, empezando en (x, y) arriba a la izquierda."""
    out = []
    w = 0.6 * height
    for ch in text:
        lines = place_glyph(ch, (x, y, x + w, y + height)) if ch not in " " else None
        if lines:
            out += [[[round(px, 2), round(py, 2)] for px, py in line] for line in lines]
        x += w * 1.15
    return out


def plot_rows(rows, size=(300.0, 200.0), samples=600):
    """Gráfica de las funciones de los renglones. Devuelve {"axes", "labels", "curves", "x_range", "y_range",
    "functions"} o {"error"} si no hay nada que dibujar."""
    w, h = float(size[0]), float(size[1])
    functions, x_range = [], None
    for row in rows:
        syms = _row_syms(row)
        if not syms:
            continue
        tree = parse(syms)
        try:
            r = _range(_tokens(tree, decimal_comma=False))
            if r:
                x_range = r
                continue
            f = _function(_tokens(tree))
        except (ParseError, ValueError, TypeError):
            continue
        if f:
            functions.append((f, to_latex(tree)))
    if not functions:
        return {"error": "No hay ninguna función de una variable para dibujar"}
    x0, x1 = x_range or (-10.0, 10.0)
    xs = np.linspace(x0, x1, samples)
    curves_y = []
    for (expr, var), _ in functions:
        f = sp.lambdify(var, expr.doit(), modules="numpy")
        with np.errstate(all="ignore"):
            ys = np.asarray(f(xs), dtype=complex) * np.ones_like(xs)
        ys = np.where(np.abs(ys.imag) < 1e-9, ys.real, np.nan).astype(float)
        ys[~np.isfinite(ys)] = np.nan
        curves_y.append(ys)
    finite = np.concatenate([y[np.isfinite(y)] for y in curves_y]) if curves_y else np.array([])
    if finite.size == 0:
        return {"error": "La función no tiene valores reales en ese intervalo"}
    # sin dejar que las asíntotas (1/x, tan x) aplasten el resto
    lo, hi = np.percentile(finite, [3, 97])
    if hi - lo < 1e-9:
        lo, hi = lo - 1.0, hi + 1.0
    pad = 0.08 * (hi - lo)
    y0, y1 = float(lo - pad), float(hi + pad)

    def px(x):
        return (x - x0) / (x1 - x0) * w

    def py(y):
        return h - (y - y0) / (y1 - y0) * h

    # curvas: se cortan donde no existen o saltan (asíntotas)
    curves = []
    for ys in curves_y:
        lines, cur = [], []
        for x, y in zip(xs, ys):
            inside = np.isfinite(y) and y0 - (y1 - y0) <= y <= y1 + (y1 - y0)
            if inside and cur and abs(y - cur[-1][2]) > 0.5 * (y1 - y0):
                inside = False
            if inside:
                cur.append((px(x), py(min(max(y, y0), y1)), y))
            elif cur:
                lines.append(cur)
                cur = []
        if cur:
            lines.append(cur)
        curves.append([[[round(a, 2), round(b, 2)] for a, b, _ in line] for line in lines if len(line) > 1])
    # ejes: en el 0 si se ve; si no, en el borde
    ax_y = py(0.0) if y0 <= 0 <= y1 else h
    ax_x = px(0.0) if x0 <= 0 <= x1 else 0.0
    axes = [[[0.0, round(ax_y, 2)], [w, round(ax_y, 2)]], [[round(ax_x, 2), 0.0], [round(ax_x, 2), h]]]
    # puntas de flecha
    a = 0.025 * min(w, h)
    axes.append([[w - a, ax_y - a], [w, ax_y], [w - a, ax_y + a]])
    axes.append([[ax_x - a, a], [ax_x, 0.0], [ax_x + a, a]])
    labels = []
    t = 0.012 * min(w, h)
    th = 0.045 * min(w, h)
    sx = _nice_step(x1 - x0)
    v = math.ceil(x0 / sx) * sx
    while v <= x1 - 0.5 * sx:
        if abs(v) > 1e-9:
            p = px(v)
            axes.append([[round(p, 2), round(ax_y - t, 2)], [round(p, 2), round(ax_y + t, 2)]])
            text = _number_text(v, sx)
            labels += _text_lines(text, p - 0.35 * th * len(text), ax_y + 2 * t, th)
        v += sx
    sy = _nice_step(y1 - y0)
    v = math.ceil(y0 / sy) * sy
    while v <= y1 - 0.3 * sy:
        if abs(v) > 1e-9:
            p = py(v)
            axes.append([[round(ax_x - t, 2), round(p, 2)], [round(ax_x + t, 2), round(p, 2)]])
            text = _number_text(v, sy)
            labels += _text_lines(text, ax_x + 2 * t, p - 0.5 * th, th)
        v += sy
    return {
        "axes": axes,
        "labels": labels,
        "curves": curves,
        "x_range": [x0, x1],
        "y_range": [y0, y1],
        "functions": [latex for _, latex in functions],
    }
