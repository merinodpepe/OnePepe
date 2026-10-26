# SPDX-License-Identifier: GPL-3.0-or-later
"""Características de un símbolo escrito a mano, para el clasificador.

Un símbolo es una lista de trazos; cada trazo, una lista de puntos (x, y) con el eje y hacia abajo.
Las características no dependen de la posición ni del tamaño del símbolo, pero sí de su proporción.
"""

import math

import numpy as np

N_POINTS = 32
GRID = 10
N_FEATURES = N_POINTS * 5 + GRID * GRID + 6


def _normalize(strokes):
    pts = [p for s in strokes for p in s]
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    w, h = x1 - x0, y1 - y0
    size = max(w, h, 1e-6)
    cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
    norm = [[((x - cx) / size * 2, (y - cy) / size * 2) for x, y in s] for s in strokes]
    return norm, w, h


def _resample(strokes, n):
    """Remuestrea los trazos seguidos (con saltos de lápiz) en n puntos equidistantes.

    Devuelve una lista de (x, y, dx, dy, salto) donde salto = 1 si el punto cae en un salto entre trazos.
    """
    segs = []
    for si, s in enumerate(strokes):
        if si > 0:
            segs.append((strokes[si - 1][-1], s[0], 1.0))
        if len(s) == 1:
            segs.append((s[0], s[0], 0.0))
        for a, b in zip(s, s[1:]):
            segs.append((a, b, 0.0))
    # los saltos cuentan con poco peso, para que no dominen
    lens = [math.dist(a, b) * (0.3 if jump else 1.0) for a, b, jump in segs]
    total = sum(lens)
    out = []
    if total <= 1e-9:
        p = strokes[0][0]
        return [(p[0], p[1], 0.0, 0.0, 0.0)] * n
    targets = [total * i / (n - 1) for i in range(n)]
    acc, k = 0.0, 0
    for t in targets:
        while k < len(segs) - 1 and acc + lens[k] < t:
            acc += lens[k]
            k += 1
        a, b, jump = segs[k]
        f = 0.0 if lens[k] <= 1e-12 else min(max((t - acc) / lens[k], 0.0), 1.0)
        x, y = a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f
        d = math.dist(a, b)
        dx, dy = ((b[0] - a[0]) / d, (b[1] - a[1]) / d) if d > 1e-9 else (0.0, 0.0)
        out.append((x, y, dx, dy, jump))
    return out


def _raster(strokes, grid):
    img = np.zeros((grid, grid), dtype=np.float32)
    for s in strokes:
        seg = s if len(s) > 1 else s * 2
        for a, b in zip(seg, seg[1:]):
            steps = max(2, int(math.dist(a, b) * grid))
            for i in range(steps + 1):
                f = i / steps
                x = (a[0] + (b[0] - a[0]) * f + 1) / 2 * (grid - 1)
                y = (a[1] + (b[1] - a[1]) * f + 1) / 2 * (grid - 1)
                ix, iy = int(round(x)), int(round(y))
                if 0 <= ix < grid and 0 <= iy < grid:
                    img[iy, ix] = 1.0
    return img.ravel()


def symbol_features(strokes):
    """Vector de características (numpy, float32) de un símbolo."""
    strokes = [s for s in strokes if s]
    norm, w, h = _normalize(strokes)
    seq = np.array(_resample(norm, N_POINTS), dtype=np.float32).ravel()
    raster = _raster(norm, GRID)
    n = len(strokes)
    extra = np.array(
        [
            math.log((w + 1e-3) / (h + 1e-3)) / 3,
            1.0 if n == 1 else 0.0,
            1.0 if n == 2 else 0.0,
            1.0 if n == 3 else 0.0,
            1.0 if n >= 4 else 0.0,
            min(n, 6) / 6,
        ],
        dtype=np.float32,
    )
    return np.concatenate([seq, raster, extra])


def augment(strokes, rng):
    """Una variante deformada del símbolo: giro, inclinación, escala, temblor y a veces otro orden de trazos."""
    pts = [p for s in strokes for p in s]
    cx = sum(p[0] for p in pts) / len(pts)
    cy = sum(p[1] for p in pts) / len(pts)
    ang = rng.normal(0, 0.12)
    shear = rng.normal(0, 0.15)
    sx = math.exp(rng.normal(0, 0.12))
    sy = math.exp(rng.normal(0, 0.12))
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    size = max(max(xs) - min(xs), max(ys) - min(ys), 1e-6)
    jitter = abs(rng.normal(0, 0.012)) * size
    ca, sa = math.cos(ang), math.sin(ang)
    out = []
    for s in strokes:
        ns = []
        for x, y in s:
            x, y = x - cx, y - cy
            x, y = x + shear * y, y
            x, y = x * sx, y * sy
            x, y = ca * x - sa * y, sa * x + ca * y
            ns.append((x + rng.normal(0, jitter), y + rng.normal(0, jitter)))
        out.append(ns)
    if len(out) > 1 and rng.random() < 0.3:
        rng.shuffle(out)
    if rng.random() < 0.15:
        out = [s[::-1] for s in out]
    return out
