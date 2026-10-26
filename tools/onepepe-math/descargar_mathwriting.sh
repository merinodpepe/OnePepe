#!/usr/bin/env bash
# OnePepe: descarga el dataset MathWriting de Google (fórmulas escritas a mano con sus trazos)
# FUERA del repositorio, en ~/.local/share/onepepe/mathwriting.
#
# Licencia de los datos: CC BY-NC-SA 4.0 (uso personal y no comercial: es tu caso).
# Más información: https://github.com/google-research/google-research/tree/master/mathwriting
#
# Uso:
#   ./descargar_mathwriting.sh            # dataset completo (2,9 GB comprimido)
#   ./descargar_mathwriting.sh --muestra  # solo un extracto de 1,5 MB para probar
set -euo pipefail

DEST="${ONEPEPE_DATA:-$HOME/.local/share/onepepe}/mathwriting"
BASE="https://storage.googleapis.com/mathwriting_data"
FILE="mathwriting-2024.tgz"
[ "${1:-}" = "--muestra" ] && FILE="mathwriting-2024-excerpt.tgz"

mkdir -p "$DEST"
echo "Descargando $FILE en $DEST ..."
curl -L --fail --continue-at - -o "$DEST/$FILE" "$BASE/$FILE"
echo "Descomprimiendo ..."
tar xzf "$DEST/$FILE" -C "$DEST"
echo "Listo: $(ls "$DEST")"
