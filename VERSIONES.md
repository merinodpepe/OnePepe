# Versiones de OnePepe

Qué cambia en cada versión, de la más nueva a la más antigua. La versión instalada se ve en la barra de estado (abajo
a la derecha) y en «Acerca de», con el commit de git del que se compiló (por ejemplo `1.1.0 · 8c41d0e`).

Cómo se numera: `mayor.menor.arreglo`. Sube la **menor** con cada tanda de funciones nuevas y el **arreglo** cuando
solo se corrigen fallos. El número está en `meson.build` (`version:`) y en
`crates/rnote-ui/data/app.metainfo.xml.in.in` (`<releases>`); al cambiarlo, se añade aquí su apartado.

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
