#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Promueve los datos de la copia de trabajo del cierre a los datos reales.

Sin --aplicar solo informa. Con --aplicar: copia cada archivo real a <nombre>.<AAAAMMDD-HHMMSS>.bak e instala los de
la copia de trabajo (el simbolos.npz anterior pasa a simbolos.anterior.npz). Si el original de correcciones.jsonl ha
cambiado desde origen.sha256, sus líneas nuevas se añaden al resultado; si cambió simbolos.npz, para.
"""
import argparse
import hashlib
import os
import shutil
import sys
import time

INSTALAR = ["correcciones.jsonl", "simbolos.npz", "examen.json", "examen_trazos.json", "referencia.json", "confianza.json"]


def sha(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def promover(real, trabajo, origen, aplicar=False, sin_confianza=False, sello=None, out=print):
    """Devuelve 0 si ok, 1 si hay que parar."""
    esperado = {}
    with open(origen) as f:
        for linea in f:
            if linea.strip():
                h, n = linea.split(None, 1)
                esperado[n.strip()] = h
    cambiados = [n for n, h in esperado.items() if not os.path.exists(os.path.join(real, n)) or sha(os.path.join(real, n)) != h]
    for n in esperado:
        out(f"original {n}: {'CAMBIADO desde origen.sha256' if n in cambiados else 'coincide con origen.sha256'}")
    if "simbolos.npz" in cambiados:
        out("PARO: simbolos.npz real ha cambiado; no se puede fusionar. Revisa a mano.")
        return 1
    archivos = [n for n in INSTALAR if not (sin_confianza and n == "confianza.json")]
    faltan = [n for n in archivos if not os.path.exists(os.path.join(trabajo, n))]
    if faltan:
        out(f"PARO: faltan en la copia de trabajo: {faltan}")
        return 1
    nuevas = []
    if "correcciones.jsonl" in cambiados:
        with open(os.path.join(trabajo, "correcciones.jsonl"), "rb") as f:
            propias = set(f.read().splitlines())
        with open(os.path.join(real, "correcciones.jsonl"), "rb") as f:
            nuevas = [x for x in f.read().splitlines() if x and x not in propias]
        out(f"líneas nuevas del original que se añadirían: {len(nuevas)}")
    for n in archivos:
        out(f"instalaría {n}")
    if not aplicar:
        out("(modo informe: no se ha escrito nada)")
        return 0
    sello = sello or time.strftime("%Y%m%d-%H%M%S")
    for n in archivos:
        dst = os.path.join(real, n)
        if os.path.exists(dst):
            shutil.copy2(dst, f"{dst}.{sello}.bak")
            if n == "simbolos.npz":
                shutil.copy2(dst, os.path.join(real, "simbolos.anterior.npz"))
        shutil.copy2(os.path.join(trabajo, n), dst)
    if nuevas:
        with open(os.path.join(real, "correcciones.jsonl"), "ab") as f:
            f.write(b"\n".join(nuevas) + b"\n")
    out("aplicado")
    return 0


def main():
    datos = os.environ.get("ONEPEPE_DATA", os.path.expanduser("~/.local/share/onepepe"))
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--aplicar", action="store_true")
    ap.add_argument("--sin-confianza", action="store_true", help="no instala confianza.json (se queda el umbral de la app)")
    ap.add_argument("--real", default=os.path.expanduser("~/.local/share/onepepe/math"))
    ap.add_argument("--trabajo", default=os.path.join(datos, "math"))
    ap.add_argument("--origen", default=os.path.join(datos, "origen.sha256"))
    a = ap.parse_args()
    sys.exit(promover(a.real, a.trabajo, a.origen, a.aplicar, a.sin_confianza))


if __name__ == "__main__":
    main()
