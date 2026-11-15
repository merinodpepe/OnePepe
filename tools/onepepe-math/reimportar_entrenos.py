#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: vuelve a importar las sesiones de entreno con el alineado ordenado y el control de calidad.

Sin `--aplicar` trabaja sobre una copia temporal del almacén e informa. Con `--aplicar`: guarda
`correcciones.<AAAAMMDD-HHMMSS>.bak`, quita los ejemplos `entreno:*`, importa todas las sesiones (ignorando la marca
`imported`) y deja lo dudoso en `revision.jsonl`. Las sesiones nunca se modifican.

Uso:
    python3 reimportar_entrenos.py --sesiones "$HOME/Documentos/OnePepe entrenos" [--aplicar]
"""

import argparse
import glob
import json
import os
import shutil
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from onepepe_math import entreno  # noqa: E402
from onepepe_math.personal import DEFAULT_PATH, PersonalModel  # noqa: E402


def reimportar(sesiones, clf, ruta=DEFAULT_PATH, aplicar=False):
    """Informe {"entran", "revision", "fallidas", "examen", "trozos"} (símbolos, símbolos, expresiones, expresiones,
    trozos)."""
    examen = entreno.ids_examen(os.path.dirname(ruta))
    tmp = None
    if aplicar:
        if os.path.exists(ruta):
            shutil.copy2(ruta, f"{ruta[: -len('.jsonl')]}.{time.strftime('%Y%m%d-%H%M%S')}.bak")
    else:  # ensayo: una copia temporal del almacén; nada del original cambia
        tmp = tempfile.mkdtemp()
        if os.path.exists(ruta):
            shutil.copy2(ruta, tmp)
        ruta = os.path.join(tmp, os.path.basename(ruta))
    manuales = []
    if os.path.exists(ruta):
        with open(ruta, encoding="utf-8") as f:
            manuales = [x for x in f if not str(json.loads(x).get("notebook") or "").startswith("entreno:")]
    with open(ruta, "w", encoding="utf-8") as f:
        f.writelines(manuales)
    revision = os.path.join(os.path.dirname(ruta), "revision.jsonl")
    if os.path.exists(revision):
        os.remove(revision)
    store = PersonalModel(clf, ruta, background=False)
    total = {"entran": 0, "revision": 0, "fallidas": 0, "examen": 0, "trozos": 0}
    for f in sorted(glob.glob(os.path.join(sesiones, "*.entreno.json"))):
        with open(f, encoding="utf-8") as fh:
            out = entreno.import_session(json.load(fh), clf, store, forzar=True, examen=examen)
        total["entran"] += out["symbols"]
        total["revision"] += out["revision"]
        total["fallidas"] += len(out["failed"])
        total["examen"] += len(out["examen"])
        total["trozos"] += out["junk"]
    if tmp:
        shutil.rmtree(tmp)
    return total


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--sesiones", default=os.path.expanduser("~/Documentos/OnePepe entrenos"))
    ap.add_argument("--aplicar", action="store_true")
    ap.add_argument("--almacen", default=DEFAULT_PATH)
    ap.add_argument("--modelo", default=None)
    args = ap.parse_args(argv)
    from onepepe_math.classifier import SymbolClassifier

    clf = SymbolClassifier(args.modelo) if args.modelo else SymbolClassifier()
    t = reimportar(args.sesiones, clf, args.almacen, args.aplicar)
    print(f"{'Aplicado' if args.aplicar else 'Ensayo (nada cambia)'}: entran {t['entran']} símbolos y {t['trozos']} "
          f"trozos; a revisión {t['revision']} símbolos; fallidas {t['fallidas']} expresiones; "
          f"del examen {t['examen']} expresiones.")


if __name__ == "__main__":
    main()
