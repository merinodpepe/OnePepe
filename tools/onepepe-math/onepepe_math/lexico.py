# SPDX-License-Identifier: GPL-3.0-or-later
"""Léxico: números de varias cifras, cifras mal leídas dentro de un número y derivadas de Leibniz.

No importa `layout` ni `evaluate` al cargarse (ellos sí lo importan a él): lo que necesita lo recibe o lo importa
dentro de la función.
"""

import math

from .symbols import DIGITS


# ---------- números (los usa layout._parse_row) ----------


def _cifra(s):
    return s.label in DIGITS and getattr(s, "node", None) is None


def _a_ras(a, b):
    """¿Tienen el fondo a la misma altura? (|fondo_b - fondo_a| <= 0,2 * el mayor alto)"""
    return abs(b.box[3] - a.box[3]) <= 0.2 * max(a.h, b.h)


def unir_numeros(syms, tipico, crear):
    """Junta las cifras seguidas de una línea (ordenadas por x) en símbolos con `node = ("num", "12.5")`.

    Dos cifras son del mismo número si van a ras de la misma base y el hueco es menor que 0,8 * típico. Una coma
    pequeña (alto < 0,4 * típico, a ras de la base) entre dos cifras con huecos < 0,6 * típico es el separador
    decimal. El punto no: multiplica y corta el número. `crear(label, box, node)` fabrica el Sym de la unión (su
    etiqueta es la de la última cifra, para que el resto del diseño lo trate como cifra).
    """
    out, i = [], 0
    while i < len(syms):
        s = syms[i]
        if not _cifra(s):
            out.append(s)
            i += 1
            continue
        grupo, texto, i = [s], s.label, i + 1
        while i < len(syms):
            a, b = grupo[-1], syms[i]
            if _cifra(b) and _a_ras(a, b) and b.box[0] - a.box[2] < 0.8 * tipico:
                grupo.append(b)
                texto += b.label
                i += 1
                continue
            c = b
            d = syms[i + 1] if i + 1 < len(syms) else None
            if (
                c.label == ","
                and "." not in texto
                and d is not None
                and _cifra(d)
                and c.h < 0.4 * tipico
                and abs(c.box[3] - a.box[3]) <= 0.4 * a.h
                and c.box[0] - a.box[2] < 0.6 * tipico
                and d.box[0] - c.box[2] < 0.6 * tipico
                and _a_ras(a, d)
            ):
                grupo += [c, d]
                texto += "." + d.label
                i += 2
                continue
            break
        box = (min(g.box[0] for g in grupo), min(g.box[1] for g in grupo),
               max(g.box[2] for g in grupo), max(g.box[3] for g in grupo))
        nuevo = crear(grupo[-1].label, box, ("num", texto))
        nuevo.members = tuple(m for g in grupo for m in getattr(g, "members", ()))
        out.append(nuevo)
    return out


# ---------- cifras mal leídas dentro de un número (las usa recognizer.recognize) ----------

PARECIDOS = {"S": "5", "O": "0", "o": "0", "I": "1", "|": "1", "l": "1", "q": "9", "g": "9", "b": "6", "B": "8",
             "Z": "2", "z": "2", "s": "5"}
_RARAS = set("SOoI|qBZ")
THETA = 2.5


def _vecinos(i, orden, cand_lists, boxes, fijos, tipico):
    """Cajas de los vecinos inmediatos de `i` (en orden de lectura) que son cifras seguras a ras y pegadas."""
    k = orden.index(i)
    b = boxes[i]
    out = []
    for j, lado in ((k - 1, -1), (k + 1, 1)):
        if not 0 <= j < len(orden):
            continue
        v = orden[j]
        c = boxes[v]
        hueco = b[0] - c[2] if lado < 0 else c[0] - b[2]
        lab, p = cand_lists[v][0]
        if lab in DIGITS and (p >= 0.5 or v in fijos) and hueco < 0.8 * tipico \
                and abs(c[3] - b[3]) <= 0.2 * max(c[3] - c[1], b[3] - b[1]):
            out.append(c)
    return out


def corregir_entidades(cand_lists, boxes, tipico, fijos=(), theta=THETA):
    """Cambia a su cifra parecida (S->5, O->0, l->1...) la letra que queda entre cifras.

    s = ln((p_d + 0,02) / p_L) + 2 * n_cifras + geom + pi_L - theta; se cambia a la cifra d si s > 0.
    `fijos`: índices que no se tocan (símbolos ya reconocidos antes). Devuelve nuevas candidatas.
    """
    orden = sorted(range(len(cand_lists)), key=lambda i: boxes[i][0] + boxes[i][2])
    out = list(cand_lists)
    for i, cands in enumerate(cand_lists):
        if i in fijos or not cands or cands[0][0] not in PARECIDOS:
            continue
        L, pL = cands[0]
        d = PARECIDOS[L]
        vec = _vecinos(i, orden, cand_lists, boxes, fijos, tipico)
        b = boxes[i]
        h = b[3] - b[1]
        geom = int(bool(vec) and all(
            h >= 0.8 * (c[3] - c[1]) and abs(b[3] - c[3]) <= 0.2 * (c[3] - c[1]) for c in vec))
        pd = sum(p for lab, p in cands if lab == d)
        s = math.log((pd + 0.02) / max(pL, 1e-9)) + 2.0 * len(vec) + geom + (1.5 if L in _RARAS else 0.0) - theta
        if s > 0:
            out[i] = [(d, pd or 0.02)] + [(lab, p) for lab, p in cands if lab != d]
    return out


# ---------- derivadas de Leibniz (las usa evaluate._tokens) ----------

_D = ("d", r"\partial")


def _pow(t):
    """(letra, n) de una ficha letra con exponente entero opcional (n = 1 si no lo tiene), o None."""
    if t[0] != "sym" or t[3] is not None:
        return None
    if t[2] is None:
        return t[1], 1
    from .evaluate import ParseError, _plain

    try:
        return t[1], int(_plain(t[2]))
    except (ParseError, ValueError):
        return None


def _letra(t):
    return t[0] == "sym" and t[2] is None and t[3] is None and t[1].isalpha() and len(t[1]) == 1


def _deriv_frac(t):
    """(f, [(v, n)]) si la fracción es una derivada de Leibniz (d/dx, d²f/dx², ∂²f/∂x∂y), si no None."""
    from .evaluate import _tokens

    _, num, den = t[1]
    nt, dt = _tokens(num), _tokens(den)
    a = _pow(nt[0]) if nt else None
    if not a or a[0] not in _D or len(nt) > 2 or (len(nt) == 2 and not _letra(nt[1])):
        return None
    d, n = a
    f = nt[1][1] if len(nt) == 2 else None
    if len(dt) == 2 and dt[0][:4] == ["sym", d, None, None]:
        v = _pow(dt[1])
        if v and v[1] == n and len(v[0]) == 1 and v[0].isalpha():
            return f, [v]
    if d == r"\partial" and n == 2 and len(dt) == 4 and all(dt[k][:4] == ["sym", d, None, None] for k in (0, 2)) \
            and _letra(dt[1]) and _letra(dt[3]):
        return f, [(dt[1][1], 1), (dt[3][1], 1)]
    return None


def _leibniz(tokens, definidos=()):
    """Derivadas de Leibniz como una sola ficha ["deriv", (f o None, [(variable, orden)]), None, None]: d/dx, dx/dt,
    d²f/dx², ∂²f/∂x∂y y en línea dx/dt. No se aplica si «d» es una variable de la página."""
    if "d" in definidos:
        return tokens
    out, i = [], 0
    while i < len(tokens):
        t = tokens[i]
        w = tokens[i : i + 5]
        d = _deriv_frac(t) if t[0] == "frac" and t[2] is None and t[3] is None else None
        if d:
            out.append(["deriv", d, None, None])
        elif (
            len(w) == 5
            and all(_letra(x) for x in (w[0], w[1], w[3], w[4]))
            and w[0][1] == w[3][1] == "d"
            and w[2][:4] == ["sym", "/", None, None]
        ):
            out.append(["deriv", (w[1][1], [(w[4][1], 1)]), None, None])
            i += 4
        else:
            out.append(t)
        i += 1
    return out
