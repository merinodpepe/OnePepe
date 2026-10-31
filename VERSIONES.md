# Versiones de OnePepe

Qué cambia en cada versión, de la más nueva a la más antigua. La versión instalada se ve en la barra de estado (abajo
a la derecha) y en «Acerca de», con el commit de git del que se compiló (por ejemplo `1.1.0 · 8c41d0e`).

Cómo se numera: `mayor.menor.arreglo`. Sube la **menor** con cada tanda de funciones nuevas y el **arreglo** cuando
solo se corrigen fallos. El número está en `meson.build` (`version:`) y en
`crates/rnote-ui/data/app.metainfo.xml.in.in` (`<releases>`); al cambiarlo, se añade aquí su apartado.

## 1.3.5 (2026-10-31) — la expresión del entreno siempre se ve

- En tu captura solo se veía el código LaTeX (`3+4\cdot 5=`): en tu equipo falta **matplotlib**, que es lo que
  dibuja las expresiones (se instala con `./instalar.sh`). Ahora, si no se puede dibujar, la expresión sale en
  **texto legible y grande** (`3 + 4 · 5 =`, `x² + y² = r²`, `β̂ = (Xᵀ X)⁻¹Xᵀ y`) y abajo pone qué instalar.
- Probado en la nube con y sin matplotlib.

## 1.3.4 (2026-10-31) — faltaban los sets del entreno

- **«Nuevo entreno» decía «file not found» en la carpeta `entrenos`**: los dos sets de 200 expresiones nunca se
  subieron al repositorio, porque `tools/onepepe-math/.gitignore` ignora todos los `.json` (para que no se suban
  datos personales). En la nube funcionaban porque estaban en mi copia local. Ahora hay una excepción solo para
  `entrenos/*.json` (tus `*.entreno.json` siguen fuera) y están subidos.
- Comprobado con una copia limpia del repositorio: están los dos sets y pasan los tests.

## 1.3.3 (2026-10-31) — arreglado el cierre al tocar el «?»

- **OnePepe se cerraba al tocar el «?»** de un «=» (tu informe de fallos lo pilló: «RefCell already borrowed» en
  `canvas/mod.rs`). Al tocar el «?», la app seguía usando el motor mientras abría la ventanita de la explicación; en
  tu escritorio, abrirla mueve el foco y lo que se ejecuta entonces también usa el motor: dos a la vez y se cerraba.
  Al tocar un número no pasaba porque esa ventanita soltaba el motor antes: por eso «a veces sí, a veces no».
- Además, las dos ventanitas (candidatas y explicación) se abren un instante después, cuando nada usa el motor.

## 1.3.2 (2026-10-31) — informe de fallos

- Me contaste que, con «Variables» y «Resultado del =» activos, a veces salía el «?», escribías el número detrás del
  «=» y **OnePepe se cerraba**. En la app de prueba de la nube no se cierra (probado: «x =», sale «?», escribo «3»
  encima y queda «x=3»), y no encuentro en el código nada que lo explique. Para pillarlo:
  - Si OnePepe se cierra por un fallo, ahora guarda un **informe** (`fallos.log`: versión, mensaje, dónde y la
    traza). Con Flatpak está en `~/.var/app/com.github.flxzt.rnote.Devel/data/onepepe/fallos.log`.
  - Al abrir la siguiente vez, avisa de dónde está para que me lo mandes.

## 1.3.1 (2026-10-31) — arreglos

- **«Nuevo entreno» no hacía nada** (tu prueba): si el programa de matemáticas no respondía, la app callaba. Ahora:
  - Dentro de Flatpak se lanza por su ruta completa, `~/.local/bin/onepepe-math`. Antes iba por su nombre, y el
    escritorio a menudo no tiene `~/.local/bin` en el PATH (solo la terminal), así que no arrancaba. Probablemente
    era también por qué no te salía el resultado del «=».
  - Si no responde, sale un aviso con la orden para comprobarlo. Si es más antiguo que la app, también lo dice.
  - Si falla la ventana para elegir el archivo, lo dice (cancelar no avisa).
- **Revisar pasos:** el aviso en rojo va debajo del renglón, no a su derecha, para no pisar el resultado del «=» ni
  el valor de las variables.

## 1.3 (2026-10-30)

- **Set «Tus asignaturas» (200)**: genética de poblaciones (Hardy-Weinberg, F, F_ST, N_e, deriva, selección,
  migración, desequilibrio de ligamiento, D de Tajima, heredabilidad), biofísica (Nernst, Goldman, Fick,
  Stokes-Einstein, Boltzmann, Gibbs, Poiseuille, Michaelis-Menten, Hill, Beer-Lambert, cinética, membrana) y
  aprendizaje estadístico (regresión, logística, verosimilitud, ridge/lasso, descenso de gradiente, softmax, Bayes,
  esperanza y varianza, normal, contrastes, métricas, AIC/BIC), con cálculo y álgebra, 30 de símbolos sueltos y 30
  desarrollos de varios pasos.
- **Elegir el set** al crear un entreno (General o Tus asignaturas).
- **Símbolos de estadística** en el reconocedor: ŷ, x̄, x̃ (acentos encima de una letra), 𝔼, ℙ, ℝ, ⊤, ⟨ ⟩, ∩, ∪, ⇌.
  ŷ y x̄ son variables propias (x̄ = 5 no cambia x). Medido: lo demás se reconoce igual.
- **Glosario de biofísica** (`tools/onepepe-math/glosarios/biofisica.txt`): k_B, R, F, N_A, h, c, q_e con
  unidades. Se copia a la carpeta de la asignatura como `.onepepe-glosario.txt`.

## 1.2 (2026-10-30)

- **Modo entreno** (pestaña Matemáticas → Tu letra → **Nuevo entreno** / **Abrir entreno**). Se ve la expresión
  dibujada y un recuadro para escribirla; con **Siguiente**, **Saltar**, **Anterior**, **Deshacer trazo** y
  **Borrar**. Para los cálculos seguidos hay un renglón guía por paso.
- **Set de 200**: 56 de símbolos sueltos (cada uno seis veces; los que más se confunden, dos veces), 104 expresiones
  de todos los temas y 40 cálculos seguidos de 2 a 4 renglones. Está en `tools/onepepe-math/entrenos/`.
- **Archivo de entreno** propio (`*.entreno.json`, en `Documentos/OnePepe entrenos`), no un `.rnote`: guarda el set,
  por dónde vas y tus trazos con sus tiempos y la versión de la app. Se guarda en cada paso; puedes parar y seguir.
- **Usar para entrenar**: lo escrito pasa a tus ejemplos (en tu equipo) cortado en símbolos sabiendo lo que dice;
  después, **Reentrenar**.
- El programa de matemáticas necesita **matplotlib** para dibujar las expresiones: `cd tools/onepepe-math &&
  ./instalar.sh` una vez (sin él se ve el LaTeX en texto).

## 1.1 (2026-10-30)

Primera versión numerada como OnePepe (antes se veía el número de Rnote, 0.15.0). Incluye todo lo hecho hasta ahora:

- **Base (fases 0–4):** hojas infinitas, 48 colores, capas, regla (con ángulo libre e infinita), tinta a formas,
  botones del lápiz, subíndices, catálogo de símbolos, fecha y hora, listas, cuadernos (nueva página, mover a…).
- **Cinta tipo OneNote** y marca OnePepe: Archivo, Inicio, Insertar, Dibujar, Matemáticas, Vista; bolis favoritos
  (Ctrl+1…5), estilos de texto, insertar tabla y plantillas, tamaño de hoja, exportar ajustado y por secciones, nota
  rápida, copias de seguridad, interfaz en español.
- **Diseño según tu guía:** temas claro/oscuro/alto contraste, lienzo oscuro, barra de estado, barra de opciones
  flotante, árbol del cuaderno con subpáginas, fondo y renglones, ajustes en pestañas, zoom del 5 % al 1600 % y
  vectorial, mano.
- **Modo matemáticas:** reconocimiento con tu letra (kNN y reentrenar), símbolos limpios, correcciones, unir y
  separar, archivo oculto por página, flechas y «;».
- **Arreglos del modo matemáticas (30/10):** escribir junto a símbolos y donde has borrado, zoom, reconocer
  selección, i y j, «Otro…», resultado del «=» siempre al día con «?», funciones (sin x, ln, log₂, sen, tg, ∛).
- **Plan tipo MyScript, primera versión (30/10):** tachar con zigzag, diccionario de símbolos, variables, revisar
  pasos, copiar como LaTeX, formas de una fracción y gráficas.
- **Versión visible** en la barra de estado y en «Acerca de».

Detalle de cada punto: `ESTADO.md` (cómo se probó), `CAMBIOS-ONEPEPE.md` (cómo probarlo), `DECISIONES.md` (por qué).
