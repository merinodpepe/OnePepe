# SPDX-License-Identifier: GPL-3.0-or-later
"""Estructura espacial de una expresión: línea base, potencias, subíndices, fracciones y raíces.

Entrada: símbolos ya reconocidos, cada uno con su etiqueta LaTeX y su caja (x0, y0, x1, y1), eje y hacia abajo.
Salida: un árbol sencillo y su LaTeX. Las tolerancias son amplias porque la letra a mano se mueve.

Nodos del árbol (tuplas):
    ("sym", latex)                      símbolo
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

from .symbols import ACCENTS, category


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


def parse(syms):
    """Árbol de la expresión."""
    syms = list(syms)
    if not syms:
        return ("row", [])
    typical = _typical_height(syms)
    return _parse_region(syms, typical)


def _parse_region(syms, typical):
    syms = sorted(syms, key=lambda s: s.box[0])
    # 1. fracciones: la raya más larga primero
    bars = sorted(
        [s for s in syms if s.label in ("-", r"\frac") and s.w > 0.8 * typical],
        key=lambda s: -s.w,
    )
    for bar in bars:
        if bar not in syms:
            continue
        x0, x1 = bar.box[0] - 0.1 * bar.w, bar.box[2] + 0.1 * bar.w
        above = [s for s in syms if s is not bar and x0 <= s.cx <= x1 and s.box[3] <= bar.cy + 0.15 * typical]
        below = [s for s in syms if s is not bar and x0 <= s.cx <= x1 and s.box[1] >= bar.cy - 0.15 * typical]
        # solo lo que está cerca de la raya en vertical
        above = [s for s in above if bar.cy - s.box[3] < 1.5 * typical + 0.0 * s.h]
        below = [s for s in below if s.box[1] - bar.cy < 1.5 * typical]
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
            node.node = ("frac", num, den)
            rest = [s for s in syms if s is not bar and s not in above and s not in below]
            return _parse_region(rest + [node], typical)
    # 2. acentos: un sombrero, una raya o una virgulilla pequeños justo encima de un símbolo (ŷ, x̄, x̃). La raya de
    # una fracción tiene algo debajo y encima; la de una media, solo su letra debajo y más estrecha.
    for acc in [s for s in syms if s.label in ACCENTS or s.label == "-"]:
        if acc not in syms:
            continue
        below = [
            s
            for s in syms
            if s is not acc
            and s.label not in ACCENTS
            and s.box[0] - 0.35 * s.w <= acc.cx <= s.box[2] + 0.35 * s.w
            and -0.15 * typical <= s.box[1] - acc.box[3] < 0.6 * typical
            and acc.w <= 2.2 * max(s.w, 0.4 * typical)
        ]
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
        node.node = ("accent", kind, _node(base))
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
            node.node = ("root", _parse_region(index, typical), radicand) if index else ("sqrt", radicand)
            rest = [s for s in syms if s is not root and s not in inside and s not in index]
            return _parse_region(rest + [node], typical)
    # 4. línea base con potencias y subíndices
    return _parse_row(sorted(syms, key=lambda s: s.box[0]), typical)


def _node(s):
    return getattr(s, "node", None) or ("sym", s.label)


FUNCTIONS = ["sin", "cos", "tan", "cot", "sec", "csc", "sinh", "cosh", "tanh", "arcsin", "arccos", "arctan",
             "ln", "log", "exp", "lim", "max", "min", "det"]
# nombres en español: se escriben así y se guardan con el nombre de LaTeX (sen x → \sin x)
FUNCTION_ALIASES = {"sen": "sin", "tg": "tan", "arcsen": "arcsin", "arctg": "arctan", "cotg": "cot",
                    "senh": "sinh", "tgh": "tanh"}
# no pueden ser exponente ni subíndice por sí solos
_NEVER_SCRIPT = {",", ".", "=", "<", ">", r"\leq", r"\geq", r"\neq", r"\approx", r"\rightarrow", r"\Rightarrow",
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


def _parse_row(syms, typical):
    syms = _merge_functions(syms, typical)
    items = []
    i = 0
    while i < len(syms):
        base = syms[i]
        node = _node(base)
        i += 1
        if base.label in _FLAT or category(base.label) in {"binop", "relation", "open"}:
            items.append((base, node))
            continue
        base_mid = _middle(base, typical)
        base_top, base_bot = base.box[1], base.box[3]
        sup, sub = [], []
        # los símbolos siguientes más pequeños y desplazados hacia arriba (o abajo) forman el exponente (o subíndice)
        while i < len(syms):
            s = syms[i]
            if s.label in _NEVER_SCRIPT or (s.box[0] > base.box[2] + 1.2 * typical and not (sup or sub)):
                break
            # una función de letras bajas tras un paréntesis («sin(x)cos(x)») sigue en la línea; solo es exponente
            # si queda entera por encima del centro (e^{sin x})
            if is_function(s.label) and not s.box[3] < base_mid:
                break
            smaller = s.h < 0.85 * max(base.h, typical) or s.label in _FLAT
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
            if smaller and mid < base_mid - 0.25 * typical and s.box[3] < base_bot - 0.05 * typical:
                sup.append(s)
            elif smaller and mid > base_mid + 0.12 * typical and s.box[1] > base_top + 0.05 * typical:
                sub.append(s)
            else:
                break
            i += 1
        if sup and sub:
            node = ("subsup", node, _parse_row(sub, typical), _parse_row(sup, typical))
        elif sup:
            node = ("sup", node, _parse_row(sup, typical))
        elif sub:
            node = ("sub", node, _parse_row(sub, typical))
        items.append((base, node))
    return ("row", [n for _, n in items])


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
    raise ValueError(kind)


def _group(node):
    t = to_latex(node)
    return t if node[0] in ("sym", "accent") else "{" + t + "}"
