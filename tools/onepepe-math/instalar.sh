#!/usr/bin/env bash
# OnePepe: instala el programa auxiliar de matemáticas en tu equipo (fuera del repositorio).
#
#   ./instalar.sh             # entorno de Python en ~/.local/share/onepepe/venv y orden ~/.local/bin/onepepe-math
#   ./instalar.sh --entrenar  # además descarga MathWriting (2,9 GB) y entrena el clasificador (unos minutos)
#
# Nada de esto se sube al repositorio: el modelo queda en ~/.local/share/onepepe/math/simbolos.npz y tus
# correcciones en ~/.local/share/onepepe/math/correcciones.jsonl.
set -euo pipefail

DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
DATA="${ONEPEPE_DATA:-$HOME/.local/share/onepepe}"
VENV="$DATA/venv"

mkdir -p "$DATA" "$HOME/.local/bin"
if [ ! -x "$VENV/bin/python" ]; then
    echo "Creando el entorno de Python en $VENV ..."
    python3 -m venv "$VENV"
fi
"$VENV/bin/pip" install --upgrade pip >/dev/null
"$VENV/bin/pip" install -r "$DIR/requirements.txt"
ln -sf "$DIR/onepepe-math" "$HOME/.local/bin/onepepe-math"
echo "Orden instalada: $HOME/.local/bin/onepepe-math"
case ":$PATH:" in
    *":$HOME/.local/bin:"*) ;;
    *) echo "AVISO: añade ~/.local/bin a tu PATH (p. ej. en ~/.bashrc: export PATH=\"\$HOME/.local/bin:\$PATH\")" ;;
esac

if [ "${1:-}" = "--entrenar" ]; then
    "$VENV/bin/pip" install -r "$DIR/requirements-train.txt"
    if [ ! -d "$DATA/mathwriting/mathwriting-2024" ]; then
        "$DIR/descargar_mathwriting.sh"
    fi
    # tus correcciones, si las hay, entran en el entrenamiento
    PERSONAL=()
    if [ -f "$DATA/math/correcciones.jsonl" ]; then
        PERSONAL=(--personal "$DATA/math/correcciones.jsonl")
    fi
    "$VENV/bin/python" "$DIR/train_classifier.py" "$DATA/mathwriting/mathwriting-2024" "${PERSONAL[@]}"
fi

if [ ! -f "$DATA/math/simbolos.npz" ]; then
    echo "Falta el modelo de símbolos. Ejecuta: $DIR/instalar.sh --entrenar"
fi
echo 'Prueba rápida:  echo '"'"'{"id":1,"cmd":"ping"}'"'"' | onepepe-math'
