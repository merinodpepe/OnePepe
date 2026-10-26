#!/usr/bin/env bash
# OnePepe: comprueba e instala las dependencias para compilar Rnote/OnePepe en el equipo.
#
# Uso:
#   ./setup-deps.sh            # comprueba qué falta e instala solo eso (pide sudo)
#   ./setup-deps.sh --check    # solo comprueba, no instala nada
#   ./setup-deps.sh --flatpak  # prepara la compilación con Flatpak (alternativa recomendada)
#
# Lista de paquetes tomada de BUILDING.md. Versiones mínimas tomadas de meson.build:
#   glib-2.0 >= 2.88, gio-2.0 >= 2.88, cairo >= 1.18, gtk4 >= 4.22, libadwaita-1 >= 1.9, appstream >= 1.0
# Son versiones muy recientes (GNOME 50). Si tu distribución trae versiones más viejas,
# no se pueden "instalar las que faltan" con el gestor de paquetes: usa Flatpak (--flatpak).

set -u

MODE="install"
case "${1:-}" in
    --check) MODE="check" ;;
    --flatpak) MODE="flatpak" ;;
    "") ;;
    *) echo "Opción desconocida: $1"; exit 2 ;;
esac

# ---------- Flatpak ----------
if [ "$MODE" = "flatpak" ]; then
    if ! command -v flatpak >/dev/null || ! command -v flatpak-builder >/dev/null; then
        echo "Falta flatpak o flatpak-builder. Instálalos con tu gestor de paquetes:"
        echo "  Debian/Ubuntu: sudo apt install flatpak flatpak-builder"
        echo "  Fedora:        sudo dnf install flatpak flatpak-builder"
        echo "  Arch:          sudo pacman -S flatpak flatpak-builder"
        exit 1
    fi
    flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
    # Mismas versiones que el manifiesto build-aux/com.github.flxzt.rnote.Devel.yaml
    flatpak install --user -y flathub org.gnome.Platform//50 org.gnome.Sdk//50 \
        org.freedesktop.Sdk.Extension.rust-stable//25.08
    echo
    echo "Listo. Para compilar e instalar OnePepe (tarda bastante la primera vez):"
    echo "  flatpak-builder --user --install --force-clean _flatpak_app build-aux/com.github.flxzt.rnote.Devel.yaml"
    echo "Para ejecutarlo:"
    echo "  flatpak run com.github.flxzt.rnote.Devel"
    exit 0
fi

# ---------- Detectar gestor de paquetes ----------
if command -v apt-get >/dev/null; then
    PM="apt"
elif command -v dnf >/dev/null; then
    PM="dnf"
elif command -v pacman >/dev/null; then
    PM="pacman"
else
    echo "No reconozco el gestor de paquetes (apt, dnf o pacman). Usa: ./setup-deps.sh --flatpak"
    exit 1
fi
echo "Gestor de paquetes detectado: $PM"

# módulo pkg-config | versión mínima | paquete apt | paquete dnf | paquete pacman
PC_DEPS=(
    "glib-2.0|2.88|libglib2.0-dev|glib2-devel|glib2"
    "gio-2.0|2.88|libglib2.0-dev|glib2-devel|glib2"
    "cairo|1.18|libcairo2-dev|cairo-devel|cairo"
    "gtk4|4.22|libgtk-4-dev|gtk4-devel|gtk4"
    "libadwaita-1|1.9|libadwaita-1-dev|libadwaita-devel|libadwaita"
    "appstream|1.0|libappstream-dev|appstream-devel|appstream"
    "alsa|0|libasound2-dev|alsa-lib-devel|alsa-lib"
    # No los pide BUILDING.md, pero GTK los usa para la tableta (presión, botones del lápiz):
    "libinput|0|libinput-dev|libinput-devel|libinput"
    "libwacom|0|libwacom-dev|libwacom-devel|libwacom"
)

# programa | paquete apt | paquete dnf | paquete pacman
BIN_DEPS=(
    "gcc|build-essential|gcc|base-devel"
    "g++|build-essential|gcc-c++|base-devel"
    "clang|clang|clang|clang"
    "python3|python3|python3|python"
    "make|make|make|make"
    "cmake|cmake|cmake|cmake"
    "meson|meson|meson|meson"
    "just|just|just|just"
    "git|git|git|git"
    "msgfmt|gettext|gettext|gettext"
    "desktop-file-validate|desktop-file-utils|desktop-file-utils|desktop-file-utils"
    "update-mime-database|shared-mime-info|shared-mime-info|shared-mime-info"
    "appstreamcli|appstream|appstream|appstream"
    "pkg-config|pkg-config|pkgconf-pkg-config|pkgconf"
)

pkg_for() { # $1 = campos apt|dnf|pacman
    IFS='|' read -r a d p <<<"$1"
    case "$PM" in apt) echo "$a" ;; dnf) echo "$d" ;; pacman) echo "$p" ;; esac
}

MISSING=()
TOO_OLD=()

for b in "${BIN_DEPS[@]}"; do
    IFS='|' read -r bin rest <<<"$b"
    if ! command -v "$bin" >/dev/null; then
        echo "  FALTA programa: $bin"
        MISSING+=("$(pkg_for "$rest")")
    fi
done

for d in "${PC_DEPS[@]}"; do
    IFS='|' read -r mod min rest <<<"$d"
    if ! command -v pkg-config >/dev/null; then
        MISSING+=("$(pkg_for "$rest")")
        continue
    fi
    if pkg-config --exists "$mod"; then
        have=$(pkg-config --modversion "$mod")
        if [ "$min" != "0" ] && ! pkg-config --atleast-version="$min" "$mod"; then
            echo "  DEMASIADO VIEJA: $mod $have (se necesita >= $min)"
            TOO_OLD+=("$mod $have < $min")
        else
            echo "  ok: $mod $have"
        fi
    else
        echo "  FALTA biblioteca: $mod"
        MISSING+=("$(pkg_for "$rest")")
    fi
done

if ! command -v cargo >/dev/null; then
    echo "  FALTA Rust (cargo). Instálalo con:"
    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

# quitar duplicados
mapfile -t MISSING < <(printf '%s\n' "${MISSING[@]}" | sort -u | sed '/^$/d')

if [ "${#MISSING[@]}" -gt 0 ]; then
    echo
    echo "Paquetes que faltan: ${MISSING[*]}"
    if [ "$MODE" = "install" ]; then
        case "$PM" in
            apt) sudo apt-get update && sudo apt-get install -y "${MISSING[@]}" ;;
            dnf) sudo dnf install -y "${MISSING[@]}" ;;
            pacman) sudo pacman -S --needed --noconfirm "${MISSING[@]}" ;;
        esac
        echo "Vuelve a ejecutar ./setup-deps.sh --check para confirmar."
    fi
else
    echo
    echo "No falta ningún paquete."
fi

if [ "${#TOO_OLD[@]}" -gt 0 ]; then
    echo
    echo "ATENCIÓN: tu distribución trae versiones demasiado viejas:"
    printf '  - %s\n' "${TOO_OLD[@]}"
    echo "No se puede compilar de forma nativa. Usa Flatpak:"
    echo "  ./setup-deps.sh --flatpak"
    exit 1
fi

echo
echo "Para compilar de forma nativa (BUILDING.md):"
echo "  meson setup --prefix=/usr _mesonbuild"
echo "  meson compile -C _mesonbuild"
echo "  sudo meson install -C _mesonbuild   # instala también el gschema de ajustes"
echo "  rnote"
