# SPDX-License-Identifier: GPL-3.0-or-later
"""Glosario por cuaderno: símbolos habituales y, si se quiere, su valor con unidades.

Archivo de texto `.onepepe-glosario.txt` en la carpeta del cuaderno (o de la sección). Una entrada por línea:

    k_B = 1.380649e-23 J/K     # constante de Boltzmann
    N_A = 6.02214076e23 1/mol
    T                          # solo el símbolo: parte con ventaja al reconocer
"""

import os
import re

FILE_NAME = ".onepepe-glosario.txt"


def find_glossary(start_dir):
    """Busca el glosario en la carpeta y sus carpetas superiores (sección -> cuaderno)."""
    d = os.path.abspath(start_dir) if start_dir else None
    while d:
        path = os.path.join(d, FILE_NAME)
        if os.path.exists(path):
            return path
        parent = os.path.dirname(d)
        if parent == d:
            return None
        d = parent
    return None


def load(path):
    """Devuelve (nombres: set, valores: {nombre: texto con valor y unidad})."""
    names, values = set(), {}
    if not path or not os.path.exists(path):
        return names, values
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            if "=" in line:
                name, value = (x.strip() for x in line.split("=", 1))
                values[name] = value
            else:
                name = line
            names.add(name)
    return names, values


def symbol_labels(names):
    """Símbolos sueltos que aparecen en los nombres del glosario (k_B -> k, B; \\Delta G -> \\Delta, G)."""
    out = set()
    for name in names:
        for tok in re.findall(r"\\[A-Za-z]+|[A-Za-z0-9]", name):
            out.add(tok)
    return out


def add_name(path, name):
    """Añade un símbolo confirmado al glosario."""
    names, _ = load(path)
    if name in names:
        return
    with open(path, "a", encoding="utf-8") as f:
        f.write(name + "\n")


def units(values):
    """{nombre: cantidad de pint} a partir de los valores del glosario."""
    if not values:
        return {}
    import pint

    ureg = pint.UnitRegistry()
    out = {}
    for name, text in values.items():
        try:
            out[name] = ureg.Quantity(text)
        except Exception:  # noqa: BLE001 - un valor mal escrito no debe romper el reconocimiento
            continue
    return out
