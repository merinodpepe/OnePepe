# SPDX-License-Identifier: GPL-3.0-or-later
"""Estructura espacial de una expresión: línea base, potencias, subíndices, fracciones y raíces.

Entrada: símbolos ya reconocidos, cada uno con su etiqueta LaTeX y su caja (x0, y0, x1, y1), eje y hacia abajo.
Salida: un árbol sencillo y su LaTeX. Las tolerancias son amplias porque la letra a mano se mueve.

Nodos del árbol (tuplas):
    ("sym", latex)                      símbolo
    ("num", "12.5")                     número de varias cifras (la coma decimal se guarda como punto)
    ("row", [nodos])                    secuencia en la misma línea
    ("sup", base, exponente)            potencia
    ("sub", base, subíndice)
    ("subsup", base, subíndice, exponente)
    ("frac", numerador, denominador)
    ("sqrt", radicando)
    ("root", índice, radicando)          raíz n-ésima
    ("accent", acento, base)             ŷ, x̄, x̃ (acento: \\hat, \\overline, \\tilde)
"""

import re
from dataclasses import dataclass

from .lexico import unir_numeros
from .symbols import ACCENTS, DIGITS, category


@dataclass
class Sym:
    label: str
    box: tuple  # (x0, y0, x1, y1)
    # otras lecturas posibles del símbolo (sus candidatas), para unir funciones mal leídas («1n» → ln)
    alts: tuple = ()

    @property
    def cx(self):
        return (self.box[0] + self.box[2]) / 2

    @property
    def cy(self):
        return (self.box[1] + self.box[3]) / 2

    @property
    def w(self):
        return self.box[2] - self.box[0]

    @property
    def h(self):
        return self.box[3] - self.box[1]


# Símbolos que no tienen "cuerpo" vertical propio: su caja no sirve para decidir potencias.
_FLAT = {"-", r"\frac", "=", ".", ",", r"\cdot", "'", r"\sim", r"\approx", r"\equiv"}
# Símbolos con parte descendente o ascendente; su caja vertical engaña para encontrar el centro de la línea.
_DESC = set("gjpqy") | {r"\beta", r"\eta", r"\mu", r"\rho", r"\phi", r"\varphi", r"\chi", r"\psi", r"\gamma"}
_ASC = set("bdfhklt") | set("0123456789") | set("ABCDEFGHIJKLMNOPQRSTUVWXYZ") | {r"\delta", r"\theta", r"\partial"}


def _typical_height(syms):
    hs = sorted(s.h for s in syms if s.label not in _FLAT)
    return hs[len(hs) // 2] if hs else max((s.w for s in syms), default=1.0)


def _middle(s, typical):
    """Altura del centro de la línea a la que pertenece el símbolo (corrige letras con rabo hacia arriba o abajo)."""
    if getattr(s, "accent_base", None) is not None:
        return _middle(s.accent_base, typical)
    if s.label in _FLAT:
        return s.cy
    if s.label in _DESC:
        return s.box[1] + 0.3 * s.h
    if s.label in _ASC and s.h > 0.8 * typical:
        return s.box[3] - 0.35 * s.h
    return s.cy


class _Hoja(tuple):
    """Nodo del árbol con los índices (`members`) de los símbolos que lo originan: da el orden LaTeX (M10.3)."""

    def __new__(cls, nodo, members):
        t = super().__new__(cls, nodo)
        t.members = members
        return t


def _miembros(*syms):
    return tuple(m for s in syms for m in getattr(s, "members", ()))


def _recorrer(n):
    yield from getattr(n, "members", ())
    if n[0] in ("sym", "num"):
        return
    for h in n[1] if n[0] == "row" else n[1:]:
        if isinstance(h, tuple):
            yield from _recorrer(h)


def parse_with_order(syms):
    """(árbol, orden): los índices de `syms` en el orden del LaTeX (raya, numerador, denominador; raíz, índice,
    radicando; acento, base; base, subíndice, exponente). Los que el árbol descarta (primas) van al final."""
    syms = list(syms)
    for i, s in enumerate(syms):
        s.members = (i,)
    tree = parse(syms)
    orden = list(_recorrer(tree))
    resto = sorted(set(range(len(syms))) - set(orden), key=lambda i: syms[i].cx)
    return tree, orden + resto


def parse(syms):
    """Árbol de la expresión."""
    syms = list(syms)
    if not syms:
        return ("row", [])
    typical = _typical_height(syms)
    return _parse_region(syms, typical)


def _big_height(syms, typical):
    alturas = sorted(s.h for s in syms if s.label not in _FLAT)
    return max(typical, alturas[int(0.9 * (len(alturas) - 1))]) if alturas else typical


def _frac_sides(bar, syms, typical, H):
    """Lo que hay encima y debajo de una raya, dentro de su ancho y cerca (si hay de los dos lados, es fracción)."""
    x0, x1 = bar.box[0] - 0.1 * bar.w, bar.box[2] + 0.1 * bar.w
    above = [s for s in syms if s is not bar and x0 <= s.cx <= x1 and s.box[3] <= bar.cy + 0.15 * typical]
    below = [s for s in syms if s is not bar and x0 <= s.cx <= x1 and s.box[1] >= bar.cy - 0.15 * typical]
    # solo lo que está cerca de la raya en vertical
    return [s for s in above if bar.cy - s.box[3] < 1.5 * H], [s for s in below if s.box[1] - bar.cy < 1.5 * H]


def _accent_below(acc, syms, typical):
    """Los símbolos que quedan justo debajo de un posible acento."""
    return [
        s
        for s in syms
        if s is not acc
        and s.label not in ACCENTS
        and s.box[0] - 0.35 * s.w <= acc.cx <= s.box[2] + 0.35 * s.w
        and -0.15 * typical <= s.box[1] - acc.box[3] < 0.6 * typical
        and acc.w <= 2.2 * max(s.w, 0.4 * typical)
    ]


def bar_role(box, others, typical):
    """Papel de una raya (M5.5): "frac" (algo encima y debajo), "overline" (solo una letra debajo, y no mucho más
    ancha) o "minus". `others`: los demás Sym de la región. Usa los mismos umbrales que `_parse_region`."""
    bar = Sym("-", tuple(box))
    if bar.w > 0.8 * typical:
        above, below = _frac_sides(bar, others, typical, _big_height(others, typical))
        if above and below:
            return "frac"
    below = _accent_below(bar, others, typical)
    if len(below) == 1:
        over = [s for s in others if s.box[3] <= bar.box[1] and abs(s.cx - bar.cx) < bar.w]
        if not over and bar.w <= 1.6 * max(below[0].w, 0.4 * typical) and below[0].label not in _FLAT:
            return "overline"
    return "minus"


def _parse_region(syms, typical):
    syms = sorted(syms, key=lambda s: s.box[0])
    # 0. límites de ∑, ∏ y ∫ debajo y encima del signo
    for big in [s for s in syms if s.label in (r"\sum", r"\prod", r"\int") and not hasattr(s, "limits_done")]:
        near = [
            s
            for s in syms
            if s is not big
            and s.h < 0.8 * big.h
            and min(s.box[2], big.box[2] + 0.6 * typical) - max(s.box[0], big.box[0] - 0.6 * typical) > 0.5 * s.w
        ]
        below = [s for s in near if s.box[1] >= big.box[3] - 0.10 * big.h and s.box[1] - big.box[3] < typical]
        above = [s for s in near if s.box[3] <= big.box[1] + 0.10 * big.h and big.box[1] - s.box[3] < typical]
        if below or above:
            used = below + above
            node = Sym(label=big.label, box=(min(s.box[0] for s in used + [big]), min(s.box[1] for s in used + [big]),
                                              max(s.box[2] for s in used + [big]), max(s.box[3] for s in used + [big])))
            base = _node(big)
            lo = _parse_region(below, typical) if below else None
            up = _parse_region(above, typical) if above else None
            node.node = ("subsup", base, lo, up) if lo and up else ("sub", base, lo) if lo else ("sup", base, up)
            node.limits_done = True
            return _parse_region([s for s in syms if s is not big and s not in used] + [node], typical)
    # 1. fracciones: la raya más larga primero
    H = _big_height(syms, typical)
    bars = sorted(
        [s for s in syms if s.label in ("-", r"\frac") and s.w > 0.8 * typical],
        key=lambda s: -s.w,
    )
    for bar in bars:
        if bar not in syms:
            continue
        above, below = _frac_sides(bar, syms, typical, H)
        if above and below:
            num = _parse_region(above, typical)
            den = _parse_region(below, typical)
            box = (
                min(bar.box[0], *(s.box[0] for s in above + below)),
                min(s.box[1] for s in above),
                max(bar.box[2], *(s.box[2] for s in above + below)),
                max(s.box[3] for s in below),
            )
            node = Sym(label="", box=box)
            node.node = _Hoja(("frac", num, den), _miembros(bar))
            rest = [s for s in syms if s is not bar and s not in above and s not in below]
            return _parse_region(rest + [node], typical)
    # 2. acentos: un sombrero, una raya o una virgulilla pequeños justo encima de un símbolo (ŷ, x̄, x̃). La raya de
    # una fracción tiene algo debajo y encima; la de una media, solo su letra debajo y más estrecha.
    for acc in [s for s in syms if s.label in ACCENTS or s.label == "-"]:
        if acc not in syms:
            continue
        below = _accent_below(acc, syms, typical)
        if len(below) != 1:
            continue
        base = below[0]
        # la raya: solo si es corta (como su letra) y no hay nada encima (si no, es una fracción o un «=»)
        if acc.label == "-":
            over = [s for s in syms if s is not acc and s.box[3] <= acc.box[1] and abs(s.cx - acc.cx) < acc.w]
            if over or acc.w > 1.6 * max(base.w, 0.4 * typical) or base.label in _FLAT:
                continue
        kind = r"\overline" if acc.label == "-" else acc.label
        node = Sym(label="", box=(min(acc.box[0], base.box[0]), acc.box[1], max(acc.box[2], base.box[2]), base.box[3]))
        node.node = _Hoja(("accent", kind, _node(base)), _miembros(acc))
        node.accent_base = base
        rest = [s for s in syms if s is not acc and s is not base]
        return _parse_region(rest + [node], typical)
    # 3. raíces: lo que está dentro de la caja de la √ y a la derecha de su gancho
    for root in [s for s in syms if s.label == r"\sqrt"]:
        inside = [
            s
            for s in syms
            if s is not root
            and s.box[0] >= root.box[0] + 0.2 * root.w
            and s.box[2] <= root.box[2] + 0.2 * typical
            and s.cy >= root.box[1]
            and s.cy <= root.box[3] + 0.1 * typical
        ]
        if inside:
            # índice: algo pequeño encima del gancho, a la izquierda de la raíz (∛)
            index = [
                s
                for s in syms
                if s is not root
                and s not in inside
                and s.cx <= root.box[0] + 0.45 * root.w
                and s.box[2] <= root.box[0] + 0.6 * root.w
                and s.box[3] <= root.cy + 0.1 * typical
                and s.box[1] >= root.box[1] - 0.8 * typical
                and s.h < 0.75 * typical
                and s.label not in _FLAT
            ]
            node = Sym(label="", box=root.box)
            radicand = _parse_region(inside, typical)
            nodo = ("root", _parse_region(index, typical), radicand) if index else ("sqrt", radicand)
            node.node = _Hoja(nodo, _miembros(root))
            rest = [s for s in syms if s is not root and s not in inside and s not in index]
            return _parse_region(rest + [node], typical)
    # 4. línea base con potencias y subíndices
    return _parse_row(sorted(syms, key=lambda s: s.box[0]), typical)


def _node(s):
    n = getattr(s, "node", None) or ("sym", s.label)
    m = getattr(s, "members", None)
    return n if m is None or isinstance(n, _Hoja) else _Hoja(n, m)


FUNCTIONS = ["sin", "cos", "tan", "cot", "sec", "csc", "sinh", "cosh", "tanh", "arcsin", "arccos", "arctan",
             "ln", "log", "exp", "lim", "max", "min", "det"]
# nombres en español: se escriben así y se guardan con el nombre de LaTeX (sen x → \sin x)
FUNCTION_ALIASES = {"sen": "sin", "tg": "tan", "arcsen": "arcsin", "arctg": "arctan", "cotg": "cot",
                    "senh": "sinh", "tgh": "tanh"}
# no pueden ser exponente ni subíndice por sí solos
_NEVER_SCRIPT = {",", ".", r"\cdot", "=", "<", ">", r"\leq", r"\geq", r"\neq", r"\approx", r"\rightarrow", r"\Rightarrow",
                 r"\Leftrightarrow", ";", ":"}
# letras que la red confunde con otros símbolos que se escriben igual: «l» con «1» o «|», «o» con «0»
_LOOKALIKE = {"l": {"1", "|", "I"}, "o": {"0", "O"}, "s": {"5", "S"}, "g": {"9"}, "i": {"1"}}


def _letter_matches(sym, letter, strict):
    if sym.label == letter:
        return True
    if strict or sym.label not in _LOOKALIKE.get(letter, ()):
        return False
    # leído como «1», pero «l» era una de sus candidatas (si no se conocen, se acepta)
    return not sym.alts or letter in sym.alts


def _merge_functions(syms, typical):
    """Une letras seguidas que forman sin, cos, ln, lim... en un solo símbolo (\\sin, \\cos...).

    Acepta las letras leídas como otro símbolo que se escribe igual («1n» → ln, «c0s» → cos) cuando la palabra
    entera encaja y la letra estaba entre las candidatas.
    """
    names = sorted(set(FUNCTIONS) | set(FUNCTION_ALIASES), key=len, reverse=True)
    out = []
    i = 0
    while i < len(syms):
        merged = False
        for name in names:
            n = len(name)
            group = syms[i : i + n]
            if len(group) != n:
                continue
            exact = all(_letter_matches(s, c, True) for s, c in zip(group, name))
            # la tolerancia solo con palabras de 2 o más letras que no son todas parecidas («10» no es «lo»)
            loose = not exact and all(_letter_matches(s, c, False) for s, c in zip(group, name))
            loose = loose and sum(s.label == c for s, c in zip(group, name)) >= max(1, n - 1)
            if exact or loose:
                gaps_ok = all(b.box[0] - a.box[2] < 0.6 * typical for a, b in zip(group, group[1:]))
                if gaps_ok:
                    box = (
                        min(s.box[0] for s in group),
                        min(s.box[1] for s in group),
                        max(s.box[2] for s in group),
                        max(s.box[3] for s in group),
                    )
                    fn = Sym("\\" + FUNCTION_ALIASES.get(name, name), box)
                    # la línea base de la palabra, sin contar las letras que bajan (la «g» de log)
                    fn.members = _miembros(*group)
                    fn.baseline = max((s.box[3] for s in group if s.label not in _DESC), default=box[3])
                    out.append(fn)
                    i += n
                    merged = True
                    break
        if not merged:
            out.append(syms[i])
            i += 1
    return out


def is_function(label):
    return label.startswith("\\") and label[1:] in FUNCTIONS


_BIGLIM = {r"\sum", r"\prod", r"\lim"}


def _is_op(label):
    return category(label) in {"binop", "relation"} or label in {r"\cdot", r"\sim", r"\equiv"}


def _crear_num(label, box, node):
    s = Sym(label, box)
    s.node = node
    return s


def _parse_row(syms, typical):
    syms = unir_numeros(_merge_functions(syms, typical), typical, _crear_num)
    items = []
    i = 0
    while i < len(syms):
        base = syms[i]
        node = _node(base)
        i += 1
        if getattr(base, "limits_done", False):
            items.append((base, node))
            continue
        if base.label in _FLAT or category(base.label) in {"binop", "relation", "open"}:
            items.append((base, node))
            continue
        base_mid = _middle(base, typical)
        base_top, base_bot = base.box[1], base.box[3]
        sup, sub, primes = [], [], 0
        # los símbolos siguientes más pequeños y desplazados hacia arriba (o abajo) forman el exponente (o subíndice)
        while i < len(syms):
            s = syms[i]
            if s.label in _NEVER_SCRIPT and not (
                # límites de ∑, ∏ y lim escritos a la derecha: «i=1» y «x→0» siguen en el subíndice
                sub and base.label in _BIGLIM and s.label in ("=", r"\rightarrow") and s.cy > base_mid + 0.12 * typical
            ):
                break
            if s.box[0] > base.box[2] + 1.2 * typical and not (sup or sub):
                break
            # prima: f', f'', ŷ', (x)'
            if s.label == "'" and s.box[3] < base_mid and (
                category(base.label) in {"latin", "greek"} or base.label == ")" or hasattr(base, "accent_base")
            ):
                primes += 1
                i += 1
                continue
            # una función de letras bajas tras un paréntesis («sin(x)cos(x)») sigue en la línea; solo es exponente
            # si queda entera por encima del centro (e^{sin x})
            if is_function(s.label) and not s.box[3] < base_mid:
                break
            # cifra tras cifra: sigue en la línea (mismo número) salvo que esté claramente subida (exponente)
            if base.label in DIGITS and s.label in DIGITS:
                mismo = abs(s.box[3] - base.box[3]) <= 0.2 * max(base.h, typical)
                sube = s.box[3] < base.box[3] - 0.35 * base.h and s.h < 0.85 * max(base.h, typical)
                if mismo or not sube:
                    break
                sup.append(s)
                i += 1
                continue
            # un operador solo sigue en un guion ya abierto; uno nuevo solo abre un exponente negativo (e^{-x})
            if _is_op(s.label):
                nxt = syms[i + 1] if i + 1 < len(syms) else None
                if sup:
                    ok, dest = s.cy < base_mid - 0.25 * typical, sup
                elif sub:
                    ok, dest = s.cy > base_mid + 0.12 * typical, sub
                else:
                    ok, dest = (
                        s.label in ("-", "+")
                        and s.cy < base_top + 0.35 * base.h
                        and nxt is not None
                        and not _is_op(nxt.label)
                        and nxt.h < 0.85 * max(base.h, typical)
                        and _middle(nxt, typical) < base_mid - 0.25 * typical
                        and nxt.box[3] < base_bot - 0.05 * typical
                    ), sup
                if not ok:
                    break
                dest.append(s)
                i += 1
                continue
            smaller = s.h < 0.85 * max(base.h, typical)
            mid = _middle(s, typical)
            if is_function(base.label):
                # tras una función, lo que sigue es su argumento (sin x, ln 2); solo es subíndice lo que queda
                # claramente por debajo de la línea base (la base de log₂)
                baseline = getattr(base, "baseline", base_bot)
                if smaller and s.box[3] < baseline - 0.45 * typical and mid < base_mid:
                    sup.append(s)
                elif smaller and s.cy > baseline - 0.1 * typical and not sup:
                    sub.append(s)
                else:
                    break
                i += 1
                continue
            # el signo ∫ es muy alto: sus límites están en su tercio de arriba o de abajo; lo que queda a media altura
            # (el integrando, ∫ x² dx) sigue en la línea aunque su centro esté algo más bajo que el del signo
            if base.label == r"\int":
                sup_ok = s.box[3] <= base_top + 0.45 * base.h
                sub_ok = s.box[1] >= base_bot - 0.45 * base.h
            else:
                sup_ok = sub_ok = True
            if smaller and sup_ok and mid < base_mid - 0.25 * typical and s.box[3] < base_bot - 0.05 * typical:
                sup.append(s)
            elif smaller and sub_ok and mid > base_mid + 0.12 * typical and s.box[1] > base_top + 0.05 * typical:
                sub.append(s)
            else:
                break
            i += 1
        if primes:
            node = ("prime", node, primes)
        if sup and sub:
            node = ("subsup", node, _parse_row(sub, typical), _parse_row(sup, typical))
        elif sup:
            node = ("sup", node, _parse_row(sup, typical))
        elif sub:
            node = ("sub", node, _parse_row(sub, typical))
        items.append((base, node))
    # hueco grande entre dos cifras de la línea: no son un solo número
    out = []
    for k, (b, n) in enumerate(items):
        p = items[k - 1] if k else None
        if p and n[0] == p[1][0] == "num" and b.box[0] - p[0].box[2] > 0.8 * typical:
            out.append(("sym", r"\,"))
        out.append(n)
    return ("row", out)


def to_latex(node):
    kind = node[0]
    if kind == "sym":
        return node[1]
    if kind == "row":
        out = ""
        for child in node[1]:
            t = to_latex(child)
            # "\alpha x" necesita un espacio; "2x" no
            if re.search(r"\\[A-Za-z]+$", out) and t[:1].isalpha():
                out += " "
            out += t
        return out
    if kind == "num":
        return node[1].replace(".", ",")
    if kind == "sup":
        return f"{_group(node[1])}^{{{to_latex(node[2])}}}"
    if kind == "sub":
        return f"{_group(node[1])}_{{{to_latex(node[2])}}}"
    if kind == "subsup":
        return f"{_group(node[1])}_{{{to_latex(node[2])}}}^{{{to_latex(node[3])}}}"
    if kind == "frac":
        return f"\\frac{{{to_latex(node[1])}}}{{{to_latex(node[2])}}}"
    if kind == "sqrt":
        return f"\\sqrt{{{to_latex(node[1])}}}"
    if kind == "root":
        return f"\\sqrt[{to_latex(node[1])}]{{{to_latex(node[2])}}}"
    if kind == "accent":
        return f"{node[1]}{{{to_latex(node[2])}}}"
    if kind == "prime":
        return f"{_group(node[1])}{chr(39) * node[2]}"
    raise ValueError(kind)


def _group(node):
    t = to_latex(node)
    return t if node[0] in ("sym", "accent") or (node[0] == "num" and len(t) == 1) else "{" + t + "}"
