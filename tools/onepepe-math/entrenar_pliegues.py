#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""OnePepe: entrena los cuatro modelos de la medida cruzada (medir.py --cruzada RECETA).

El modelo k se entrena sin el pliegue k de tus expresiones y sin el examen, con el peso y las épocas de la receta
(sin parada). Se guardan en $ONEPEPE_DATA/pliegues/RECETA/pliegue<k>.npz. Los modelos nunca van a git.

    python3 entrenar_pliegues.py --receta v1 --peso 30 --epocas 20
"""

import argparse
import os
import subprocess
import sys

DIR = os.path.dirname(os.path.abspath(__file__))
DATA = os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe"))
DATASET = os.environ.get("ONEPEPE_MATHWRITING", os.path.join(DATA, "mathwriting", "mathwriting-2024"))


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--receta", required=True)
    ap.add_argument("--peso", required=True, choices=["10", "30", "90"])
    ap.add_argument("--epocas", required=True, type=int)
    ap.add_argument("--personal", default=os.path.join(DATA, "math", "correcciones.jsonl"))
    args = ap.parse_args(argv)
    salida = os.path.join(DATA, "pliegues", args.receta)
    os.makedirs(salida, exist_ok=True)
    for k in range(4):
        cmd = [sys.executable, os.path.join(DIR, "train_classifier.py"), DATASET, "-o",
               os.path.join(salida, f"pliegue{k}.npz"), "--personal", args.personal, "--personal-weight", args.peso,
               "--epocas-fijas", str(args.epocas), "--excluir-pliegue", str(k)]
        print(f"pliegue {k}: {' '.join(cmd[2:])}", flush=True)
        subprocess.run(cmd, check=True, cwd=DIR)


if __name__ == "__main__":
    main()
