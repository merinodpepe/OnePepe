# SPDX-License-Identifier: GPL-3.0-or-later
"""Diccionario de símbolos: nombre en español e inglés, LaTeX, carácter, grupo y atajo.

Es el «fichero de datos» de la fase 2 del plan tipo MyScript y lo que usa el campo «Otro…» de la ventanita de
candidatas (PLAN-PENDIENTE.md §2, punto 5): buscar «delta», «alfa», «menor o igual», teclear «δ», «\\delta» o un
atajo como «\\d» o «<=». El programa auxiliar lo envía a la app con la orden `names`, así no se duplica.
"""

import unicodedata

from .symbols import normalize_label

# (latex, carácter, grupo, nombres separados por «|», atajo o "")
_TABLE = [
    # griego
    (r"\alpha", "α", "griego", "alfa|alpha", r"\a"),
    (r"\beta", "β", "griego", "beta", r"\b"),
    (r"\gamma", "γ", "griego", "gamma", r"\g"),
    (r"\delta", "δ", "griego", "delta", r"\d"),
    (r"\epsilon", "ϵ", "griego", "epsilon|épsilon", r"\e"),
    (r"\varepsilon", "ε", "griego", "epsilon curvo|varepsilon", ""),
    (r"\zeta", "ζ", "griego", "zeta|dseta", r"\z"),
    (r"\eta", "η", "griego", "eta", r"\h"),
    (r"\theta", "θ", "griego", "theta|zeta griega|teta", r"\t"),
    (r"\iota", "ι", "griego", "iota", ""),
    (r"\kappa", "κ", "griego", "kappa|kapa", r"\k"),
    (r"\lambda", "λ", "griego", "lambda", r"\l"),
    (r"\mu", "μ", "griego", "mu|micro", r"\m"),
    (r"\nu", "ν", "griego", "nu", r"\n"),
    (r"\xi", "ξ", "griego", "xi", r"\x"),
    (r"\pi", "π", "griego", "pi", r"\p"),
    (r"\rho", "ρ", "griego", "rho|ro", r"\r"),
    (r"\sigma", "σ", "griego", "sigma", r"\s"),
    (r"\tau", "τ", "griego", "tau", ""),
    (r"\upsilon", "υ", "griego", "upsilon|ípsilon", r"\u"),
    (r"\phi", "ϕ", "griego", "phi|fi", r"\f"),
    (r"\varphi", "φ", "griego", "phi curvo|varphi|fi curva", ""),
    (r"\chi", "χ", "griego", "chi|ji", r"\c"),
    (r"\psi", "ψ", "griego", "psi", r"\y"),
    (r"\omega", "ω", "griego", "omega", r"\o"),
    (r"\Gamma", "Γ", "griego", "gamma mayúscula", r"\G"),
    (r"\Delta", "Δ", "griego", "delta mayúscula|incremento", r"\D"),
    (r"\Theta", "Θ", "griego", "theta mayúscula", r"\T"),
    (r"\Lambda", "Λ", "griego", "lambda mayúscula", r"\L"),
    (r"\Xi", "Ξ", "griego", "xi mayúscula", ""),
    (r"\Pi", "Π", "griego", "pi mayúscula", r"\P"),
    (r"\Sigma", "Σ", "griego", "sigma mayúscula", r"\S"),
    (r"\Phi", "Φ", "griego", "phi mayúscula|fi mayúscula", r"\F"),
    (r"\Psi", "Ψ", "griego", "psi mayúscula", ""),
    (r"\Omega", "Ω", "griego", "omega mayúscula|ohmio", r"\O"),
    # operadores
    ("+", "+", "operadores", "más|suma|plus", ""),
    ("-", "−", "operadores", "menos|resta|minus", ""),
    (r"\pm", "±", "operadores", "más menos|plus minus", "+-"),
    (r"\mp", "∓", "operadores", "menos más", "-+"),
    (r"\times", "×", "operadores", "por|multiplicar|aspa|times", "*"),
    (r"\cdot", "·", "operadores", "punto|por punto|cdot", ""),
    (r"\div", "÷", "operadores", "dividir|entre|división", ""),
    ("/", "/", "operadores", "barra|dividido", ""),
    (r"\frac", "—", "operadores", "fracción|raya de fracción", ""),
    (r"\sqrt", "√", "operadores", "raíz|raiz|raíz cuadrada|sqrt", ""),
    (r"\circ", "°", "operadores", "grado|círculo|composición", ""),
    ("%", "%", "operadores", "por ciento|porcentaje", ""),
    ("!", "!", "operadores", "factorial|exclamación", ""),
    # relaciones
    ("=", "=", "relaciones", "igual", ""),
    (r"\neq", "≠", "relaciones", "distinto|no igual|diferente", "!="),
    ("<", "<", "relaciones", "menor", ""),
    (">", ">", "relaciones", "mayor", ""),
    (r"\leq", "≤", "relaciones", "menor o igual", "<="),
    (r"\geq", "≥", "relaciones", "mayor o igual", ">="),
    (r"\approx", "≈", "relaciones", "aproximadamente|aproximado|casi igual", "~="),
    (r"\sim", "∼", "relaciones", "del orden|virgulilla|tilde", "~"),
    (r"\equiv", "≡", "relaciones", "equivalente|idéntico|congruente", "=="),
    (r"\propto", "∝", "relaciones", "proporcional", ""),
    (r"\in", "∈", "relaciones", "pertenece|en", ""),
    (r"\subset", "⊂", "relaciones", "subconjunto|contenido", ""),
    # flechas
    (r"\rightarrow", "→", "flechas", "flecha|flecha derecha|tiende a|implica paso", "->"),
    (r"\leftarrow", "←", "flechas", "flecha izquierda", "<-"),
    (r"\Rightarrow", "⇒", "flechas", "implica|entonces|flecha doble", "=>"),
    (r"\Leftrightarrow", "⇔", "flechas", "si y solo si|equivale|doble implicación", "<=>"),
    # cálculo
    (r"\int", "∫", "cálculo", "integral", ""),
    (r"\sum", "∑", "cálculo", "sumatorio|sumatoria|suma grande", ""),
    (r"\prod", "∏", "cálculo", "productorio|producto", ""),
    (r"\partial", "∂", "cálculo", "derivada parcial|parcial|de redonda", ""),
    (r"\nabla", "∇", "cálculo", "nabla|gradiente", ""),
    (r"\infty", "∞", "cálculo", "infinito", "inf"),
    (r"\lim", "lim", "cálculo", "límite|limite", ""),
    # signos
    ("(", "(", "signos", "paréntesis abre", ""),
    (")", ")", "signos", "paréntesis cierra", ""),
    ("[", "[", "signos", "corchete abre", ""),
    ("]", "]", "signos", "corchete cierra", ""),
    (r"\{", "{", "signos", "llave abre", ""),
    (r"\}", "}", "signos", "llave cierra", ""),
    ("|", "|", "signos", "barra vertical|valor absoluto", ""),
    (",", ",", "signos", "coma", ""),
    (".", ".", "signos", "punto decimal", ""),
    (";", ";", "signos", "punto y coma", ""),
    (":", ":", "signos", "dos puntos", ""),
    ("'", "′", "signos", "prima|derivada", ""),
]

ENTRIES = [
    {"latex": latex, "char": char, "group": group, "names": names.split("|"), "shortcut": shortcut}
    for latex, char, group, names, shortcut in _TABLE
]
GROUPS = ["griego", "operadores", "relaciones", "flechas", "cálculo", "signos"]

_BY_LATEX = {e["latex"]: e for e in ENTRIES}
_BY_CHAR = {e["char"]: e for e in ENTRIES}
_BY_SHORTCUT = {e["shortcut"]: e for e in ENTRIES if e["shortcut"]}


def _plain(text):
    """Minúsculas y sin tildes, para buscar («Raiz» encuentra «raíz»)."""
    text = unicodedata.normalize("NFD", text.strip().lower())
    return "".join(c for c in text if unicodedata.category(c) != "Mn")


def resolve(text):
    """El LaTeX de lo que el usuario teclea, si es exacto: el carácter, el LaTeX, un atajo o un nombre.

    Una letra o cifra suelta es ella misma. Devuelve None si no se sabe qué es.
    """
    t = text.strip()
    if not t:
        return None
    if t in _BY_SHORTCUT:
        return _BY_SHORTCUT[t]["latex"]
    if t in _BY_CHAR:
        return _BY_CHAR[t]["latex"]
    if t.startswith("\\") and (normalize_label(t) in _BY_LATEX or t[1:].isalpha()):
        return normalize_label(t)
    if len(t) == 1 and t.isalnum():
        return t
    plain = _plain(t)
    for e in ENTRIES:
        if any(_plain(n) == plain for n in e["names"]) or _plain(e["latex"].lstrip("\\")) == plain:
            return e["latex"]
    return None


def search(text, limit=12):
    """Símbolos que encajan con lo tecleado, del más parecido al menos: exactos, luego por el principio del
    nombre y luego por cualquier parte."""
    t = text.strip()
    if not t:
        return []
    exact = resolve(t)
    plain = _plain(t)
    scored = []
    for e in ENTRIES:
        names = [_plain(n) for n in e["names"]] + [_plain(e["latex"].lstrip("\\"))]
        if e["latex"] == exact:
            score = 0
        elif any(n.startswith(plain) for n in names):
            score = 1
        elif any(plain in n for n in names):
            score = 2
        else:
            continue
        scored.append((score, ENTRIES.index(e), e))
    out = [e for _, _, e in sorted(scored, key=lambda x: (x[0], x[1]))]
    if exact and exact not in _BY_LATEX:
        out.insert(0, {"latex": exact, "char": exact, "group": "", "names": [exact], "shortcut": ""})
    return out[:limit]


def entries():
    """Todo el diccionario, con si hay símbolo limpio para dibujarlo."""
    from .glyphs import font_and_char

    out = []
    for e in ENTRIES:
        e = dict(e)
        e["glyph"] = font_and_char(e["latex"]) is not None or e["latex"] in _manual_glyphs()
        out.append(e)
    return out


def _manual_glyphs():
    from . import glyphs

    return getattr(glyphs, "_MANUAL", {})
