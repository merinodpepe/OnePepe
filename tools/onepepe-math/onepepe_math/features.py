# SPDX-License-Identifier: GPL-3.0-or-later
"""Características de un símbolo escrito a mano, para el clasificador.

Un símbolo es una lista de trazos; cada trazo, una lista de puntos (x, y) con el eje y hacia abajo.
Las características no dependen de la posición ni del tamaño del símbolo, pero sí de su proporción.
"""

import itertools
import math

import numpy as np

from .segment import bbox, typical_size

N_POINTS = 32
GRID = 10
N_FEATURES = N_POINTS * 5 + GRID * GRID + 6  # v1
VERSION = 2
GRID2 = 16
N_FEATURES_V2 = N_POINTS * 5 + 128 + GRID2 * GRID2 + 6 + 6  # 556
PASO = 0.05


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


def symbol_features_v1(strokes):
    """Vector de características v1 (numpy, float32) de un símbolo."""
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


# ---------------------------------------------------------------- v2 (M7.2-M7.5)


def _remuestreo_paso(norm):
    """Cada trazo remuestreado a paso PASO: lista de arrays (n, 2)."""
    out = []
    for s in norm:
        a = np.asarray(s, dtype=np.float64)
        if len(a) < 2:
            out.append(a.reshape(-1, 2))
            continue
        cum = np.concatenate([[0.0], np.cumsum(np.hypot(*np.diff(a, axis=0).T))])
        t = np.linspace(0.0, cum[-1], max(1, math.ceil(cum[-1] / PASO)) + 1)
        out.append(np.stack([np.interp(t, cum, a[:, 0]), np.interp(t, cum, a[:, 1])], axis=1))
    return out


def _bilineal(c, n):
    """Índices y pesos de las dos celdas vecinas de la coordenada c, recortados a [0, n-1]."""
    i0 = np.floor(c).astype(int)
    f = c - i0
    return (np.clip(i0, 0, n - 1), 1 - f), (np.clip(i0 + 1, 0, n - 1), f)


def _direcciones(tramos):
    """M7.3: histograma 4x4x8 de orientaciones (módulo π) ponderado por longitud, raíz de H/Σℓ."""
    H = np.zeros((4, 4, 8))
    seg = [t for t in tramos if len(t) > 1]
    if not seg:
        return H.ravel().astype(np.float32)
    a = np.concatenate([t[:-1] for t in seg])
    b = np.concatenate([t[1:] for t in seg])
    d = b - a
    ln = np.hypot(d[:, 0], d[:, 1])
    m = (a + b) / 2
    bk = (np.arctan2(d[:, 1], d[:, 0]) % math.pi) / (math.pi / 8)
    k0 = np.floor(bk).astype(int)
    orient = (((k0 % 8), 1 - (bk - k0)), (((k0 + 1) % 8), bk - k0))
    for ix, wx in _bilineal((m[:, 0] + 1) * 2 - 0.5, 4):
        for iy, wy in _bilineal((m[:, 1] + 1) * 2 - 0.5, 4):
            for ik, wk in orient:
                np.add.at(H, (iy, ix, ik), ln * wk * wx * wy)
    return np.sqrt(H / max(ln.sum(), 1e-9)).ravel().astype(np.float32)


def _rejilla16(tramos):
    """M7.4: puntos del remuestreo repartidos bilinealmente en 16x16, suavizado [1 2 1]/4 y tope 1."""
    g = np.zeros((GRID2, GRID2))
    p = np.concatenate(tramos)
    for ix, wx in _bilineal((p[:, 0] + 1) / 2 * (GRID2 - 1), GRID2):
        for iy, wy in _bilineal((p[:, 1] + 1) / 2 * (GRID2 - 1), GRID2):
            np.add.at(g, (iy, ix), wx * wy)
    for ax in (0, 1):
        q = np.pad(g, [(1, 1) if i == ax else (0, 0) for i in range(2)])
        g = (np.take(q, range(0, GRID2), ax) + 2 * np.take(q, range(1, GRID2 + 1), ax)
             + np.take(q, range(2, GRID2 + 2), ax)) / 4
    return np.minimum(g, 1.0).ravel().astype(np.float32)


def _contexto(box, linea):
    """M7.5: seis valores del símbolo respecto al renglón; ceros sin contexto."""
    if linea is None or linea[0] <= 1e-6:  # típico 0: línea degenerada, sin contexto
        return np.zeros(6, dtype=np.float32)
    tip, pend, corte = linea
    x0, y0, x1, y1 = box
    w, h = max(x1 - x0, 0.02 * tip), max(y1 - y0, 0.02 * tip)
    base = corte + pend * (x0 + x1) / 2
    lg = lambda v: float(np.clip(math.log(v / tip), -3, 2)) / 2  # noqa: E731
    cl = lambda v: float(np.clip(v / tip, -3, 3)) / 3  # noqa: E731
    return np.array([1.0, lg(w), lg(h), cl(y0 - base), cl(y1 - base), lg(max(w, h))], dtype=np.float32)


def symbol_features(strokes, linea=None, version=VERSION):
    """Vector de características de un símbolo: v1 (266 valores) o v2 (556). `linea`: (típico, pendiente, corte)."""
    if version < 2:
        return symbol_features_v1(strokes)
    strokes = [s for s in strokes if s]
    v1 = symbol_features_v1(strokes)
    norm, _, _ = _normalize(strokes)
    tramos = _remuestreo_paso(norm)
    return np.concatenate([v1[: N_POINTS * 5], _direcciones(tramos), _rejilla16(tramos),
                           _contexto(bbox(strokes), linea), v1[-6:]])


def linea(strokes, known_boxes=()):
    """M7.5: (típico, pendiente, corte) del renglón, con los trazos y las cajas de los símbolos conocidos; o None."""
    tip = typical_size(strokes)
    if known_boxes:
        sizes = sorted([max(b[3] - b[1], (b[2] - b[0]) * 0.5) for b in map(lambda s: bbox([s]), strokes)]
                       + [max(b[3] - b[1], (b[2] - b[0]) * 0.5) for b in known_boxes])
        tip = max(sizes[len(sizes) // 2], 1e-6)
    boxes = [bbox([s]) for s in strokes] + [tuple(b) for b in known_boxes]
    pts = [((b[0] + b[2]) / 2, b[3]) for b in boxes if b[3] - b[1] >= 0.5 * tip]
    if len(pts) < 2:
        return None
    pend = 0.0
    if len(pts) >= 4 and max(p[0] for p in pts) - min(p[0] for p in pts) > 4 * tip:
        sl = sorted((q[1] - p[1]) / (q[0] - p[0]) for p, q in itertools.combinations(pts, 2) if q[0] != p[0])
        if sl:
            pend = max(-0.3, min(0.3, (sl[(len(sl) - 1) // 2] + sl[len(sl) // 2]) / 2))
    fondos = sorted(y - pend * x for x, y in pts)
    corte = (fondos[(len(fondos) - 1) // 2] + fondos[len(fondos) // 2]) / 2
    return (tip, pend, corte)


def perturbar_linea(lin, rng):
    """Entrenamiento: típico·exp(N(0; 0,1)) y base + N(0; 0,08·típico)."""
    if lin is None:
        return None
    tip = lin[0] * math.exp(rng.normal(0, 0.1))
    return (tip, lin[1], lin[2] + rng.normal(0, 0.08 * tip))


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
