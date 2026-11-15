# SPDX-License-Identifier: GPL-3.0-or-later
"""Las N mejores lecturas de un renglón y su gramática (M10).

Cada símbolo tiene candidatas con probabilidad p. Una lectura y puntúa S(y) = Σ log p + G(y), con G la gramática:
+premio si se puede analizar, −2 por paréntesis sin pareja, −2 por operador mal puesto.
"""

import heapq
import math

from . import evaluate as ev
from .layout import Sym, parse
from .symbols import category


def _binario(t):
    return t[0] == "sym" and category(t[1]) == "binop"


def gramatica(tree, premio=3.0):
    """La G de M10.1 para el árbol de un renglón."""
    try:
        tokens = ev._tokens(tree)
    except Exception:  # noqa: BLE001 -- shortcut: un árbol raro no se puede analizar; G = 0
        return 0.0
    if tokens and tokens[-1][:2] == ["sym", "="]:
        tokens = tokens[:-1]
    ok = bool(tokens)
    try:
        for lado, _ in ev._split(tokens, set(ev.RELATIONS) | {","} | set(ev.STEP_SEPARATORS)):
            ev._Parser(lado, {}).full()
    except Exception:  # noqa: BLE001 -- ParseError, pero también errores de SymPy con entradas raras
        ok = False
    prof = sin_pareja = mal = 0
    for k, t in enumerate(tokens):
        if t[0] == "sym":
            c = category(t[1])
            if c == "open":
                prof += 1
            elif c == "close":
                if prof:
                    prof -= 1
                else:
                    sin_pareja += 1
        if _binario(t):
            previo = tokens[k - 1] if k else None
            if k == len(tokens) - 1 or ((previo is None or _binario(previo)) and t[1] not in ("+", "-")):
                mal += 1
    return premio * ok - 2.0 * (sin_pareja + prof) - 2.0 * mal


def _ordenadas(costes):
    """Combinaciones de rangos de menor a mayor coste total (Dijkstra; cada una sale una vez).
    `costes[k]`: costes crecientes de las opciones de la coordenada k. Da (coste, rangos)."""
    n = len(costes)
    cola = [(0.0, 0, (0,) * n, 0)]
    cuenta = 1
    while cola:
        c, _, rs, a = heapq.heappop(cola)
        yield c, rs
        for b in range(a, n):
            if rs[b] + 1 < len(costes[b]):
                r2 = rs[:b] + (rs[b] + 1,) + rs[b + 1 :]
                heapq.heappush(cola, (sum(cs[r] for cs, r in zip(costes, r2)), cuenta, r2, b))
                cuenta += 1


def mejor_lectura(cands, boxes, fijos=(), premio=3.0, abrir=5, nmax=32, margen=3.0, histeresis=0.5):
    """Índice de la candidata a usar en cada símbolo (0 = la primera). `fijos`: índices que no se abren."""
    n = len(cands)
    cero = [0] * n

    def leer(sel):
        return gramatica(parse([Sym(c[s][0], tuple(b), tuple(x for x, _ in c[:5])) for c, b, s in zip(cands, boxes, sel)]), premio)

    g0 = leer(cero)
    if g0 >= premio:
        return cero

    def lp(j, r):
        return math.log(max(cands[j][r][1], 1e-9))

    dudosos = sorted((lp(j, 0) - lp(j, 1), j) for j in range(n) if j not in fijos and len(cands[j]) > 1)
    abiertos = [j for m, j in dudosos[:abrir] if m < margen]
    opts = [[r for r in range(min(4, len(cands[j]))) if lp(j, 0) - lp(j, r) <= margen] for j in abiertos]
    costes = [[lp(j, 0) - lp(j, r) for r in o] for j, o in zip(abiertos, opts)]
    mejor_s, mejor_g, mejor_c, mejor = g0, g0, 0.0, cero
    for visto, (c, rs) in enumerate(_ordenadas(costes)):
        if visto >= nmax or c - mejor_c > premio - mejor_g:
            break
        if visto == 0:
            continue
        sel = list(cero)
        for j, o, r in zip(abiertos, opts, rs):
            sel[j] = o[r]
        g = leer(sel)
        if g - c > mejor_s:
            mejor_s, mejor_g, mejor_c, mejor = g - c, g, c, sel
    return mejor if mejor_s - g0 > histeresis else cero
