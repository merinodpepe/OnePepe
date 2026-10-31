# SPDX-License-Identifier: GPL-3.0-or-later
"""Todo lo que se calcula con los renglones ya reconocidos de una página (orden `page` del programa auxiliar).

Cada vez que cambia algo (un símbolo nuevo, una corrección, una opción), la app envía los renglones de la página
con sus símbolos y recibe lo que hay que mostrar al lado de cada uno:

- el resultado del «=» final (y un «?» con lo que se ha leído cuando no se puede calcular);
- variables: `x = 3` define x en la página; los renglones que la usan muestran su valor (fase 3 de MyScript);
- revisión de pasos: cada paso de un desarrollo se compara con el anterior y, si no es equivalente, se marca y se
  explica el error típico que lo produce (fase 4);
- el LaTeX de cada renglón (fase 5) y las formas de una fracción: fracción, decimal y mixta (fase 6).

Nada de esto cambia la tinta: son anotaciones que la app dibuja encima.
"""

import random

import sympy as sp

from . import glossary as gl
from .evaluate import (
    MUL_OPS,
    RELATIONS,
    ParseError,
    _natural_log_names,
    _Parser,
    _split,
    _tokens,
    accent_token,
    answer,
)
from .layout import Sym, parse, to_latex
from .symbols import STEP_SEPARATORS


# ---------- utilidades ----------


def _row_syms(row):
    return [Sym(s["latex"], tuple(s["box"]), tuple(s.get("alts", ()))) for s in row.get("symbols", [])]


def _bounds(syms):
    return (
        min(s.box[0] for s in syms),
        min(s.box[1] for s in syms),
        max(s.box[2] for s in syms),
        max(s.box[3] for s in syms),
    )


def _typical(syms):
    hs = sorted(s.h for s in syms if s.label not in ("-", "=", ".", ",", r"\cdot", r"\frac"))
    return hs[len(hs) // 2] if hs else max((s.w for s in syms), default=10.0)


def _ends_with_equals(tokens):
    return bool(tokens) and tokens[-1][0] == "sym" and tokens[-1][1] == "=" and tokens[-1][2] is None


def _expr(tokens, env):
    """Expresión de SymPy de unas fichas, tal como se escribió (sin simplificar: hace falta para explicar errores)."""
    if not tokens:
        raise ParseError("vacío")
    with sp.evaluate(False):
        return _Parser(tokens, env).full()


def _value(expr):
    return sp.simplify(expr.doit())


# ---------- formas de un número (fase 6) ----------


def number_formats(value):
    """Formas de mostrar un número racional: [fracción, decimal, mixta]. Vacío si no es una fracción."""
    try:
        value = sp.nsimplify(value) if isinstance(value, sp.Float) else sp.Rational(value)
    except (TypeError, ValueError):
        return []
    if not isinstance(value, sp.Rational) or value.q == 1 or value.q > 1000:
        return []
    out = [{"text": f"{value.p}/{value.q}", "latex": sp.latex(value), "kind": "fraction"}]
    dec = sp.Float(value, 6)
    text = f"{float(dec):.6g}"
    out.append({"text": text, "latex": text, "kind": "decimal"})
    if abs(value) > 1:
        whole = int(abs(value.p) // value.q)
        rest = abs(value) - whole
        sign = "-" if value < 0 else ""
        out.append(
            {
                "text": f"{sign}{whole} {rest.p}/{rest.q}",
                "latex": f"{sign}{whole}\\tfrac{{{rest.p}}}{{{rest.q}}}",
                "kind": "mixed",
            }
        )
    return out


# ---------- variables (fase 3) ----------


def _unit_text(tokens):
    """Texto de unas fichas que podrían ser una unidad (kg, m/s^2, µm)."""
    out = ""
    for t in tokens:
        if t[0] != "sym" or t[3] is not None:
            raise ParseError("no es una unidad")
        v = t[1]
        if v == r"\mu":
            v = "µ"
        elif v == r"\Omega":
            v = "Ω"
        elif v in MUL_OPS:
            v = "*"
        elif v not in ("/",) and not v.isalpha():
            raise ParseError("no es una unidad")
        out += v
        if t[2] is not None:
            out += "**" + to_latex(t[2])
    return out


def _definition(tokens, env, units):
    """Si el renglón es `letra = valor` (o `m = 70 kg`), (nombre, valor) con valor sympy o cantidad de pint."""
    parts = _split(tokens, RELATIONS)
    if len(parts) != 2 or parts[0][1] != "=":
        return None
    lhs, rhs = parts[0][0], parts[1][0]
    if len(lhs) == 1 and lhs[0][0] == "accent":
        try:
            lhs = [accent_token(lhs[0])]
        except ParseError:
            return None
    if len(lhs) != 1 or lhs[0][0] != "sym" or lhs[0][2] is not None or not rhs:
        return None
    try:
        sym = _Parser([], {})._symbol(lhs[0])
    except ParseError:
        return None
    if not isinstance(sym, sp.Symbol):
        return None
    name = sym.name
    # número con unidad: lo que sigue a la primera ficha numérica es la unidad
    if len(rhs) >= 2 and rhs[0][0] == "num" and rhs[0][2] is None:
        try:
            unit = _unit_text(rhs[1:])
            q = gl.registry().Quantity(float(rhs[0][1]), unit)
            if not q.dimensionless:
                return name, q
        except Exception:  # noqa: BLE001 - no era una unidad
            pass
    try:
        value = _value(_expr(rhs, {**env, name: sp.Symbol(name)}))
    except Exception:  # noqa: BLE001
        return None
    if value.free_symbols or not value.is_number:
        return None
    return name, value


def _env_for(index, definitions):
    """Variables que valen en el renglón `index`: la última definición de cada una por encima; si no hay ninguna
    encima, la primera de la página."""
    env = {}
    for _, name, value in definitions:
        env.setdefault(name, value)
    for i, name, value in definitions:
        if i < index:
            env[name] = value
    return env


def _split_env(env):
    """(valores sympy, cantidades con unidad)."""
    plain = {k: v for k, v in env.items() if not hasattr(v, "magnitude")}
    quantities = {k: v for k, v in env.items() if hasattr(v, "magnitude")}
    return plain, quantities


# ---------- revisión de pasos (fase 4) ----------


def _numeric(expr, values):
    v = complex(expr.subs(values).doit().evalf())
    return v


def _random_points(symbols, n=6, seed=7):
    rnd = random.Random(seed)
    return [{s: sp.Rational(rnd.randint(55, 245), 100) for s in symbols} for _ in range(n)]


def same_expr(a, b):
    """¿Valen lo mismo dos expresiones para muchos valores al azar de sus variables?"""
    symbols = sorted(a.free_symbols | b.free_symbols, key=lambda s: s.name)
    ok = 0
    for point in _random_points(symbols):
        try:
            va, vb = _numeric(a, point), _numeric(b, point)
        except (TypeError, ValueError, ZeroDivisionError):
            continue
        if any(map(lambda z: z != z or abs(z) == float("inf"), (va, vb))):
            continue
        if abs(va - vb) > 1e-7 * (1 + abs(va)):
            return False
        ok += 1
    return ok > 0


def same_equation(e1, e2):
    """¿Tienen las mismas soluciones dos ecuaciones (lhs, rhs)?"""
    f1 = e1[0] - e1[1]
    f2 = e2[0] - e2[1]
    symbols = sorted(f1.free_symbols | f2.free_symbols, key=lambda s: s.name)
    if len(symbols) == 1:
        x = symbols[0]
        try:
            s1 = set(sp.solve(sp.Eq(f1.doit(), 0), x))
            s2 = set(sp.solve(sp.Eq(f2.doit(), 0), x))
            if s1 or s2:
                return {sp.nsimplify(v) for v in s1} == {sp.nsimplify(v) for v in s2}
        except Exception:  # noqa: BLE001 - si no se puede despejar, se compara de otra forma
            pass
    # proporcionales: f1 = k·f2 con k constante
    ratios = []
    for point in _random_points(symbols):
        try:
            v2 = _numeric(f2, point)
            if abs(v2) < 1e-9:
                continue
            ratios.append(_numeric(f1, point) / v2)
        except (TypeError, ValueError, ZeroDivisionError):
            continue
    return bool(ratios) and all(abs(r - ratios[0]) < 1e-7 * (1 + abs(ratios[0])) for r in ratios)


def _subexprs(expr, cls):
    return [e for e in sp.preorder_traversal(expr) if isinstance(e, cls)]


def _expression_mistakes(a):
    """Versiones de `a` con un error típico: [(expresión equivocada, explicación)]."""
    out = []
    with sp.evaluate(False):
        terms = sp.Add.make_args(a)
        if len(terms) > 1:
            for t in terms:
                out.append((sp.Add(*[(-u if u is t else u) for u in terms]), "Se ha perdido un signo: revisa el signo de cada término."))
        for m in _subexprs(a, sp.Mul):
            factors = m.args
            for k, f in enumerate(factors):
                if isinstance(f, sp.Add) and len(f.args) > 1:
                    others = sp.Mul(*[g for j, g in enumerate(factors) if j != k])
                    wrong = sp.Add(sp.Mul(others, f.args[0]), *f.args[1:])
                    out.append((a.xreplace({m: wrong}), "Falta multiplicar el factor por todos los términos del paréntesis."))
        for p in _subexprs(a, sp.Pow):
            base, exp = p.args
            if isinstance(base, sp.Add) and exp.is_Integer and exp > 1:
                wrong = sp.Add(*[sp.Pow(t, exp) for t in base.args])
                out.append((a.xreplace({p: wrong}), "Una potencia no se reparte en una suma: (a+b)² = a² + 2ab + b²."))
            if isinstance(base, sp.Add) and exp == sp.Rational(1, 2):
                wrong = sp.Add(*[sp.sqrt(t) for t in base.args])
                out.append((a.xreplace({p: wrong}), "La raíz no se reparte en una suma: √(a+b) ≠ √a + √b."))
        for s in _subexprs(a, sp.Add):
            fracs = [t for t in s.args if isinstance(t, sp.Mul) and any(isinstance(g, sp.Pow) and g.exp == -1 for g in t.args)]
            if len(fracs) >= 2:
                nums, dens = [], []
                for t in fracs[:2]:
                    n, d = sp.fraction(t)
                    nums.append(n)
                    dens.append(d)
                wrong = sp.Add(sp.Mul(sp.Add(*nums), sp.Pow(sp.Add(*dens), -1)), *[t for t in s.args if t not in fracs[:2]])
                out.append((a.xreplace({s: wrong}), "Las fracciones no se suman sumando numeradores y denominadores: busca el denominador común."))
    return out


def _equation_mistakes(lhs, rhs):
    """Versiones de la ecuación lhs = rhs con un error típico."""
    out = []
    with sp.evaluate(False):
        for side, other, left in ((lhs, rhs, True), (rhs, lhs, False)):
            terms = sp.Add.make_args(side)
            if len(terms) > 1:
                for t in terms:
                    rest = sp.Add(*[u for u in terms if u is not t])
                    moved = sp.Add(other, t)
                    eq = (rest, moved) if left else (moved, rest)
                    out.append((eq, "Has pasado un término al otro lado sin cambiarle el signo."))
        # a·x = b  →  x = b − a, x = b·a o x = a/b
        if isinstance(lhs, sp.Mul):
            numbers = [f for f in lhs.args if f.is_number]
            rest = [f for f in lhs.args if not f.is_number]
            if len(numbers) == 1 and rest:
                a, x = numbers[0], sp.Mul(*rest)
                out.append(((x, sp.Add(rhs, -a)), "El número que multiplica pasa al otro lado dividiendo, no restando."))
                out.append(((x, sp.Mul(rhs, a)), "El número que multiplica pasa al otro lado dividiendo, no multiplicando."))
                out.append(((x, sp.Mul(a, sp.Pow(rhs, -1))), "Al despejar, la división va al revés."))
    for wrong, why in _expression_mistakes(lhs):
        out.append(((wrong, rhs), why))
    for wrong, why in _expression_mistakes(rhs):
        out.append(((lhs, wrong), why))
    return out


def explain_expressions(a, b):
    for wrong, why in _expression_mistakes(a):
        try:
            if same_expr(wrong, b):
                return why
        except Exception:  # noqa: BLE001
            continue
    return "No es equivalente al paso anterior."


def explain_equations(e1, e2):
    for wrong, why in _equation_mistakes(*e1):
        try:
            if same_equation(wrong, e2):
                return why
        except Exception:  # noqa: BLE001
            continue
    return "No tiene las mismas soluciones que el paso anterior."


def _statement(tokens, env):
    """("eq", (lhs, rhs)) o ("expr", e) o None."""
    parts = _split(tokens, RELATIONS)
    if len(parts) == 1:
        return ("expr", _expr(tokens, env))
    if len(parts) == 2 and parts[0][1] == "=":
        return ("eq", (_expr(parts[0][0], env), _expr(parts[1][0], env)))
    return None


def check_steps(a, b):
    """Compara dos pasos. None si no se pueden comparar; "" si están bien; si no, la explicación."""
    if a is None or b is None or a[0] != b[0]:
        return None
    if a[0] == "expr":
        return "" if same_expr(a[1], b[1]) else explain_expressions(a[1], b[1])
    if not (a[1][0] - a[1][1]).free_symbols & (b[1][0] - b[1][1]).free_symbols:
        return None
    return "" if same_equation(a[1], b[1]) else explain_equations(a[1], b[1])


# ---------- la página ----------


def _mark(kind, text, latex, anchor, height, **extra):
    out = {"kind": kind, "text": text, "latex": latex, "anchor": [round(anchor[0], 2), round(anchor[1], 2)],
           "height": round(height, 2)}
    out.update(extra)
    return out


def _after(box, typical):
    return (box[2] + 0.3 * max(typical, 8.0), (box[1] + box[3]) / 2)


def page(rows, equals=True, variables=False, check=False, glossary_units=None):
    """Anotaciones de cada renglón. `rows`: [{"symbols": [{"latex", "box"}]}], de arriba abajo."""
    glossary_units = glossary_units or {}
    parsed = []
    for row in rows:
        syms = _row_syms(row)
        if not syms:
            parsed.append(None)
            continue
        tree = parse(syms)
        parsed.append((syms, tree, _tokens(tree)))
    # variables definidas en la página
    definitions = []
    if variables:
        env = {}
        for i, p in enumerate(parsed):
            if p is None or _ends_with_equals(p[2]):
                continue
            d = _definition(p[2], _split_env(env)[0], glossary_units)
            if d:
                env[d[0]] = d[1]
                definitions.append((i, d[0], d[1]))
    out = []
    previous = None
    for i, p in enumerate(parsed):
        if p is None:
            out.append({"latex": "", "marks": []})
            previous = None
            continue
        syms, tree, tokens = p
        typical = _typical(syms)
        box = _bounds(syms)
        marks = []
        env = _env_for(i, definitions) if variables else {}
        plain, quantities = _split_env(env)
        units = {**glossary_units, **quantities}
        is_definition = any(j == i for j, _, _ in definitions)
        # 1. el «=» final
        if equals and _ends_with_equals(tokens):
            eq = max((s for s in syms if s.label == "="), key=lambda s: s.box[2])
            anchor = _after(eq.box, typical)
            height = max(eq.w, eq.h, 0.8 * typical) * 1.2
            result = answer(tree, glossary=plain, units=units or None)
            if result is None and units:
                result = answer(tree, glossary=plain)
            if result is None:
                marks.append(_mark("unknown", "?", to_latex(tree), anchor, height, at=syms.index(eq)))
            else:
                formats = []
                if result["kind"] == "value":
                    try:
                        formats = number_formats(sp.sympify(result["text"]))
                    except (sp.SympifyError, TypeError, ValueError):
                        formats = []
                marks.append(_mark(result["kind"], result["text"], result["latex"], anchor, height, formats=formats,
                                   at=syms.index(eq)))
        # 2. variables: una expresión que las usa muestra su valor
        elif variables and env and not is_definition and len(tokens) >= 2:
            parts = _split(tokens, RELATIONS | {","} | STEP_SEPARATORS)
            if len(parts) == 1:
                try:
                    expr = _Parser(tokens, plain).full()
                    free = {s.name for s in expr.free_symbols}
                    used = {s.name for s in _Parser(tokens, {}).full().free_symbols} & set(env)
                    if used and (not free or free <= set(quantities)):
                        result = None
                        if quantities:
                            result = answer(("row", tree[1] + [("sym", "=")]), glossary=plain, units=units)
                        else:
                            value = _value(expr)
                            result = _natural_log_names(
                                {"text": str(value if value.is_Rational else sp.Float(value, 4)),
                                 "latex": sp.latex(value if value.is_Rational else sp.Float(value, 4)),
                                 "kind": "value"})
                        if result:
                            text = "= " + result["text"]
                            formats = number_formats(sp.sympify(result["text"])) if not quantities else []
                            for f in formats:
                                f["text"] = "= " + f["text"]
                            marks.append(_mark("variable", text, "= " + result["latex"], _after(box, typical),
                                               0.9 * typical, formats=formats))
                except Exception:  # noqa: BLE001 - un renglón que no se entiende no anota nada
                    pass
        # 3. revisión de pasos
        if check:
            marks += _check_row(tokens, syms, typical, box, previous, plain)
        out.append({"latex": to_latex(tree), "marks": marks})
        previous = (tokens, syms)
    return out


def _check_row(tokens, syms, typical, box, previous, env):
    """Pasos dentro del renglón (separados por → ⇒ ;, o una cadena A = B = C) y con el renglón de encima."""
    marks = []
    if _ends_with_equals(tokens):
        tokens = tokens[:-1]

    def bad(why, where):
        # debajo del renglón, empezando por la izquierda: así no se cruza con el resultado del «=» ni con el valor
        # de las variables, que van a la derecha
        return _mark("error", "✗ " + why, why, (where[0], where[3] + 0.55 * typical), 0.55 * typical)

    # «= …» sigue la expresión del renglón de encima
    continues = bool(tokens) and tokens[0][0] == "sym" and tokens[0][1] == "="
    body = tokens[1:] if continues else tokens
    try:
        steps = [t for t, _ in _split(body, STEP_SEPARATORS) if t]
        stmts = [_statement(t, env) for t in steps]
        for a, b in zip(stmts, stmts[1:]):
            why = check_steps(a, b)
            if why:
                return [bad(why, box)]
        # cadena «A = B = C»: cada parte igual a la anterior
        parts = [t for t, _ in _split(body, {"="})]
        if len(parts) >= 3 and all(parts):
            exprs = [_expr(t, env) for t in parts]
            for a, b in zip(exprs, exprs[1:]):
                if not same_expr(a, b):
                    return [bad(explain_expressions(a, b), box)]
        # con el renglón de encima
        if previous is not None:
            prev_tokens = previous[0][:-1] if _ends_with_equals(previous[0]) else previous[0]
            prev_steps = [t for t, _ in _split(prev_tokens, STEP_SEPARATORS) if t]
            if not prev_steps:
                return marks
            if continues:
                a = _expr(_split(prev_steps[-1], {"="})[-1][0], env)
                b = _expr(_split(body, {"="})[0][0], env)
                if not same_expr(a, b):
                    return [bad(explain_expressions(a, b), box)]
            elif stmts:
                why = check_steps(_statement(prev_steps[-1], env), stmts[0])
                if why:
                    return [bad(why, box)]
    except Exception:  # noqa: BLE001 - lo que no se entiende no se marca
        return marks
    return marks


# ---------- LaTeX de varios renglones (fase 5) ----------


def latex_rows(rows):
    out = []
    for row in rows:
        syms = _row_syms(row)
        if syms:
            out.append(to_latex(parse(syms)))
    return out
