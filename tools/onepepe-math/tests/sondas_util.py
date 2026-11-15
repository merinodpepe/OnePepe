# SPDX-License-Identifier: GPL-3.0-or-later
"""Compositor de «sondas»: renglones en LaTeX simple -> símbolos con caja -> resultado de `pagina.page`.

Es el `build.py` de la auditoría sin rutas fijas. Sirve para probar el motor sin tinta real: la caja de cada símbolo
se calcula con reglas sencillas (eje y hacia abajo, alto de letra H = 20).

Uso:
    from sondas_util import compone, resumen
    r = compone("x=5", r"\\frac{d}{dx}(x^2)=", variables=True)     # lo mismo que pagina.page(filas, **kw)
    resumen(r)                                                     # [(latex del renglón, [(clase, texto), ...])]
Un renglón puede ser un LaTeX (str), una lista de (etiqueta, (x0, y0, x1, y1)) o ya un dict {"symbols": [...]}.

`SONDAS` es la lista compartida con `medir.py --sondas`: cada sonda lleva un `esperado` (regex sobre el texto de la
marca del último renglón); `erroneas()` cuenta las que muestran un resultado que no encaja.
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from onepepe_math import pagina  # noqa: E402
from onepepe_math.layout import Sym  # noqa: E402

H = 20.0
SCRIPT = 0.5
TALL = set("bdfhiklt0123456789()|ABCDEFGHIJKLMNOPQRSTUVWXYZ/") | {r"\partial", r"\delta", r"\theta"}
DESC = set("gjpqy") | {r"\mu", r"\beta", r"\rho"}


def tokeniza(s):
    toks, i = [], 0
    while i < len(s):
        if s[i] == " ":
            i += 1
        elif s[i] == "\\":
            m = re.match(r"\\[A-Za-z]+", s[i:])
            toks.append(m.group(0))
            i += len(m.group(0))
        else:
            toks.append(s[i])
            i += 1
    return toks


def _grupo(toks, i):
    """(tokens del grupo que empieza en toks[i], nuevo i)."""
    if toks[i] != "{":
        return [toks[i]], i + 1
    d, j = 0, i
    while True:
        d += {"{": 1, "}": -1}.get(toks[j], 0)
        if d == 0:
            return toks[i + 1 : j], j + 1
        j += 1


def _caja(lab, x, base, sc):
    h = H * sc
    if lab in TALL:
        y0, y1 = base - h, base
    elif lab == "=":
        y0, y1 = base - 0.45 * h, base - 0.25 * h
    elif lab in ("+", "-", r"\times", r"\div"):
        y0, y1 = base - 0.5 * h, base - 0.2 * h
    elif lab in (".", ","):
        y0, y1 = base - 0.12 * h, base
    elif lab == "'":
        y0, y1 = base - h, base - 0.6 * h
    elif lab in DESC:
        y0, y1 = base - 0.65 * h, base + 0.35 * h
    else:
        y0, y1 = base - 0.65 * h, base
    return (x, y0, x + (3 * sc if lab in (".", ",", "'") else 8 * sc), y1)


def _coloca(toks, x, base, sc, gap=2.0):
    """(símbolos, x final)."""
    out, i, last = [], 0, None
    while i < len(toks):
        t = toks[i]
        i += 1
        if t == r"\frac":
            a, i = _grupo(toks, i)
            b, i = _grupo(toks, i)
            na, wa = _coloca(a, 0, 0, sc)
            nb, wb = _coloca(b, 0, 0, sc)
            w = max(wa, wb) + 14 * sc
            eje = base - 0.35 * H * sc
            for s, ww, dy in ((na, wa, eje - 5 * sc), (nb, wb, eje + 5 * sc + H * sc)):
                dx = x + (w - ww) / 2
                for q in s:
                    q.box = (q.box[0] + dx, q.box[1] + dy, q.box[2] + dx, q.box[3] + dy)
            out += na + nb + [Sym("-", (x, eje - 1, x + w, eje + 1))]
            x += w + gap
            last = None
        elif t == r"\sqrt":
            a, i = _grupo(toks, i)
            na, wa = _coloca(a, x + 10 * sc, base, sc)
            out.append(Sym(r"\sqrt", (x, base - H * sc * 1.25, x + wa + gap + 2 * sc, base + 2 * sc)))
            out += na
            x = wa + gap + 2 * sc
            last = None
        elif t in ("^", "_"):
            a, i = _grupo(toks, i)
            if last is None:
                continue
            bs = last.box
            s2 = sc * SCRIPT
            if last.label in (r"\int", r"\sum"):
                bb = bs[1] + H * s2 * 1.1 if t == "^" else bs[3] + H * s2 * 0.5
            elif t == "^":
                bb = bs[1] + H * s2 * 0.45
            else:
                bb = bs[3] + H * s2 * 0.45 - (0.35 * H * sc if last.label in DESC else 0)
            ns, nw = _coloca(a, bs[2] + 1, bb, s2)
            out += ns
            x = max(x, nw + 1)
        elif t == r"\hat":
            a, i = _grupo(toks, i)
            ns, nw = _coloca(a, x, base, sc)
            b = ns[0]
            out += ns + [Sym(r"\hat", (b.box[0], b.box[1] - 6 * sc, b.box[2], b.box[1] - 2 * sc))]
            x, last = nw, b
        elif t in (r"\int", r"\sum"):
            w = 8 * sc if t == r"\int" else 16 * sc
            b = (x, base - 1.4 * H * sc, x + w, base + 0.4 * H * sc) if t == r"\int" else (
                x, base - H * sc, x + w, base + 0.1 * H * sc)
            last = Sym(t, b)
            out.append(last)
            x += w + gap
        elif t in ("{", "}"):
            continue
        else:
            last = Sym(t, _caja(t, x, base, sc))
            out.append(last)
            x += last.w + gap * sc
    return out, x


def renglon(r, y=0.0):
    """Un renglón en el formato de `pagina.page`, a la altura `y`."""
    if isinstance(r, dict):
        return r
    if isinstance(r, str):
        syms, _ = _coloca(tokeniza(r), 0.0, 24.0 + y, 1.0)
        return {"symbols": [{"latex": s.label, "box": list(s.box)} for s in syms]}
    return {"symbols": [{"latex": l, "box": [b[0], b[1] + y, b[2], b[3] + y]} for l, b in r]}


def compone(*renglones, paso=60.0, **kw):
    """`pagina.page` de los renglones (cada uno `paso` unidades más abajo). `kw`: equals (True), variables, check."""
    kw.setdefault("equals", True)
    return pagina.page([renglon(r, paso * i) for i, r in enumerate(renglones)], **kw)


def resumen(resultado):
    return [(x["latex"], [(m["kind"], m["text"]) for m in x["marks"]]) for x in resultado]


# (nombre, renglones, variables, esperado: regex sobre el texto de la marca del último renglón)
SONDAS = [
    ("paren_mas", ["(x+1)+3="], False, r"^x\s*\+\s*4$"),
    ("segunda_derivada", [r"\frac{d^2}{dx^2}(x^3)="], False, r"^(6\s*\*?\s*x|\?)$"),
    ("dxdt_frac", ["x=t^2", r"\frac{dx}{dt}="], True, r"^\?$"),
    ("dydx_frac", ["y=x^2", r"\frac{dy}{dx}="], True, r"^\?$"),
    ("dxdt_linea", ["x=t^2", "dx/dt="], True, r"^\?$"),
    ("f_de_3", ["f(x)=x^2+1", "f(3)="], True, r"^\?$"),
    ("f_de_x", ["x=5", "f(x)=x^2+1", "f(x)="], True, r"^\?$"),
    ("unidades", ["m=70kg", "g=9,8m/s^2", "mg="], True, r"^686.*N$"),
    ("e_a_la_5", ["e^5="], False, r"148,4"),
    ("exponente_negativo", ["e^{-x}"], False, r"^$"),
    ("prima", ["f'(x)="], False, r"^\?$"),
]


def erroneas(sondas=SONDAS):
    """[(nombre, lo mostrado)] de las sondas cuyo último renglón muestra algo (no «?» ni nada) que no encaja con lo
    esperado. Una sonda que no muestra nada o muestra «?» no cuenta como errónea."""
    malas = []
    for nombre, filas, variables, esperado in sondas:
        try:
            ultimo = compone(*filas, variables=variables)[-1]
        except Exception as e:  # noqa: BLE001 - una excepción también es un resultado malo
            malas.append((nombre, f"EXC {type(e).__name__}"))
            continue
        for m in ultimo["marks"]:
            if m["kind"] not in ("unknown", "variable") and not re.search(esperado, m["text"]):
                malas.append((nombre, m["text"]))
    return malas
