# SPDX-License-Identifier: GPL-3.0-or-later
"""Alineado ordenado (ARQUITECTURA-EJECUCION.md §6 M0): qué trazos forman cada ficha del LaTeX esperado.

A diferencia de `forced.py`, la ficha `j` de un orden de escritura fijo puntúa solo el grupo `j` (programación
dinámica). El orden de escritura no es el del LaTeX (fracciones, raíces, acentos, límites), así que se prueban las
variantes que da `arbol_latex`. También el reparto congelado por hash (examen / validación / pliegues).
"""

import hashlib
import itertools
import math
import re

from .latex_tokens import _INVISIBLE, _TOKEN
from .layout import FUNCTION_ALIASES, FUNCTIONS
from .personal import D_MAX
from .segment import MAX_STROKES, _close, bbox, typical_size
from .symbols import normalize_label

LEJOS = 3.0
UNK = 0.05
MAX_VARIANTES = 64
ACENTOS = {r"\hat", r"\bar", r"\overline", r"\tilde", r"\widehat", r"\widetilde"}

# ---------------------------------------------------------------- árbol y variantes de orden


def _crudos(latex):
    """Como `expected_symbols`, pero conservando llaves, ^ y _ (y quitando lo invisible que no estructura)."""
    latex = re.sub(r"\\(begin|end)\{[a-z]*\}", " ", latex)
    latex = latex.replace(r"\mathrm", "").replace(r"\,", " ").replace(r"\quad", " ").replace(r"\ ", " ")
    toks = []
    for part in re.split(r"(\\mathbb\{[A-Z]\})", latex):
        toks += [part] if part.startswith(r"\mathbb") else _TOKEN.findall(part)
    return [t for t in toks if t in "{}^_" or t not in _INVISIBLE]


class _Parser:
    def __init__(self, latex):
        self.t = _crudos(latex)
        self.i = 0  # posición en self.t
        self.n = 0  # fichas numeradas
        self.sid = 0  # estructuras con variantes
        self.nvar = []

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else None

    def sec(self, stop):
        items = []
        while self.peek() is not None and self.peek() != stop:
            items.append(self.item())
        return ("seq", items)

    def ficha(self, name=None):
        name = name if name is not None else self.t[self.i - 1]
        base = name[1:] if name.startswith("\\") else None
        k = len(base) if base and (base in FUNCTIONS or base in FUNCTION_ALIASES) else 1
        idx = list(range(self.n, self.n + k))
        self.n += k
        return ("leaf", idx)

    def estructura(self, nvars):
        self.nvar.append(nvars)
        return len(self.nvar) - 1

    def grupo(self):
        if self.peek() == "{":
            self.i += 1
            g = self.sec("}")
            self.i += 1
            return g
        return self.base()

    def base(self):
        tok = self.t[self.i]
        self.i += 1
        if tok == "{":
            g = self.sec("}")
            self.i += 1
            return g
        if tok == "}":
            return ("seq", [])
        if tok in ("^", "_"):  # guion sin base: se numera sin estructura
            return self.grupo()
        if tok == r"\frac":
            s = self.estructura(3)
            raya = self.n
            self.n += 1
            return ("frac", s, raya, self.grupo(), self.grupo())
        if tok == r"\sqrt":
            raiz = self.n
            self.n += 1
            indice = None
            if self.peek() == "[":
                self.i += 1
                ini = self.ficha("[")
                cuerpo = self.sec("]")
                self.i += 1
                indice = ("seq", [ini, cuerpo, self.ficha("]")])
            s = self.estructura(2)
            return ("sqrt", s, raiz, indice, self.grupo())
        if tok in ACENTOS:
            s = self.estructura(2)
            acento = self.n
            self.n += 1
            return ("acc", s, acento, self.grupo())
        return self.ficha(tok)

    def item(self):
        b = self.base()
        sub = sup = None
        orden = []
        while self.peek() in ("^", "_") and self.peek() not in orden:
            orden.append(self.peek())
            self.i += 1
            if orden[-1] == "_":
                sub = self.grupo()
            else:
                sup = self.grupo()
        if sub is None and sup is None:
            return b
        return ("script", self.estructura(2 if sub and sup else 1), b, sub, sup)


def _arbol(latex):
    p = _Parser(latex)
    nodo = p.sec(None)
    return nodo, p.n, p.nvar


def arbol_latex(latex):
    """Analiza la cadena con la gramática de M0.3. Devuelve (nodo, n_fichas); las fichas van numeradas en el orden
    de `expected_symbols`."""
    return _arbol(latex)[:2]


def _emitir(nodo, ch):
    """Índices de ficha del nodo, en el orden de escritura que marca la elección `ch[estructura]`."""
    k = nodo[0]
    if k == "leaf":
        return list(nodo[1])
    if k == "seq":
        return [i for x in nodo[1] for i in _emitir(x, ch)]
    c = ch[nodo[1]]
    if k == "frac":
        r, n, d = [nodo[2]], _emitir(nodo[3], ch), _emitir(nodo[4], ch)
        return [n + r + d, r + n + d, n + d + r][c]
    if k == "sqrt":
        r, ind, cuerpo = [nodo[2]], _emitir(nodo[3], ch) if nodo[3] else [], _emitir(nodo[4], ch)
        return [ind + r + cuerpo, r + ind + cuerpo][c]
    if k == "acc":
        b, a = _emitir(nodo[3], ch), [nodo[2]]
        return [b + a, a + b][c]
    b = _emitir(nodo[2], ch)  # script
    sub, sup = (_emitir(x, ch) if x else [] for x in nodo[3:])
    return b + (sub + sup if c == 0 else sup + sub)


def _estructuras(latex, tokens):
    """(nodo, nº de variantes de cada estructura) o (None, []) si el árbol no cuadra con las fichas."""
    try:
        nodo, n, nvar = _arbol(latex)
    except IndexError:
        return None, []
    if n != len(tokens) or sorted(_emitir(nodo, [0] * len(nvar))) != list(range(n)):
        return None, []
    return nodo, nvar


def variantes_de_orden(latex, tokens):
    """Permutaciones de las fichas (orden de escritura) a probar: todas si el producto es <= MAX_VARIANTES; si no,
    solo la de partida (`ordered_groups` hace entonces el descenso por coordenadas)."""
    nodo, nvar = _estructuras(latex, tokens)
    if nodo is None:
        yield list(range(len(tokens)))
    elif math.prod(nvar) <= MAX_VARIANTES:
        for ch in itertools.product(*(range(v) for v in nvar)):
            yield _emitir(nodo, ch)
    else:
        yield _emitir(nodo, [0] * len(nvar))


# ---------------------------------------------------------------- programación dinámica


class _Puntuador:
    """Probabilidad por etiqueta de cada grupo de trazos (caché por los índices originales de los trazos)."""

    def __init__(self, strokes, classifier):
        self.strokes = strokes
        self.clf = classifier
        self.labels = list(getattr(classifier, "labels", classifier.classes))
        self.por_etiqueta = {}
        for i, lab in enumerate(self.labels):
            self.por_etiqueta.setdefault(lab, []).append(i)
        self.cache, self.cerca = {}, {}
        self.boxes = [bbox([s]) for s in strokes]
        self.tipico = typical_size(strokes) if strokes else 1.0

    def p(self, ids):
        if ids not in self.cache:
            v = self.clf.probabilities([self.strokes[i] for i in ids])
            self.cache[ids] = {lab: float(sum(v[i] for i in ix)) for lab, ix in self.por_etiqueta.items()}
        return self.cache[ids]

    def encadenados(self, ids):
        if ids not in self.cerca:
            self.cerca[ids] = all(
                any(_close(self.boxes[ids[j]], self.boxes[ids[i]], self.tipico) for j in range(i))
                for i in range(1, len(ids))
            )
        return self.cerca[ids]

    def coste(self, ids, t):
        """(g, p, desconocido) de que el grupo `ids` sea la ficha `t`."""
        if t not in self.por_etiqueta:
            return math.log(UNK), UNK, True
        p = self.p(ids)[t]
        lejos = len(ids) > 1 and not self.encadenados(ids)
        return math.log(max(p, 1e-9)) - (LEJOS if lejos else 0.0), p, False


def _dp(pt, orden, v, tokens):
    """Mejor reparto de los trazos `orden` (índices originales) entre las fichas `tokens[v[0]]`, ... Devuelve
    (total, [(índices, ficha)]) o None."""
    n, m = len(orden), len(v)
    if m == 0 or not m <= n <= MAX_STROKES * m:
        return None
    B = [[-math.inf] * (m + 1) for _ in range(n + 1)]
    R = [[0] * (m + 1) for _ in range(n + 1)]
    B[0][0] = 0.0
    for i in range(1, n + 1):
        for j in range(1, min(m, i) + 1):
            for k in range(1, min(MAX_STROKES, i) + 1):
                if B[i - k][j - 1] == -math.inf:
                    continue
                s = B[i - k][j - 1] + pt.coste(tuple(orden[i - k : i]), tokens[v[j - 1]])[0]
                if s > B[i][j]:
                    B[i][j], R[i][j] = s, k
    if B[n][m] == -math.inf:
        return None
    grupos, i = [], n
    for j in range(m, 0, -1):
        k = R[i][j]
        grupos.append((tuple(orden[i - k : i]), v[j - 1]))
        i -= k
    return B[n][m], grupos[::-1]


def _mejor_orden(pt, orden, tokens, latex, variantes):
    """Prueba las variantes de orden de escritura; devuelve (total, grupos, v) del mejor."""
    ident = list(range(len(tokens)))
    nodo, nvar = _estructuras(latex, tokens) if (variantes and latex is not None) else (None, [])
    mejor = None

    def probar(v):
        nonlocal mejor
        r = _dp(pt, orden, v, tokens)
        if r and (mejor is None or r[0] > mejor[0]):
            mejor = (r[0], r[1], v)
        return r[0] if r else -math.inf

    if nodo is None:
        probar(ident)
    elif math.prod(nvar) <= MAX_VARIANTES:
        for ch in itertools.product(*(range(v) for v in nvar)):
            probar(_emitir(nodo, ch))
    else:  # descenso por coordenadas, dos pasadas
        ch = [0] * len(nvar)
        probar(_emitir(nodo, ch))
        for _ in range(2):
            for s in range(len(nvar)):
                top, mejor_c = -math.inf, ch[s]
                for c in range(nvar[s]):
                    t = probar(_emitir(nodo, ch[:s] + [c] + ch[s + 1 :]))
                    if t > top:
                        top, mejor_c = t, c
                ch[s] = mejor_c
    return mejor


def _resultado(pt, total, grupos, tokens, movido):
    ps, desc, out = [], [], []
    for ids, f in grupos:
        _, p, d = pt.coste(ids, tokens[f])
        ps.append(p)
        desc.append(d)
        out.append((list(ids), tokens[f]))
    return {
        "grupos": out, "p": ps, "total": total, "media": sum(ps) / len(ps), "min": min(ps),
        "orden": [f for _, f in grupos], "movido": movido, "desconocido": desc,
    }


def ordered_groups(strokes, tokens, classifier, latex=None, variantes=True, tardios=False):
    """Reparte los trazos (en orden de escritura) entre las fichas `tokens`. Dict de M0.3 o None si es imposible.
    Sin `latex` o con `variantes=False` solo se prueba el orden del LaTeX."""
    n = len(strokes)
    if not tokens or not len(tokens) <= n <= MAX_STROKES * len(tokens):
        return None
    pt = _Puntuador(strokes, classifier)
    orden = list(range(n))
    r = _mejor_orden(pt, orden, tokens, latex, variantes)
    if r is None:
        return None
    movido = None
    res = _resultado(pt, r[0], r[1], tokens, movido)
    if tardios and res["min"] < 0.2:
        mejor = (r[0] + 1.0, None)  # solo se acepta un movimiento que mejore en más de 1,0
        for rr in range(n):
            for q in range(max(0, rr - 6), rr):
                nuevo = orden[:q] + [rr] + orden[q:rr] + orden[rr + 1 :]
                x = _mejor_orden(pt, nuevo, tokens, latex, variantes)
                if x and x[0] > mejor[0]:
                    mejor = (x[0], (rr, q, x))
        if mejor[1]:
            rr, q, x = mejor[1]
            res = _resultado(pt, x[0], x[1], tokens, (rr, q))
    return res


def aceptados(res, strokes, classifier, personal=None):
    """Control de calidad (receta 3 de la auditoría): [bool] por símbolo del alineado, en orden de escritura."""
    pt = _Puntuador(strokes, classifier)
    A = []
    for ids, lab in res["grupos"]:
        top5 = sorted(pt.p(tuple(ids)).items(), key=lambda x: -x[1])[:5]
        ok = lab in {t for t, _ in top5}
        if not ok and personal is not None and len(personal):
            vec = personal.neighbors(classifier.embedding([strokes[i] for i in ids]), 1)
            ok = bool(vec) and vec[0][0] <= D_MAX and normalize_label(vec[0][1]) == lab
        A.append(ok)
    ext = [True] + A + [True]
    return [A[j] or (ext[j] and ext[j + 2]) for j in range(len(A))]


# ---------------------------------------------------------------- reparto congelado por hash


def u(sal, clave):
    """Número fijo en [0, 1) para cada expresión."""
    return int(hashlib.sha1(f"{sal}:{clave}".encode()).hexdigest()[:8], 16) / 2**32


def pliegue(clave):
    return int(hashlib.sha1(("onepepe-pliegues-v1:" + clave).encode()).hexdigest()[:8], 16) % 4


def _elegir(ids, sal, fraccion):
    """Los primeros int(fraccion·N + 0,5) ids por (u, id). ids: [(set, id)]."""
    orden = sorted(ids, key=lambda x: (u(sal, f"{x[0]}#{x[1]}"), x[1]))
    return orden[: int(fraccion * len(orden) + 0.5)]


def reparto(sets):
    """{set: [(id, kind)]} -> {"examen": {set: [ids]}, "validacion": {set: [ids]}} por estratos (set, kind)."""
    out = {"examen": {s: [] for s in sets}, "validacion": {s: [] for s in sets}}
    for s, items in sets.items():
        for kind in sorted({k for _, k in items}):
            ids = [(s, i) for i, k in items if k == kind]
            ex = _elegir(ids, "onepepe-examen-v1", 0.20)
            va = _elegir([x for x in ids if x not in ex], "onepepe-validacion-v1", 0.15)
            out["examen"][s] += [i for _, i in ex]
            out["validacion"][s] += [i for _, i in va]
    return out
