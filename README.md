<div align="center">

<sub><b>PEPE FLOCK</b></sub>

<img src="misc/onepepe/logo.svg" width="200" alt="Logo de OnePepe">

# OnePepe

**Tus apuntes a mano, como en OneNote, pero en Linux.**

<sub>Un proyecto de la familia <b>Pepe Flock</b></sub>

</div>

OnePepe es una aplicación de apuntes a mano para Linux pensada para tabletas gráficas (Wacom) y para estudiar
asignaturas de ciencias: escribes con el lápiz en un lienzo infinito, ordenas tus apuntes en cuadernos, secciones y
páginas, conviertes tus trazos en formas limpias y, en el modo matemáticas, OnePepe reconoce lo que escribes, lo pasa a
limpio y te calcula el resultado del "=".

Está hecha en Rust con GTK4 y libadwaita, y funciona sobre Wayland.

## Qué puedes hacer con OnePepe

**Escribir y dibujar**

- Lápiz sensible a la presión, con estilos de trazo configurables y resaltador que no tapa lo escrito.
- Lienzo infinito en todas las direcciones (o páginas fijas, si lo prefieres).
- Paleta de 48 colores, de muy claros a muy oscuros, además de tus 9 colores favoritos y el selector personalizado.
- **Tinta a formas:** dibuja una línea, un círculo, una elipse, un triángulo o un rectángulo a mano y se convierte en la
  forma limpia. Con interruptor, control de sensibilidad y deshacer.
- **Regla** que se mueve y se gira con el lápiz, muestra el ángulo y endereza los trazos que haces a lo largo de su borde.
- Herramienta de formas, ejes cartesianos 2D y 3D, rejillas y flechas.

**Matemáticas**

- **Modo matemáticas** (Ctrl+Alt+M): OnePepe reconoce los símbolos que escribes y los redibuja limpios en el mismo
  sitio, sin tocar los que no tiene claros. Ctrl+Z devuelve tu letra.
- **Corrección con un toque:** toca un símbolo para elegir otra lectura, para ese símbolo o para todos los que se le
  parecen. Cada corrección le enseña tu letra (y se queda en tu equipo).
- **Resultado del "="** en gris junto a lo que escribes: calcula, despeja ecuaciones, simplifica, deriva, integra,
  sumatorios y límites, con unidades y constantes físicas y comprobación de dimensiones.
- **Glosario por cuaderno** con los símbolos y constantes de cada asignatura (k_B, N_A, ΔG…).

**Organizar**

- **Cuadernos, secciones y páginas** en la barra lateral: crea páginas nuevas, muévelas entre secciones, renómbralas,
  duplícalas o bórralas (con confirmación).
- Pestañas para tener varias páginas abiertas, autoguardado e impresión.

**Seleccionar y ordenar**

- Selección con lazo o rectángulo para mover, girar y redimensionar.
- **Traer al frente, enviar al fondo, subir y bajar un nivel**, también con imágenes y páginas de PDF, que se pueden
  poner por encima de la tinta.
- Importa PDF, imágenes y SVG como objetos que puedes mover, girar y redimensionar.
- Exporta a PDF, SVG, PNG, JPEG y Xopp.

**Texto**

- Negrita, cursiva, subrayado, tachado y color.
- **Superíndices y subíndices** (x², H₂O), **catálogo de símbolos** (griego, operadores, ∂ ∇ ∫ ∑ √ ≈ ≠ ≤ ≥ →…),
  **viñetas, listas numeradas y casillas** ☐ ☑, y **fecha y hora** con un clic.

**El lápiz**

- Botones del lápiz configurables: por defecto, el botón 1 borra trazos enteros y el botón 2 selecciona con lazo
  mientras los mantienes pulsados. Modos "mientras pulso" o "alterna al pulsar". Extremo borrador compatible.
- Borrador de trazo entero o por área, en tamaño pequeño, mediano o grande.

## Instalación

OnePepe necesita GNOME 50 (GTK ≥ 4.22, libadwaita ≥ 1.9, glib ≥ 2.88). Lo más sencillo es compilarlo con **Flatpak**,
que trae todo lo necesario:

```bash
git clone https://github.com/merinodpepe/OnePepe.git
cd OnePepe
git checkout onepepe
git submodule update --init --recursive
./setup-deps.sh --flatpak
flatpak-builder --user --install --force-clean _flatpak_app build-aux/com.github.flxzt.rnote.Devel.yaml
flatpak run com.github.flxzt.rnote.Devel
```

Si tu distribución ya trae GNOME 50, puedes compilarlo de forma nativa: `./setup-deps.sh` te dice qué falta y te da
los comandos. Más detalles en [BUILDING.md](./BUILDING.md).

### Modo matemáticas

El reconocimiento de matemáticas es un programa auxiliar en Python que se instala aparte, una sola vez:

```bash
cd tools/onepepe-math
./instalar.sh --entrenar
```

Descarga el conjunto de datos público MathWriting y entrena el reconocedor de símbolos en tu equipo (unos minutos).
Todo se guarda en `~/.local/share/onepepe/`; tus correcciones nunca salen de tu ordenador. Detalles en
[tools/onepepe-math/README.md](./tools/onepepe-math/README.md).

## Documentación

| Documento | Para qué |
|---|---|
| [ESTADO.md](./ESTADO.md) | Qué está hecho, cómo se ha probado y qué falta |
| [CAMBIOS-ONEPEPE.md](./CAMBIOS-ONEPEPE.md) | Cada función de OnePepe y cómo probarla con el lápiz |
| [MATEMATICAS-INFORME.md](./MATEMATICAS-INFORME.md) | Cómo funciona el reconocimiento de matemáticas y sus resultados |
| [DECISIONES.md](./DECISIONES.md) | Por qué se hizo cada cosa así |
| [AUDITORIA.md](./AUDITORIA.md) | Qué traía ya la base y qué se añadió |
| [PLAN-PENDIENTE.md](./PLAN-PENDIENTE.md) | El plan de mejora: lo que falta por hacer |

## Problemas conocidos

- **Usa Wayland.** En X11 la entrada del lápiz y de la pantalla táctil no es fiable.
- **Si los botones del lápiz no responden**, comprueba que `libinput` y `libwacom` están instalados (`./setup-deps.sh --check`).
  Para ver qué evento envía cada botón usa `evtest` (instrucciones en [CAMBIOS-ONEPEPE.md](./CAMBIOS-ONEPEPE.md)).
- **Algunos lápices** (no las Wacom) tienen un botón que se presenta como "goma"; la aplicación no puede distinguirlo
  del extremo borrador. Elige qué hace en Ajustes → "Tool for the eraser action".
- **Con Flatpak**, guarda tus cuadernos en Documentos, Imágenes o Escritorio: son las carpetas que la aplicación puede ver.
- **El reconocimiento de matemáticas aún está aprendiendo:** sin muestras de tu letra se equivoca a menudo; por eso
  viene apagado y deja intactos los símbolos dudosos.

## Formato de archivo

Las páginas se guardan como archivos `.rnote` (JSON comprimido con gzip), compatibles con Rnote 0.15. OnePepe no
cambia el formato: lo que necesita recordar el modo matemáticas va en un archivo oculto junto a cada página
(`.<página>.rnote.onepepe.json`).

## Licencia

OnePepe es software libre bajo la licencia GPL-3.0-or-later. Consulta el archivo [LICENSE](./LICENSE).

    Copyright (C) 2026  merinodpepe (OnePepe)
    Copyright (C) 2023  The Rnote Authors

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

## Créditos

- OnePepe está construido sobre [Rnote](https://github.com/flxzt/rnote), de The Rnote Authors (GPL-3.0-or-later).
  Gracias a sus autores, colaboradores y traductores.
- Sonidos del lápiz: [Freesound](https://freesound.org/) (créditos en `sounds/Licenses.md`).
- Formas a mano alzada: algoritmos de [Rough.js](https://roughjs.com/).
- Símbolos limpios del modo matemáticas: fuentes de trazo único de Dr. A. V. Hershey (U.S. National Bureau of
  Standards), formato de datos de James Hurt (Cognition, Inc.), a través del paquete
  [Hershey-Fonts](https://github.com/apshu/HersheyFonts) (MIT).
- Datos para entrenar el reconocimiento de matemáticas: [MathWriting](https://github.com/google-research/google-research/tree/master/mathwriting)
  de Google (CC BY-NC-SA 4.0), que se descarga en tu equipo y no se distribuye con OnePepe.
- Cálculo: [SymPy](https://www.sympy.org/), [Pint](https://pint.readthedocs.io/) y [SciPy](https://scipy.org/).
- Tipografías incluidas: [Grape Nuts](https://fonts.google.com/specimen/Grape+Nuts),
  [OpenDyslexic](https://github.com/antijingoist/opendyslexic), [TT2020](https://github.com/ctrlcctrlv/TT2020) y
  [Virgil](https://virgil.excalidraw.com/).
