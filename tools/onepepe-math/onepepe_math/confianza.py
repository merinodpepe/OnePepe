# SPDX-License-Identifier: GPL-3.0-or-later
"""Umbrales de confianza calibrados (M8): `confianza.json` y la cota de Wilson."""

import json
import math
import os

Z90 = 1.2816  # unilateral al 90 %


def wilson_cs(k, n, z=Z90):
    """Cota superior de Wilson del error con k fallos en n."""
    if not n:
        return 1.0
    q = k / n
    return (q + z * z / (2 * n) + z * math.sqrt(q * (1 - q) / n + z * z / (4 * n * n))) / (1 + z * z / n)


def cargar(datos=None):
    """El contenido de `confianza.json` ({} si no existe o no se lee)."""
    ruta = os.path.join(datos or os.path.join(os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math"),
                        "confianza.json")
    try:
        with open(ruta, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {}
