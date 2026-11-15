# SPDX-License-Identifier: GPL-3.0-or-later
"""M8.3: elige el umbral de sustitución (tau*) y el del renglón (tau_r) con una medida cruzada hecha con --umbral 0.

    medir.py --sesiones "$ENTRENOS" --cruzada v1 --conjunto entreno --umbral 0 --salida cruzada.json
    calibrar_umbral.py cruzada.json            # escribe $ONEPEPE_DATA/math/confianza.json
"""

import json
import os
import sys
from datetime import datetime

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from onepepe_math.confianza import wilson_cs  # noqa: E402

TAUS = (0.90, 0.95, 0.97, 0.98, 0.99, 0.995, 0.999)
TAUS_R = (0.5, 0.6, 0.7, 0.8, 0.9)
CS_MAX, COBERTURA_MIN = 0.02, 0.60


def calibrar(simbolos, renglones):
    """`simbolos`: [[c, acuerdo, ok]]; `renglones`: [[mín c, perfecto]]. Devuelve el dict de confianza.json + la tabla."""
    N = len(simbolos)
    tabla = []
    for t in TAUS:
        sel = [ok for c, acuerdo, ok in simbolos if acuerdo and c >= t]
        n, k = len(sel), sum(1 for ok in sel if not ok)
        tabla.append({"tau": t, "n": n, "k": k, "CS": round(wilson_cs(k, n), 4), "cobertura": round(n / N, 4) if N else 0})
    buenos = [r for r in tabla if r["CS"] <= CS_MAX and r["n"]]
    plenos = [r for r in buenos if r["cobertura"] >= COBERTURA_MIN]
    elegido = plenos[0] if plenos else max(buenos, key=lambda r: r["cobertura"]) if buenos else dict(tabla[-1])
    tau_r = next((t for t in TAUS_R if (g := [p for m, p in renglones if m >= t]) and sum(g) >= 0.9 * len(g)), 0.7)
    return {"umbral": elegido["tau"], "umbral_renglon": tau_r, "n": elegido["n"], "k": elegido["k"],
            "cobertura": elegido["cobertura"], "fecha": datetime.now().strftime("%Y-%m-%d"),
            "cumple": bool(plenos), "tabla": tabla}


def main():
    med = json.load(open(sys.argv[1], encoding="utf-8"))
    res = calibrar(med["simbolos_pred"], med["renglones_pred"])
    datos = os.path.join(os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe")), "math")
    with open(os.path.join(datos, "confianza.json"), "w", encoding="utf-8") as f:
        json.dump({k: res[k] for k in ("umbral", "umbral_renglon", "n", "k", "cobertura", "fecha")}, f, indent=1)
    print(json.dumps(res, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
