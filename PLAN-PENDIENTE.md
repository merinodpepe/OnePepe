# Plan de mejora de OnePepe (pendiente)

Actualizado: 2026-10-26. Solo lo que falta por hacer. Lo ya hecho está en [`ESTADO.md`](ESTADO.md) y
[`CAMBIOS-ONEPEPE.md`](CAMBIOS-ONEPEPE.md); el plan original completo queda en el historial de git de este archivo.

## 1. Pruebas en tu equipo (lo primero)

Nada se ha ejecutado todavía en la aplicación ni probado con lápiz real. Seguir `CAMBIOS-ONEPEPE.md` y anotar qué falla:

- Hojas infinitas con zoom y desplazamiento.
- Tinta a formas: cuántas formas acierta y cuántas letras convierte por error (para ajustar la sensibilidad).
- Matemáticas: modo matemáticas, sustitución, corrección por toque, resultado del "=", archivo oculto al reabrir.
- Botones del lápiz y extremo borrador, con la salida de `evtest`.
- Regla (tamaño de la zona de enganche y ancho), colores, capas, cuadernos y herramientas de texto.

## 2. Matemáticas con tu letra

Bloqueado hasta tener tus datos (instrucciones en `tools/onepepe-math/README.md`):

- Recibir `mis_expresiones_1.rnote` (las 30 expresiones de `tools/onepepe-math/expresiones.txt`) y `mis_simbolos.rnote`,
  adjuntos en el chat o en tu Google Drive, nunca en el repositorio.
- Respuestas cortas: signo de multiplicar, coma o punto decimal, unidades en el mismo renglón, asignaturas y símbolos
  para el glosario.
- Medir el reconocimiento con tu letra, ajustar umbrales, agrupación y estructura (potencias, subíndices, fracciones)
  a tu escritura "poco lineal", y entrenar el clasificador personal con tus ejemplos.
- Validar con tu letra, una por una, las etapas: aritmética y álgebra → unidades y constantes → ecuaciones, derivadas e
  integrales → matrices, sumatorios y límites.
- Preparar el glosario de cada asignatura (biofísica y las que me digas).
- **Modo matemáticas solo mientras se mantiene pulsado un botón del lápiz** (estaba en el plan como "si es barato" y
  no está hecho).

## 3. Que la aplicación se llame OnePepe por dentro

Hoy el README y la documentación ya son de OnePepe, pero la aplicación sigue mostrando "Rnote": título de la ventana,
"Acerca de", icono, nombre en el menú de aplicaciones e identificador (`com.github.flxzt.rnote`). Cambiarlo implica:

- **Nombre visible, "Acerca de" e icono:** cambio sencillo (textos, logo de `misc/onepepe/logo.svg` como icono de la app,
  autores y enlaces apuntando a tu repositorio, conservando el crédito a Rnote que exige la GPL).
- **Identificador de la app** (p. ej. `io.github.merinodpepe.OnePepe`). Es lo delicado:
  - **Ajustes guardados:** Rnote guarda los ajustes (colores favoritos, botones del lápiz, cuadernos de la barra
    lateral, herramientas…) con su identificador. Con uno nuevo, OnePepe empezaría con los ajustes por defecto. Se
    puede añadir una migración que copie los de Rnote la primera vez.
  - **Instalación en paralelo:** con un identificador propio, OnePepe y Rnote se pueden instalar a la vez sin pisarse
    (Flatpak, menú de aplicaciones, ajustes). Hay que renombrar el manifiesto de Flatpak, el archivo `.desktop`, el
    esquema de ajustes, el icono y los metadatos.
  - **Archivos `.rnote`:** no cambian; ambos programas los abren. Hay que decidir cuál los abre por defecto con doble clic.
  - **Traducciones y textos:** los nombres "Rnote" en las cadenas de la interfaz pasarían a "OnePepe".
- Antes de hacerlo: confirmar contigo el identificador y si quieres migrar los ajustes de Rnote.

## 4. Mejoras opcionales (si sobra tiempo y crédito)

- Listas que continúan solas al pulsar Intro (ahora hay que pulsar el botón en cada párrafo).
- Estilos rápidos de texto: Título 1, Título 2, cita, código.
- Recorte de pantalla.
- Árbol desplegable de tres niveles (cuaderno → sección → página) en la barra lateral, en lugar de entrar en carpetas.
- Capturas de pantalla de OnePepe para el README (cuando funcione en tu equipo).
- Tinta a texto (Fase 5; poco prioritaria).

## 5. Cosas que haces tú en GitHub

- Para que la portada del repositorio muestre el README de OnePepe sin tocar `main`: **Settings → General → Default
  branch** → `onepepe`.

## No hacer

Copiar formato, adjuntar archivos como icono, búsqueda de imágenes en línea, tinta con texturas, editor de ecuaciones
nativo, enlaces entre páginas y notas, grabación de audio. No cambiar el formato `.rnote` sin avisar, no tocar `main`
ni otros repositorios ni la licencia, no subir modelos, datasets ni datos personales.

## Orden de prioridad

1) Pruebas en tu equipo, 2) matemáticas con tu letra, 3) nombre OnePepe dentro de la app, 4) mejoras opcionales.
