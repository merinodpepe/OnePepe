# Compilar OnePepe

Guía corta para compilar OnePepe en Linux. Hay dos caminos:

- **Flatpak (recomendado):** trae todo lo necesario (GNOME 50) y funciona en cualquier distribución.
- **Nativo (meson):** solo si tu distribución ya tiene GTK ≥ 4.22 y libadwaita ≥ 1.9.

## 1. Descargar el código

```bash
git clone https://github.com/merinodpepe/OnePepe.git
cd OnePepe
git submodule update --init --recursive
```

## 2. Con Flatpak

```bash
./setup-deps.sh --flatpak
flatpak-builder --user --install --force-clean _flatpak_app build-aux/com.github.flxzt.rnote.Devel.yaml
flatpak run com.github.flxzt.rnote.Devel
```

`setup-deps.sh --flatpak` instala el SDK de GNOME y las extensiones de Rust y LLVM si faltan.

## 3. Nativo con meson

Comprueba qué falta (y lo instala con tu permiso):

```bash
./setup-deps.sh --check   # solo mira
./setup-deps.sh           # instala lo que falte (pide sudo)
```

Después:

```bash
meson setup --prefix=/usr _mesonbuild
meson compile -C _mesonbuild
meson install -C _mesonbuild      # opcional: instalarlo en el sistema
```

Opciones útiles al configurar:

- `-Dprofile=devel`: perfil de desarrollo.
- `-Dcli=true`: compila también la herramienta de línea de órdenes.

Cambiar una opción más tarde: `meson configure -D<opción>=<valor> _mesonbuild`.

Desinstalar: `sudo -E ninja uninstall -C _mesonbuild`.

> El binario, el id de la aplicación (`com.github.flxzt.rnote`) y la extensión `.rnote` conservan el nombre
> interno heredado para no romper archivos ni ajustes. Ver `DECISIONES.md`.

## 4. Tests

```bash
cargo test -p rnote-engine -p rnote-compose
cd tools/onepepe-math && python3 -m unittest discover -s tests
```

## 5. Modo matemáticas

El reconocimiento de matemáticas es un programa auxiliar aparte. Ver `README.md` y `tools/onepepe-math/`.

Más detalle (también para compilar en una máquina sin GNOME 50): `CONTEXTO-IA.md`, sección 10 «Compilar, instalar y probar».
