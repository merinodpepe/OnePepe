# SPDX-License-Identifier: GPL-3.0-or-later
"""Vocabulario de símbolos: alias de LaTeX y categorías (para el contexto y la estructura)."""

ALIASES = {
    r"\ne": r"\neq", r"\le": r"\leq", r"\ge": r"\geq", r"\to": r"\rightarrow", r"\gets": r"\leftarrow",
    r"\lt": "<", r"\gt": ">", r"\%": "%", r"\vert": "|", r"\mid": "|", r"\lbrace": r"\{", r"\rbrace": r"\}",
    r"\prime": "'", r"\ast": "*", r"\colon": ":",
}

GREEK = {
    r"\alpha", r"\beta", r"\gamma", r"\delta", r"\epsilon", r"\varepsilon", r"\zeta", r"\eta", r"\theta",
    r"\vartheta", r"\iota", r"\kappa", r"\lambda", r"\mu", r"\nu", r"\xi", r"\pi", r"\varpi", r"\rho", r"\sigma",
    r"\varsigma", r"\tau", r"\upsilon", r"\phi", r"\varphi", r"\chi", r"\psi", r"\omega", r"\Gamma", r"\Delta",
    r"\Theta", r"\Lambda", r"\Xi", r"\Pi", r"\Sigma", r"\Upsilon", r"\Phi", r"\Psi", r"\Omega",
}
BINARY_OPS = {"+", "-", r"\pm", r"\mp", r"\times", r"\cdot", r"\div", "*", "/"}
RELATIONS = {"=", r"\neq", "<", ">", r"\leq", r"\geq", r"\approx", r"\sim", r"\simeq", r"\equiv", r"\propto",
             r"\rightarrow", r"\leftarrow", r"\Rightarrow", r"\in", r"\subset", r"\supset"}
OPEN = {"(", "[", r"\{", r"\langle", r"\lfloor", r"\lceil"}
CLOSE = {")", "]", r"\}", r"\rangle", r"\rfloor", r"\rceil"}
BIG_OPS = {r"\sum", r"\prod", r"\int", r"\oint", r"\iint", r"\bigcup", r"\bigcap"}
DIGITS = set("0123456789")
LATIN = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
# un trazo horizontal: puede ser una resta, una raya de fracción o parte de un "="
HORIZONTAL_BAR = {"-", r"\frac"}


def normalize_label(label):
    label = label.strip()
    return ALIASES.get(label, label)


def category(latex):
    if latex in DIGITS:
        return "digit"
    if latex in LATIN:
        return "latin"
    if latex in GREEK:
        return "greek"
    if latex in BINARY_OPS or latex in HORIZONTAL_BAR:
        return "binop"
    if latex in RELATIONS:
        return "relation"
    if latex in OPEN:
        return "open"
    if latex in CLOSE:
        return "close"
    if latex in BIG_OPS:
        return "bigop"
    if latex in {r"\sqrt"}:
        return "sqrt"
    if latex in {".", ",", "'", "!"}:
        return "punct"
    return "other"


def is_operand(latex):
    """Cifras, letras y griegas: lo que puede ir a los lados de un operador."""
    return category(latex) in {"digit", "latin", "greek"}

# Vocabulario básico de matemáticas y física (el que se usa por defecto). Los demás símbolos de MathWriting
# solo se consideran si se activan en el glosario del cuaderno.
CORE = (
    DIGITS
    | LATIN
    | {r"\alpha", r"\beta", r"\gamma", r"\delta", r"\epsilon", r"\varepsilon", r"\eta", r"\theta", r"\kappa",
       r"\lambda", r"\mu", r"\nu", r"\pi", r"\rho", r"\sigma", r"\tau", r"\phi", r"\varphi", r"\chi", r"\psi",
       r"\omega", r"\Gamma", r"\Delta", r"\Theta", r"\Lambda", r"\Pi", r"\Sigma", r"\Phi", r"\Psi", r"\Omega"}
    | {"+", "-", r"\pm", r"\mp", r"\times", r"\cdot", r"\div", "/", "=", r"\neq", "<", ">", r"\leq", r"\geq",
       r"\approx", r"\sim", r"\equiv", r"\propto", "(", ")", "[", "]", r"\{", r"\}", "|", ",", ".", "'", "!",
       r"\frac", r"\sqrt", r"\int", r"\sum", r"\prod", r"\partial", r"\nabla", r"\infty", r"\rightarrow",
       r"\leftarrow", r"\in", "%", r"\circ", r"\lim", "*"}
)

# Símbolos que, colocados en la caja del símbolo escrito, se dibujan igual: confundirlos no cambia el resultado
# visual de la sustitución (sí puede cambiar el significado matemático).
_SAME_SHAPE = [
    {"c", "C"}, {"o", "O", "0"}, {"s", "S"}, {"u", "U"}, {"v", "V"}, {"w", "W"}, {"x", "X"}, {"z", "Z"},
    {"-", r"\frac"}, {".", r"\cdot"}, {"l", "|"}, {"p", "P"}, {"y", "Y"},
]


def same_shape(a, b):
    return a == b or any(a in g and b in g for g in _SAME_SHAPE)
