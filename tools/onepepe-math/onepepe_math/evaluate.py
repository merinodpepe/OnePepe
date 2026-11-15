# SPDX-License-Identifier: GPL-3.0-or-later
"""Del árbol de la expresión a SymPy, y el resultado del "=".

No se usa `sympy.parsing.latex` (solo entiende 19 de las 30 expresiones de prueba): el árbol de `layout.py` se recorre
con un analizador propio. Comportamiento (ver MATEMATICAS-INFORME.md, sección 0.3):

- `expr =` sin incógnitas: el valor (exacto si es una fracción sencilla; si no, 4 cifras significativas).
- `ecuación, x =`: despeja x. `ecuación =` con una sola incógnita: la despeja.
- `expr =` con variables: la forma simplificada, solo si es distinta y no más larga.
- derivadas `d/dx(...)`, integrales, sumatorios y límites: se calculan.
- constantes y unidades del glosario: se calculan con pint; si las dimensiones no casan, no hay resultado.
- cualquier cosa ambigua o inválida: None (no se muestra nada).
"""

import sympy as sp

from .latex_tokens import visible_tokens
from .lexico import _leibniz
from .symbols import GREEK, STEP_SEPARATORS

GREEK_NAMES = {g: g[1:] for g in GREEK}
FUNCS = {
    r"\sin": sp.sin, r"\cos": sp.cos, r"\tan": sp.tan, r"\cot": sp.cot, r"\sec": sp.sec, r"\csc": sp.csc,
    r"\sinh": sp.sinh, r"\cosh": sp.cosh, r"\tanh": sp.tanh, r"\arcsin": sp.asin, r"\arccos": sp.acos,
    r"\arctan": sp.atan, r"\ln": sp.log, r"\log": lambda x: sp.log(x, 10), r"\exp": sp.exp,
}
MUL_OPS = {r"\times", r"\cdot", "*"}
DIV_OPS = {"/", r"\div"}
RELATIONS = {"=", r"\neq", "<", ">", r"\leq", r"\geq", r"\approx"}


class ParseError(Exception):
    pass


# ---------- árbol -> tokens ----------


def _tokens(row, decimal_comma=True):
    """Fichas de una fila del árbol: (tipo, valor, exponente, subíndice).

    `decimal_comma` (lo normal, como escribe el usuario): la coma es el separador decimal y **el punto multiplica**
    (2·x, 3.4 = 3 por 4). Con False, el punto es decimal (solo para leer intervalos «[0.5; 2]»).
    """
    assert row[0] == "row"
    out = []
    for node in row[1]:
        sup = sub = None
        base = node
        if node[0] == "sup":
            base, sup = node[1], node[2]
        elif node[0] == "sub":
            base, sub = node[1], node[2]
        elif node[0] == "subsup":
            base, sub, sup = node[1], node[2], node[3]
        if base[0] == "sym":
            out.append(["sym", base[1], sup, sub])
        elif base[0] == "num":
            out.append(["num", base[1], sup, sub])
        else:
            out.append([base[0], base, sup, sub])
    # unir cifras (y el separador decimal) en números
    merged = []
    for t in out:
        prev = merged[-1] if merged else None
        if t[0] == "num":  # número ya unido por el diseño; solo sigue a «3.» (punto decimal abierto)
            if prev is not None and prev[0] == "num" and prev[1].endswith(".") and prev[2] is None:
                prev[1] += t[1]
                prev[2], prev[3] = t[2], t[3]
            else:
                merged.append(t)
            continue
        is_digit = t[0] == "sym" and t[1].isdigit()
        if is_digit and prev is not None and prev[0] == "num" and prev[2] is None and prev[3] is None:
            prev[1] += t[1]
            prev[2], prev[3] = t[2], t[3]
            continue
        if (
            t[0] == "sym"
            and t[1] in ({","} if decimal_comma else {"."})
            and prev is not None
            and prev[0] == "num"
            and "." not in prev[1]
            and prev[2] is None
        ):
            prev[1] += "."
            prev.append(t[1])  # separador original, por si no siguen decimales
            continue
        if is_digit:
            merged.append(["num", t[1], t[2], t[3]])
        else:
            merged.append(t)
    # un número que termina en "." sin decimales: el punto era otra cosa
    final = []
    for t in merged:
        if t[0] == "num" and t[1].endswith("."):
            final.append(["num", t[1][:-1], None, None])
            final.append(["sym", t[4] if len(t) > 4 else ".", t[2], t[3]])
        else:
            final.append(t)
    # con coma decimal, un punto suelto multiplica si está entre dos cosas (2.x, a.b, 3.4); si no, sobra (un punto
    # final) y se quita
    if decimal_comma:
        dotted = []
        for i, t in enumerate(final):
            if t[0] == "sym" and t[1] == "." and t[2] is None and t[3] is None:
                prev = dotted[-1] if dotted else None
                nxt = final[i + 1] if i + 1 < len(final) else None
                if _operand_end(prev) and _operand_start(nxt):
                    dotted.append(["sym", r"\cdot", None, None])
                continue
            dotted.append(t)
        final = dotted
    # una «x» entre dos números es el signo de multiplicar (3 x 4), no una variable
    for i in range(1, len(final) - 1):
        t = final[i]
        if (
            t[0] == "sym"
            and t[1] in ("x", "X")
            and t[2] is None
            and t[3] is None
            and final[i - 1][0] == "num"
            and final[i - 1][2] is None
            and final[i + 1][0] == "num"
        ):
            final[i] = ["sym", r"\times", None, None]
    return _leibniz(final)


_NOT_OPERAND = {"+", "-", r"\pm", r"\mp", r"\times", r"\cdot", r"\div", "/", "*", "=", r"\neq", "<", ">",
                r"\leq", r"\geq", r"\approx", ",", ";", ":", r"\rightarrow", r"\Rightarrow", r"\Leftrightarrow"}


def _operand_end(t):
    """¿Puede la ficha ser lo de la izquierda de una multiplicación? (un número, una letra, «)»…)"""
    return t is not None and not (t[0] == "sym" and (t[1] in _NOT_OPERAND or t[1] in ("(", "[", r"\{")))


def _operand_start(t):
    """¿Puede la ficha ser lo de la derecha de una multiplicación? (un número, una letra, «(»…)"""
    return t is not None and not (t[0] == "sym" and (t[1] in _NOT_OPERAND or t[1] in (")", "]", r"\}")))


# ---------- analizador ----------


class _Parser:
    def __init__(self, tokens, glossary, funciones=(), conocidas=()):
        self.t = tokens
        self.i = 0
        self.glossary = glossary or {}
        self.funciones = funciones  # nombres de funciones definidas en la página: f(x) es una llamada
        self.conocidas = conocidas  # nombres con valor o alias en la página: se puede derivar df/dx

    def hijo(self, tokens):
        return _Parser(tokens, self.glossary, self.funciones, self.conocidas)

    def sub(self, row):
        return self.hijo(_tokens(row)).full()

    def _llamada(self, nombre, tok):
        """f(a) con f función de la página: la llamada sin sustituir (la sustituye `Vista.resolver`)."""
        self.take()
        arg = self.expr()
        end = self.take()
        if end is None or end[1] != ")":
            raise ParseError("paréntesis sin cerrar")
        return self._apply_scripts(sp.Function(nombre)(arg), end)

    def peek(self, k=0):
        j = self.i + k
        return self.t[j] if j < len(self.t) else None

    def take(self):
        tok = self.peek()
        self.i += 1
        return tok

    def is_sym(self, tok, *values):
        return tok is not None and tok[0] == "sym" and tok[1] in values

    def expr(self):
        node = self.term()
        while self.is_sym(self.peek(), "+", "-", r"\pm"):
            op = self.take()[1]
            if op == r"\pm":
                raise ParseError("± no se calcula")
            rhs = self.term()
            node = node + rhs if op == "+" else node - rhs
        return node

    def term(self):
        node = self.unary()
        while True:
            tok = self.peek()
            if tok is None:
                return node
            if self.is_sym(tok, *MUL_OPS):
                self.take()
                node = node * self.unary()
            elif self.is_sym(tok, *DIV_OPS):
                self.take()
                node = node / self.unary()
            elif self._starts_atom(tok):
                node = node * self.unary()
            else:
                return node

    def unary(self):
        if self.is_sym(self.peek(), "-"):
            self.take()
            return -self.unary()
        if self.is_sym(self.peek(), "+"):
            self.take()
            return self.unary()
        return self.atom()

    def _starts_atom(self, tok):
        if tok[0] in ("num", "frac", "sqrt", "root", "accent", "deriv", "prime"):
            return True
        if tok[0] != "sym":
            return False
        v = tok[1]
        return (
            len(v) == 1
            and v.isalpha()
            or v in GREEK_NAMES
            or v in FUNCS
            or v in ("(", "[", r"\{", r"\int", r"\sum", r"\prod", r"\lim", r"\infty", r"\partial", r"\sqrt")
        )

    def _apply_scripts(self, value, tok):
        sup = tok[2]
        if sup is not None:
            value = value ** self.sub(sup)
        return value

    def atom(self):
        tok = self.take()
        if tok is None:
            raise ParseError("falta un término")
        kind = tok[0]
        if kind == "num":
            value = sp.Rational(tok[1]) if "." in tok[1] else sp.Integer(tok[1])
            if tok[3] is not None:
                raise ParseError("subíndice en un número")
            return self._apply_scripts(value, tok)
        if kind == "deriv":
            f, vs = tok[1]
            if f is None:
                operando = self.unary()
            elif f in self.conocidas:
                operando = self._symbol(["sym", f, None, None])
            else:
                raise ParseError("derivada de una función sin definir")
            return sp.Derivative(operando, *[(self._symbol(["sym", v, None, None]), n) for v, n in vs])
        if kind == "prime":
            _, base, n = tok[1]
            if base[0] == "sym" and base[1] in self.funciones and self.is_sym(self.peek(), "("):
                return self._llamada(f"{base[1]}__d{n}", tok)
            raise ParseError("derivada de una función sin definir")
        if kind == "frac":
            _, num, den = tok[1]
            value = self.sub(num) / self.sub(den)
            return self._apply_scripts(value, tok)
        if kind == "sqrt":
            return self._apply_scripts(sp.sqrt(self.sub(tok[1][1])), tok)
        if kind == "accent":
            return self._apply_scripts(self._symbol(accent_token(tok)), tok)
        if kind == "root":
            index = self.sub(tok[1][1])
            radicand = self.sub(tok[1][2])
            # ∛-8 = -2 (la raíz real), como en clase
            if index.is_integer and index.is_odd:
                value = sp.real_root(radicand, index)
            else:
                value = sp.root(radicand, index)
            return self._apply_scripts(value, tok)
        if kind != "sym":
            raise ParseError(f"no sé leer {kind}")
        v = tok[1]
        if v in ("(", "[", r"\{"):
            close = {"(": ")", "[": "]", r"\{": r"\}"}[v]
            inner = self.expr()
            end = self.take()
            if end is None or end[1] != close:
                raise ParseError("paréntesis sin cerrar")
            return self._apply_scripts(inner, end)
        if v in FUNCS:
            arg = self.atom() if self.is_sym(self.peek(), "(") else self._bare_argument()
            if v == r"\log" and tok[3] is not None:
                # log₂(8): la base va de subíndice
                value = sp.log(arg, self.sub(tok[3]))
            else:
                value = FUNCS[v](arg)
            return self._apply_scripts(value, tok)
        if v == r"\int":
            return self._integral(tok)
        if v in (r"\sum", r"\prod"):
            return self._sum(tok, v)
        if v == r"\lim":
            return self._limit(tok)
        if v == r"\infty":
            return sp.oo
        if v in self.funciones and tok[2] is None and tok[3] is None and self.is_sym(self.peek(), "("):
            return self._llamada(v, tok)
        if v in ("f", "g", "h", "F", "G") and v not in self.glossary and tok[3] is None and self.is_sym(self.peek(), "("):
            raise ParseError("función sin definir")
        return self._apply_scripts(self._symbol(tok), tok)

    def _bare_argument(self):
        """Argumento sin paréntesis: sin 2x = sin(2x), pero sin x cos x = sin(x)·cos(x)."""
        node = self.unary()
        while True:
            tok = self.peek()
            if tok is None or not self._starts_atom(tok):
                return node
            if tok[0] == "sym" and (tok[1] in FUNCS or tok[1] in (r"\int", r"\sum", r"\prod", r"\lim")):
                return node
            node = node * self.unary()

    def _symbol(self, tok):
        v = tok[1]
        name = GREEK_NAMES.get(v, v)
        if not (name.isalpha()):
            raise ParseError(f"símbolo inesperado {v}")
        if tok[3] is not None:
            name = f"{name}_{_plain(tok[3])}"
        if name in self.glossary:
            return self.glossary[name]
        if name == "pi":
            return sp.pi
        if name == "e" and "e" not in self.conocidas:
            return sp.E
        return sp.Symbol(name)

    def _integral(self, tok):
        lower = self.sub(tok[3]) if tok[3] is not None else None
        upper = self.sub(tok[2]) if tok[2] is not None else None
        # el integrando llega hasta "d" seguido de una variable
        start = self.i
        j = start
        while j < len(self.t) - 1 and not (self.is_sym(self.t[j], "d") and self.t[j + 1][0] == "sym"):
            j += 1
        if j >= len(self.t) - 1:
            raise ParseError("integral sin diferencial")
        integrand = self.hijo(self.t[start:j]).full()
        var = self._symbol(self.t[j + 1])
        self.i = j + 2
        if lower is None and upper is None:
            return sp.Integral(integrand, var)
        if lower is None or upper is None:
            raise ParseError("falta un límite de integración")
        return sp.Integral(integrand, (var, lower, upper))

    def _sum(self, tok, kind):
        if tok[3] is None or tok[2] is None:
            raise ParseError("sumatorio sin límites")
        sub_t = _tokens(tok[3])
        if len(sub_t) < 3 or not self.is_sym(sub_t[1], "="):
            raise ParseError("sumatorio sin índice")
        var = self._symbol(sub_t[0])
        lower = self.hijo(sub_t[2:]).full()
        upper = self.sub(tok[2])
        body = self.term()
        cls = sp.Sum if kind == r"\sum" else sp.Product
        return cls(body, (var, lower, upper))

    def _limit(self, tok):
        if tok[3] is None:
            raise ParseError("límite sin variable")
        sub_t = _tokens(tok[3])
        if len(sub_t) < 3 or not self.is_sym(sub_t[1], r"\rightarrow"):
            raise ParseError("límite mal escrito")
        var = self._symbol(sub_t[0])
        to = self.hijo(sub_t[2:]).full()
        return sp.Limit(self.term(), var, to)

    def full(self):
        value = self.expr()
        if self.peek() is not None:
            raise ParseError("hueco entre números" if self.peek()[1] == r"\," else f"sobra {self.peek()[1]}")
        return value


ACCENT_NAMES = {r"\hat": "hat", r"\overline": "bar", r"\tilde": "tilde"}


def accent_token(tok):
    """La ficha de un símbolo con acento como un símbolo con nombre propio: ŷ → «y_hat», x̄ → «x_bar» (otra variable
    distinta de y o de x). Conserva su subíndice."""
    _, kind, base = tok[1]
    if base[0] != "sym":
        raise ParseError("acento sobre algo compuesto")
    name = GREEK_NAMES.get(base[1], base[1])
    if not name.isalpha():
        raise ParseError(f"acento sobre {base[1]}")
    return ["sym", f"{name}{ACCENT_NAMES[kind]}", tok[2], tok[3]]


def _plain(row):
    """Texto de un subíndice (k_{B} -> "B", x_{12} -> "12")."""
    out = ""
    for node in row[1]:
        if node[0] not in ("sym", "num"):
            raise ParseError("subíndice complejo")
        out += GREEK_NAMES.get(node[1], node[1])
    return out


def sub_expr(row, glossary=None):
    return _Parser(_tokens(row), glossary).full()


def _split(tokens, values):
    parts, cur = [], []
    for t in tokens:
        if t[0] == "sym" and t[1] in values and t[2] is None and t[3] is None:
            parts.append((cur, t[1]))
            cur = []
        else:
            cur.append(t)
    parts.append((cur, None))
    return parts


# ---------- resultado del "=" ----------


def _format_number(value):
    value = sp.nsimplify(value) if isinstance(value, sp.Float) else value
    if isinstance(value, sp.Rational) and (value.q <= 1000 and abs(value.p) < 10**9):
        return value
    return sp.Float(sp.N(value, 4), 4)


def _natural_log_names(result):
    """SymPy escribe log para el logaritmo neperiano; en clase es ln (log sin base es log₁₀)."""
    if result is None:
        return None
    result = dict(result)
    result["text"] = result["text"].replace("log(", "ln(")
    result["latex"] = result["latex"].replace(r"\log", r"\ln")
    return result


def _parse(tokens, glossary, vista):
    """(expresión con lo que vale en el renglón, ¿cambió algo?). `vista`: ver `entorno.Vista`."""
    if vista is None:
        return _Parser(tokens, glossary).full(), False
    return vista.resolver(_Parser(tokens, glossary, vista.funciones, vista.conocidas).full())


def answer(tree, glossary=None, units=None, vista=None):
    """Resultado del "=" de una fila. Devuelve dict {"latex", "text", "kind"} o None.

    `glossary`: {nombre: valor sympy} (p. ej. constantes); `units`: {nombre: cantidad de pint} para calcular con
    unidades y comprobar dimensiones.
    """
    try:
        return _natural_log_names(_answer(tree, glossary or {}, units or {}, vista))
    except (ParseError, ValueError, TypeError, ZeroDivisionError, NotImplementedError, AttributeError):
        return None
    except Exception as e:  # SymPy puede lanzar errores variados con entradas raras
        if type(e).__module__.startswith(("sympy", "pint")):
            return None
        raise


def _answer(tree, glossary, units, vista):
    tokens = _tokens(tree)
    if not tokens or not (tokens[-1][0] == "sym" and tokens[-1][1] == "=" and tokens[-1][2] is None):
        return None
    tokens = tokens[:-1]
    # las comas, los ";" y las flechas separan expresiones o pasos: solo cuenta lo último ("2x+4=10 → x =" despeja x
    # de la ecuación de antes, como "2x+4=10, x =")
    statements = [s for s, _ in _split(tokens, {","} | STEP_SEPARATORS)]
    last = statements[-1]
    # "ecuación, x ="
    if len(statements) >= 2 and len(last) == 1 and last[0][0] == "sym":
        var = _Parser([], glossary)._symbol(last[0])
        eq_parts = _split(statements[-2], RELATIONS)
        if len(eq_parts) != 2 or eq_parts[0][1] != "=":
            return None
        lhs = _parse(eq_parts[0][0], glossary, vista)[0]
        rhs = _parse(eq_parts[1][0], glossary, vista)[0]
        return _solve(sp.Eq(lhs, rhs), var)
    parts = _split(last, RELATIONS)
    if len(parts) == 2 and parts[0][1] == "=":
        # "ecuación =": se despeja la única incógnita
        lhs = _parse(parts[0][0], glossary, vista)[0]
        rhs = _parse(parts[1][0], glossary, vista)[0]
        free = (lhs - rhs).free_symbols
        if len(free) != 1:
            return None
        return _solve(sp.Eq(lhs, rhs), free.pop())
    if len(parts) != 1:
        return None
    expr, cambiado = _parse(last, glossary, vista)
    if units and any(s.name in units for s in expr.free_symbols):
        return _with_units(expr, units)
    has_calculus = expr.has(sp.Derivative, sp.Integral, sp.Sum, sp.Product, sp.Limit)
    value = expr.doit() if has_calculus else expr
    if not value.free_symbols:
        exact = sp.simplify(value)
        if exact.is_Rational or sp.count_ops(exact) > 3:
            value = _format_number(exact)
            return {"latex": sp.latex(value), "text": str(value), "kind": "value"}
        # irracional corto (π, √2, ln 2): exacto y aproximado
        approx = sp.Float(sp.N(exact, 4), 4)
        return {"latex": f"{sp.latex(exact)} \\approx {sp.latex(approx)}", "text": f"{exact} ≈ {approx}",
                "kind": "value"}
    if isinstance(expr, sp.Integral) and len(expr.limits[0]) == 1:
        return {"latex": sp.latex(value) + " + C", "text": f"{value} + C", "kind": "antiderivative"}
    if has_calculus:
        return {"latex": sp.latex(sp.simplify(value)), "text": str(sp.simplify(value)), "kind": "calculus"}
    # SymPy ya agrupa términos al construir la expresión (2a+3a -> 5a), así que se compara con lo escrito
    simple = sp.simplify(expr)
    # si las definiciones de la página cambiaron algo, se muestra siempre; si no, solo si es más corto
    if cambiado or len(visible_tokens(sp.latex(simple))) < len(_written_symbols(last)):
        return {"latex": sp.latex(simple), "text": str(simple), "kind": "simplified"}
    return None


def _written_symbols(tokens):
    """Símbolos escritos de una fila de fichas (contando exponentes, subíndices, fracciones y raíces)."""

    def count(node):
        kind = node[0]
        if kind == "sym":
            return 1
        if kind == "row":
            return sum(count(c) for c in node[1])
        return sum(count(c) for c in node[1:] if isinstance(c, tuple)) + (1 if kind in ("frac", "sqrt") else 0)

    n = 0
    for t in tokens:
        if t[0] == "num":
            n += len(t[1])
        elif t[0] == "sym":
            n += 1
        else:
            n += count(t[1])
        n += sum(count(x) for x in (t[2], t[3]) if x is not None)
    return [None] * n


def _solve(eq, var):
    sols = sp.solve(eq, var)
    if not sols:
        return {"latex": r"\text{sin solución}", "text": "sin solución", "kind": "solve"}
    sols = [_format_number(s) if not s.free_symbols else s for s in sols]
    name = sp.latex(var)
    latex = ",\\; ".join(f"{name} = {sp.latex(s)}" for s in sols)
    return {"latex": latex, "text": ", ".join(f"{var} = {s}" for s in sols), "kind": "solve"}


def _with_units(expr, units):
    """Calcula con unidades: cada símbolo con unidad se sustituye por una cantidad de pint."""
    free = sorted(expr.free_symbols, key=lambda s: s.name)
    if any(s.name not in units for s in free):
        return None
    f = sp.lambdify(free, expr, modules=[{"sqrt": lambda q: q**0.5}, "math"])
    q = f(*[units[s.name] for s in free])
    if hasattr(q, "to_base_units"):
        from pint import DimensionalityError

        q = q.to_base_units()
        for u in ("N", "J", "W", "Pa", "C", "V", "ohm", "Hz"):
            try:
                q = q.to(u)
                break
            except DimensionalityError:
                pass
        if len(q.unit_items()) == 1:
            q = q.to_compact()
    mag = sp.Float(q.magnitude, 4) if hasattr(q, "magnitude") else sp.Float(q, 4)
    unit = f" {q.units:~P}" if hasattr(q, "units") and str(q.units) != "dimensionless" else ""
    return {"latex": sp.latex(mag) + (r"\," + rf"\mathrm{{{unit.strip()}}}" if unit else ""),
            "text": f"{mag}{unit}", "kind": "value"}
